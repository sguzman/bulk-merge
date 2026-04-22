use crate::config::AppConfig;
use crate::db::{Db, PgTargetType};
use crate::openlibrary::parser::parse_line;
use crate::progress::{ProgressConfig, ProgressTicker};
use anyhow::Context as _;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use tracing::{info, instrument, warn};

pub struct OlIngestPlan {
    pub dump_path: String,
    pub table_name: String, // authors, editions, or works
    pub schema: String,
    pub max_records: Option<usize>,
}

fn get_table_definition(table_name: &str) -> (Vec<String>, Vec<PgTargetType>) {
    let mut columns = vec![
        "ol_key".to_string(),
        "ol_type".to_string(),
        "revision".to_string(),
        "last_modified".to_string(),
    ];
    let mut types = vec![
        PgTargetType::Text,
        PgTargetType::Text,
        PgTargetType::Int4,
        PgTargetType::Timestamptz,
    ];

    match table_name {
        "authors" => {
            columns.push("name".to_string());
            types.push(PgTargetType::Text);
        }
        "works" => {
            columns.push("title".to_string());
            types.push(PgTargetType::Text);
            columns.push("author_keys".to_string());
            types.push(PgTargetType::TextArray);
            columns.push("subjects".to_string());
            types.push(PgTargetType::TextArray);
        }
        "editions" => {
            columns.push("title".to_string());
            types.push(PgTargetType::Text);
            columns.push("isbn_10".to_string());
            types.push(PgTargetType::TextArray);
            columns.push("isbn_13".to_string());
            types.push(PgTargetType::TextArray);
            columns.push("publishers".to_string());
            types.push(PgTargetType::TextArray);
            columns.push("publish_date".to_string());
            types.push(PgTargetType::Text);
        }
        _ => {}
    }

    columns.push("data".to_string());
    types.push(PgTargetType::Jsonb);

    (columns, types)
}

#[instrument(skip_all, fields(table = %plan.table_name, dump = %plan.dump_path))]
pub async fn ingest_openlibrary_dump(
    db: &Db,
    config: &AppConfig,
    plan: &OlIngestPlan,
    import_run_id: i64,
) -> anyhow::Result<()> {
    let file = std::fs::File::open(&plan.dump_path)
        .with_context(|| format!("failed to open dump file `{}`", plan.dump_path))?;
    let size_bytes = file.metadata().ok().map(|m| m.len() as i64);

    db.upsert_import_file(import_run_id, &plan.dump_path, size_bytes, "in_progress")
        .await
        .context("failed to upsert bm_meta.import_file")?;

    let (columns, types) = get_table_definition(&plan.table_name);

    // Provision table
    db.ensure_schema(&plan.schema).await?;
    provision_ol_table(db, &plan.schema, &plan.table_name, &columns, &types).await?;

    let checkpoint_key = format!("openlibrary:{}:{}", plan.table_name, plan.dump_path);
    let start_line = db
        .get_checkpoint_offset(import_run_id, &checkpoint_key)
        .await?
        .unwrap_or(0);

    let reader = BufReader::new(file);
    let mut ticker = ProgressTicker::new(ProgressConfig {
        log_interval: Duration::from_secs(config.progress.log_interval_seconds),
    });

    let mut lines_processed: u64 = 0;
    let mut batch: Vec<Vec<Option<String>>> =
        Vec::with_capacity(config.execution.batch.max_rows as usize);


    for (i, line) in reader.lines().enumerate() {
        let line_num = i as u64;
        if line_num < start_line {
            continue;
        }

        let line = line?;
        lines_processed += 1;

        if let Some(max) = plan.max_records {
            if lines_processed > max as u64 {
                info!(max, "reached max_records limit, stopping");
                break;
            }
        }

        match parse_line(&line) {
            Ok(rec) => {
                let mut row = vec![
                    Some(rec.ol_key),
                    Some(rec.ol_type),
                    Some(rec.revision.to_string()),
                    Some(rec.last_modified.to_rfc3339()),
                ];

                match plan.table_name.as_str() {
                    "authors" => {
                        row.push(rec.name);
                    }
                    "works" => {
                        row.push(rec.title);
                        row.push(Some(to_pg_array(&rec.author_keys)));
                        row.push(Some(to_pg_array(&rec.subjects)));
                    }
                    "editions" => {
                        row.push(rec.title);
                        row.push(Some(to_pg_array(&rec.isbn_10)));
                        row.push(Some(to_pg_array(&rec.isbn_13)));
                        row.push(Some(to_pg_array(&rec.publishers)));
                        row.push(rec.publish_date);
                    }
                    _ => {}
                }

                row.push(Some(rec.data.to_string()));
                batch.push(row);
            }
            Err(e) => {
                warn!(line_num, error = %e, "failed to parse line, skipping");
                continue;
            }
        }

        if batch.len() >= config.execution.batch.max_rows as usize {
            flush_batch(db, &plan.schema, &plan.table_name, &columns, &types, &batch).await?;
            batch.clear();

            db.set_checkpoint_offset(import_run_id, &checkpoint_key, line_num + 1)
                .await?;
        }

        ticker.maybe_log("ol_ingest", line_num, None, || {
            info!(lines_processed, "progress");
        });
    }

    if !batch.is_empty() {
        flush_batch(db, &plan.schema, &plan.table_name, &columns, &types, &batch).await?;
    }

    // Mark as finished by setting a high value or a convention.
    // Here we'll just set it to lines_processed + start_line or something.
    db.set_checkpoint_offset(import_run_id, &checkpoint_key, lines_processed + start_line)
        .await?;

    Ok(())
}

async fn provision_ol_table(
    db: &Db,
    schema: &str,
    table: &str,
    columns: &[String],
    types: &[PgTargetType],
) -> anyhow::Result<()> {
    let schema_q = crate::db::quote_ident(schema);
    let table_q = crate::db::quote_ident(table);

    let mut col_defs = Vec::new();
    for (i, col) in columns.iter().enumerate() {
        let name_q = crate::db::quote_ident(col);
        let ty = &types[i];
        let ty_sql = match ty {
            PgTargetType::Text => "TEXT",
            PgTargetType::Int4 => "INTEGER",
            PgTargetType::Timestamptz => "TIMESTAMPTZ",
            PgTargetType::TextArray => "TEXT[]",
            PgTargetType::Jsonb => "JSONB",
            _ => "TEXT",
        };
        let pk = if col == "ol_key" { " PRIMARY KEY" } else { "" };
        let not_null = if col == "ol_type" || col == "revision" || col == "last_modified" || col == "data" {
            " NOT NULL"
        } else {
            ""
        };
        col_defs.push(format!("{} {}{}{}", name_q, ty_sql, pk, not_null));
    }

    let drop_sql = format!("DROP TABLE IF EXISTS {}.{}", schema_q, table_q);
    db.execute_raw(&drop_sql).await?;

    let sql = format!(
        "CREATE TABLE {}.{} ({})",
        schema_q,
        table_q,
        col_defs.join(", ")
    );

    db.execute_raw(&sql).await?;
    Ok(())
}

async fn flush_batch(
    db: &Db,
    schema: &str,
    table: &str,
    columns: &[String],
    types: &[PgTargetType],
    batch: &[Vec<Option<String>>],
) -> anyhow::Result<()> {
    db.upsert_rows_from_text_with_types(schema, table, columns, types, &["ol_key".to_string()], batch)
        .await?;
    Ok(())
}

fn to_pg_array(vals: &[String]) -> String {
    let joined = vals
        .iter()
        .map(|v| format!("\"{}\"", v.replace('\"', "\\\"")))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{}}}", joined)
}
