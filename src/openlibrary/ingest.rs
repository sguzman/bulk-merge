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
        "created".to_string(),
        "last_modified".to_string(),
    ];
    let mut types = vec![
        PgTargetType::Text,
        PgTargetType::Text,
        PgTargetType::Int4,
        PgTargetType::Timestamptz,
        PgTargetType::Timestamptz,
    ];

    match table_name {
        "authors" => {
            columns.extend(vec![
                "name".to_string(),
                "birth_date".to_string(),
                "death_date".to_string(),
                "bio".to_string(),
                "website".to_string(),
            ]);
            types.extend(vec![
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::Text,
            ]);
        }
        "works" => {
            columns.extend(vec![
                "title".to_string(),
                "subtitle".to_string(),
                "description".to_string(),
                "author_keys".to_string(),
                "subjects".to_string(),
                "subject_people".to_string(),
                "subject_places".to_string(),
                "subject_times".to_string(),
                "covers".to_string(),
            ]);
            types.extend(vec![
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::Int8Array,
            ]);
        }
        "editions" => {
            columns.extend(vec![
                "title".to_string(),
                "subtitle".to_string(),
                "isbn_10".to_string(),
                "isbn_13".to_string(),
                "publishers".to_string(),
                "publish_date".to_string(),
                "number_of_pages".to_string(),
                "physical_format".to_string(),
                "languages".to_string(),
                "lc_classifications".to_string(),
                "dewey_decimal_class".to_string(),
                "notes".to_string(),
            ]);
            types.extend(vec![
                PgTargetType::Text,
                PgTargetType::Text,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::Text,
                PgTargetType::Int4,
                PgTargetType::Text,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::TextArray,
                PgTargetType::Text,
            ]);
        }
        _ => {}
    }

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
                    rec.created.map(|dt| dt.to_rfc3339()),
                    Some(rec.last_modified.to_rfc3339()),
                ];

                match plan.table_name.as_str() {
                    "authors" => {
                        row.extend(vec![
                            rec.name,
                            rec.birth_date,
                            rec.death_date,
                            rec.bio,
                            rec.website,
                        ]);
                    }
                    "works" => {
                        row.extend(vec![
                            rec.title,
                            rec.subtitle,
                            rec.description,
                            Some(to_pg_array(&rec.author_keys)),
                            Some(to_pg_array(&rec.subjects)),
                            Some(to_pg_array(&rec.subject_people)),
                            Some(to_pg_array(&rec.subject_places)),
                            Some(to_pg_array(&rec.subject_times)),
                            Some(to_pg_int_array(&rec.covers)),
                        ]);
                    }
                    "editions" => {
                        row.extend(vec![
                            rec.title,
                            rec.subtitle,
                            Some(to_pg_array(&rec.isbn_10)),
                            Some(to_pg_array(&rec.isbn_13)),
                            Some(to_pg_array(&rec.publishers)),
                            rec.publish_date,
                            rec.number_of_pages.map(|i| i.to_string()),
                            rec.physical_format,
                            Some(to_pg_array(&rec.languages)),
                            Some(to_pg_array(&rec.lc_classifications)),
                            Some(to_pg_array(&rec.dewey_decimal_class)),
                            rec.notes,
                        ]);
                    }
                    _ => {}
                }

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
            PgTargetType::Int8 => "BIGINT",
            PgTargetType::Timestamptz => "TIMESTAMPTZ",
            PgTargetType::TextArray => "TEXT[]",
            PgTargetType::Int8Array => "BIGINT[]",
            PgTargetType::Jsonb => "JSONB",
            _ => "TEXT",
        };
        let pk = if col == "ol_key" { " PRIMARY KEY" } else { "" };
        let not_null = if col == "ol_type" || col == "revision" || col == "last_modified" {
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
    _types: &[PgTargetType],
    batch: &[Vec<Option<String>>],
) -> anyhow::Result<()> {
    db.copy_rows_text_tsv(schema, table, columns, batch).await?;
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

fn to_pg_int_array(vals: &[i64]) -> String {
    let joined = vals
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{}}}", joined)
}
