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

    // Provision table
    db.ensure_schema(&plan.schema).await?;
    provision_ol_table(db, &plan.schema, &plan.table_name).await?;

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

    let columns = vec![
        "ol_key".to_string(),
        "ol_type".to_string(),
        "revision".to_string(),
        "last_modified".to_string(),
        "data".to_string(),
    ];
    let types = vec![
        PgTargetType::Text,
        PgTargetType::Text,
        PgTargetType::Int4,
        PgTargetType::Timestamptz,
        PgTargetType::Jsonb,
    ];

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
                let row = vec![
                    Some(rec.ol_key),
                    Some(rec.ol_type),
                    Some(rec.revision.to_string()),
                    Some(rec.last_modified.to_rfc3339()),
                    Some(rec.data.to_string()),
                ];
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

async fn provision_ol_table(db: &Db, schema: &str, table: &str) -> anyhow::Result<()> {
    let schema_q = crate::db::quote_ident(schema);
    let table_q = crate::db::quote_ident(table);
    let sql = format!(
        r#"
CREATE TABLE IF NOT EXISTS {schema_q}.{table_q} (
    ol_key TEXT PRIMARY KEY,
    ol_type TEXT NOT NULL,
    revision INTEGER NOT NULL,
    last_modified TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);
"#
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
