use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::Db;
use crate::libgen::mysql_dump::{parse_insert_into, seek_to_offset, StatementReader, TableDef, Value};
use crate::progress::{ProgressConfig, ProgressTicker};
use anyhow::Context as _;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, instrument};

#[derive(Debug, Clone)]
pub struct IngestPlan {
    pub kind: LibgenDumpKind,
    pub dump_path: String,
    pub table_defs: Vec<TableDef>,
    pub overall_prefix: String,
    pub kind_prefix: String,
}

impl IngestPlan {
    pub fn pg_table_for_mysql(&self, mysql_table: &str) -> String {
        format!("{}{}{}", self.overall_prefix, self.kind_prefix, mysql_table)
    }
}

#[instrument(skip_all, fields(kind = ?plan.kind, dump = %plan.dump_path))]
pub async fn ingest_dump_rows(
    db: &Db,
    config: &AppConfig,
    plan: &IngestPlan,
    import_run_id: i64,
) -> anyhow::Result<()> {
    let mut table_map: HashMap<String, TableDef> = HashMap::new();
    for def in &plan.table_defs {
        table_map.insert(def.name.clone(), def.clone());
    }

    let mut file = std::fs::File::open(&plan.dump_path)
        .with_context(|| format!("failed to open dump file `{}`", plan.dump_path))?;
    let size_bytes = file.metadata().ok().map(|m| m.len() as i64);
    let import_file_id = db
        .upsert_import_file(import_run_id, &plan.dump_path, size_bytes, "in_progress")
        .await
        .context("failed to upsert bm_meta.import_file")?;

    let checkpoint_key = format!("libgen:{}:{}", format!("{:?}", plan.kind).to_lowercase(), plan.dump_path);
    let start_offset = if config.libgen.resume.enabled {
        db.get_checkpoint_offset(import_run_id, &checkpoint_key)
            .await
            .unwrap_or(None)
            .unwrap_or(0)
    } else {
        0
    };

    if start_offset > 0 {
        info!(start_offset, "resuming from checkpoint");
        seek_to_offset(&mut file, start_offset).context("failed to seek dump file")?;
    }

    let mut stmt_reader =
        StatementReader::new_with_offset(file, config.libgen.dump.max_statement_bytes, start_offset);

    let mut ticker = ProgressTicker::new(ProgressConfig {
        log_interval: Duration::from_secs(config.progress.log_interval_seconds),
    });

    let mut rows_loaded: u64 = 0;
    let mut rows_seen: u64 = 0;
    while let Some(stmt) = stmt_reader
        .next_statement()
        .context("failed reading statement")?
    {
        let current_offset = stmt_reader.offset();
        let total = size_bytes.and_then(|s| u64::try_from(s).ok());
        ticker.maybe_log("libgen_ingest", current_offset, total, || {
            info!(rows_seen, rows_loaded, offset = current_offset, "progress_detail");
        });

        let Some(insert) = parse_insert_into(&stmt).context("failed parsing INSERT INTO")? else {
            if config.libgen.raw.enabled && config.libgen.raw.store_other_statements {
                db.insert_libgen_raw_statement(import_run_id, current_offset as i64, "other", None, &stmt)
                    .await
                    .context("failed inserting raw_statement")?;
            }
            continue;
        };

        if config.libgen.raw.enabled {
            db.insert_libgen_raw_statement(
                import_run_id,
                current_offset as i64,
                "insert_into",
                Some(&insert.table),
                &stmt,
            )
            .await
            .context("failed inserting raw_statement")?;
        }
        rows_seen += insert.rows.len() as u64;

        let Some(def) = table_map.get(&insert.table) else {
            // Some dumps may include tables we didn't provision (or we skipped). Ignore for now.
            continue;
        };

        let pg_table = plan.pg_table_for_mysql(&insert.table);
        let cols: Vec<String> = def.columns.iter().map(|c| c.name.clone()).collect();

        let mut row_buf: Vec<Vec<Option<String>>> = Vec::new();
        for row in insert.rows {
            let mut out_row: Vec<Option<String>> = Vec::with_capacity(row.len());
            for v in row {
                match v {
                    Value::Null => out_row.push(None),
                    Value::Text(s) => out_row.push(Some(s)),
                }
            }
            row_buf.push(out_row);
        }

        // Chunk large INSERT statements to configured batch size.
        let batch_max = config.execution.batch.max_rows.max(1) as usize;
        for chunk in row_buf.chunks(batch_max) {
            db.insert_rows_text(&config.postgres.schema_libgen, &pg_table, &cols, chunk)
                .await
                .with_context(|| format!("failed inserting rows into `{}`", pg_table))?;
            rows_loaded += chunk.len() as u64;
        }

        if config.libgen.resume.enabled {
            let off = stmt_reader.offset();
            db.set_checkpoint_offset(import_run_id, &checkpoint_key, off)
                .await
                .context("failed updating checkpoint")?;
            db.update_import_file_progress(
                import_file_id,
                off as i64,
                rows_seen as i64,
                rows_loaded as i64,
                "in_progress",
            )
            .await
            .context("failed updating import_file progress")?;
        }
    }

    db.update_import_file_progress(
        import_file_id,
        stmt_reader.offset() as i64,
        rows_seen as i64,
        rows_loaded as i64,
        "succeeded",
    )
    .await
    .context("failed finalizing import_file progress")?;

    info!(rows_loaded, "ingest completed");
    Ok(())
}
