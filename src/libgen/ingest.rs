use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::Db;
use crate::libgen::mysql_dump::{parse_insert_into, seek_to_offset, statement_preview, StatementReader, TableDef, Value};
use crate::progress::{ProgressConfig, ProgressTicker};
use anyhow::Context as _;
use sha2::{Digest as _, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, instrument};

fn sanitize_text(config: &AppConfig, s: String) -> String {
    if !config.libgen.dump.sanitize_nul_bytes {
        return s;
    }
    if !s.contains('\0') {
        return s;
    }
    s.replace('\0', &config.libgen.dump.nul_replacement)
}

#[derive(Debug, Clone)]
pub struct IngestPlan {
    pub kind: LibgenDumpKind,
    pub dump_path: String,
    pub table_defs: Vec<TableDef>,
    pub overall_prefix: String,
    pub kind_prefix: String,
    pub mode: IngestMode,
    pub conflict_columns: Vec<String>,
    pub apply_deletes: bool,
    pub row_hash_enabled: bool,
}

impl IngestPlan {
    pub fn pg_table_for_mysql(&self, mysql_table: &str) -> String {
        format!("{}{}{}", self.overall_prefix, self.kind_prefix, mysql_table)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestMode {
    Ingest,
    Update,
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
    let mut seen_pk_buf: Vec<String> = Vec::new();
    while let Some(stmt) = stmt_reader
        .next_statement()
        .context("failed reading statement")?
    {
        let current_offset = stmt_reader.offset();
        let total = size_bytes.and_then(|s| u64::try_from(s).ok());
        ticker.maybe_log("libgen_ingest", current_offset, total, || {
            info!(rows_seen, rows_loaded, offset = current_offset, "progress_detail");
        });

        let Some(insert) = parse_insert_into(&stmt).with_context(|| {
            let preview = statement_preview(&stmt, config.libgen.dump.error_preview_bytes as usize);
            format!("failed parsing INSERT INTO at offset_end={current_offset}: {preview}")
        })? else {
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
        let mut cols: Vec<String> = def.columns.iter().map(|c| c.name.clone()).collect();
        if plan.row_hash_enabled {
            cols.push("_bm_row_hash".to_string());
        }

        let pk_col = if plan.mode == IngestMode::Update && plan.apply_deletes {
            // Phase 1 limitation: only support single-column PK for delete handling.
            if plan.conflict_columns.len() != 1 {
                anyhow::bail!(
                    "apply_deletes requires exactly 1 primary key column, got {}",
                    plan.conflict_columns.len()
                );
            }
            Some(plan.conflict_columns[0].clone())
        } else {
            None
        };
        let pk_index = pk_col
            .as_ref()
            .and_then(|pk| cols.iter().position(|c| c == pk));

        // Chunk large INSERT statements without buffering the entire INSERT in memory.
        let batch_max_rows = config.execution.batch.max_rows.max(1) as usize;
        let batch_max_bytes = config.execution.batch.max_bytes.max(1) as usize;
        let mem_limit = config.execution.memory_hard_limit_bytes.max(1) as usize;

        let mut chunk: Vec<Vec<Option<String>>> = Vec::with_capacity(batch_max_rows.min(1024));
        let mut chunk_bytes: usize = 0;

        for row in insert.rows {
            let mut out_row: Vec<Option<String>> = Vec::with_capacity(row.len());
            let mut pk_value_for_seen: Option<String> = None;
            for v in row {
                match v {
                    Value::Null => out_row.push(None),
                    Value::Text(s) => {
                        let s = sanitize_text(config, s);
                        chunk_bytes = chunk_bytes.saturating_add(s.len());
                        out_row.push(Some(s));
                    }
                }
            }

            if plan.row_hash_enabled {
                let mut hasher = Sha256::new();
                for (i, v) in out_row.iter().enumerate() {
                    if i > 0 {
                        hasher.update(b"\t");
                    }
                    match v {
                        None => hasher.update(b"\\N"),
                        Some(s) => hasher.update(s.as_bytes()),
                    }
                }
                let hash = hasher.finalize();
                let hex = hex::encode(hash);
                chunk_bytes = chunk_bytes.saturating_add(hex.len());
                out_row.push(Some(hex));
            }

            if let Some(idx) = pk_index {
                if let Some(Some(v)) = out_row.get(idx) {
                    pk_value_for_seen = Some(v.clone());
                }
            }
            if let Some(v) = pk_value_for_seen {
                seen_pk_buf.push(v);
            }

            chunk.push(out_row);

            if chunk.len() >= batch_max_rows || chunk_bytes >= batch_max_bytes || chunk_bytes >= mem_limit / 2
            {
                match plan.mode {
                    IngestMode::Ingest => {
                        if plan.row_hash_enabled {
                            db.insert_rows_text_on_conflict_do_nothing(
                                &config.postgres.schema_libgen,
                                &pg_table,
                                &cols,
                                &["_bm_row_hash".to_string()],
                                &chunk,
                            )
                            .await
                            .with_context(|| format!("failed inserting (dedupe) into `{}`", pg_table))?;
                        } else {
                            match config.execution.loader.kind {
                                crate::config::LoaderKind::Copy => {
                                db.copy_rows_text_tsv(
                                    &config.postgres.schema_libgen,
                                    &pg_table,
                                    &cols,
                                    &chunk,
                                )
                                .await
                                .with_context(|| format!("failed COPY into `{}`", pg_table))?;
                                }
                                crate::config::LoaderKind::Insert => {
                                db.insert_rows_text(
                                    &config.postgres.schema_libgen,
                                    &pg_table,
                                    &cols,
                                    &chunk,
                                )
                                .await
                                .with_context(|| format!("failed inserting rows into `{}`", pg_table))?;
                                }
                            }
                        }
                    }
                    IngestMode::Update => {
                        if plan.row_hash_enabled {
                            db.insert_rows_text_on_conflict_do_nothing(
                                &config.postgres.schema_libgen,
                                &pg_table,
                                &cols,
                                &plan.conflict_columns,
                                &chunk,
                            )
                            .await
                            .with_context(|| format!("failed inserting (dedupe) into `{}`", pg_table))?;
                        } else {
                            db.upsert_rows_text(
                                &config.postgres.schema_libgen,
                                &pg_table,
                                &cols,
                                &plan.conflict_columns,
                                &chunk,
                            )
                            .await
                            .with_context(|| format!("failed upserting rows into `{}`", pg_table))?;
                        }
                    }
                }
                rows_loaded += chunk.len() as u64;
                chunk.clear();
                chunk_bytes = 0;

                if let (Some(pk_col), true) = (pk_col.as_deref(), plan.apply_deletes) {
                    if !seen_pk_buf.is_empty() {
                        db.insert_seen_pk_values(import_run_id, &insert.table, pk_col, &seen_pk_buf)
                            .await
                            .context("failed inserting seen_pk values")?;
                        seen_pk_buf.clear();
                    }
                }
            }
        }

        if !chunk.is_empty() {
            match plan.mode {
                IngestMode::Ingest => {
                    if plan.row_hash_enabled {
                        db.insert_rows_text_on_conflict_do_nothing(
                            &config.postgres.schema_libgen,
                            &pg_table,
                            &cols,
                            &["_bm_row_hash".to_string()],
                            &chunk,
                        )
                        .await
                        .with_context(|| format!("failed inserting (dedupe) into `{}`", pg_table))?;
                    } else {
                        match config.execution.loader.kind {
                            crate::config::LoaderKind::Copy => {
                            db.copy_rows_text_tsv(
                                &config.postgres.schema_libgen,
                                &pg_table,
                                &cols,
                                &chunk,
                            )
                            .await
                            .with_context(|| format!("failed COPY into `{}`", pg_table))?;
                            }
                            crate::config::LoaderKind::Insert => {
                            db.insert_rows_text(
                                &config.postgres.schema_libgen,
                                &pg_table,
                                &cols,
                                &chunk,
                            )
                            .await
                            .with_context(|| format!("failed inserting rows into `{}`", pg_table))?;
                            }
                        }
                    }
                }
                IngestMode::Update => {
                    if plan.row_hash_enabled {
                        db.insert_rows_text_on_conflict_do_nothing(
                            &config.postgres.schema_libgen,
                            &pg_table,
                            &cols,
                            &plan.conflict_columns,
                            &chunk,
                        )
                        .await
                        .with_context(|| format!("failed inserting (dedupe) into `{}`", pg_table))?;
                    } else {
                        db.upsert_rows_text(
                            &config.postgres.schema_libgen,
                            &pg_table,
                            &cols,
                            &plan.conflict_columns,
                            &chunk,
                        )
                        .await
                        .with_context(|| format!("failed upserting rows into `{}`", pg_table))?;
                    }
                }
            }
            rows_loaded += chunk.len() as u64;
        }

        if let (Some(pk_col), true) = (pk_col.as_deref(), plan.apply_deletes) {
            if !seen_pk_buf.is_empty() {
                db.insert_seen_pk_values(import_run_id, &insert.table, pk_col, &seen_pk_buf)
                    .await
                    .context("failed inserting seen_pk values")?;
                seen_pk_buf.clear();
            }
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
