use crate::config::{AppConfig, LibgenDumpKind, LibgenOfflineLoadStrategy};
use crate::db::Db;
use crate::libgen::mysql_dump::{
    parse_create_table, parse_insert_into, seek_to_offset, statement_preview, StatementReader, TableDef, Value,
};
use crate::progress::{ProgressConfig, ProgressTicker};
use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineManifest {
    pub kind: String,
    pub dump_path: String,
    pub schema: String,
    pub overall_prefix: String,
    pub kind_prefix: String,
    pub tables: Vec<TableDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineState {
    pub dump_offset: u64,
}

fn manifest_path(out_dir: &Path) -> PathBuf {
    out_dir.join("manifest.json")
}

fn state_path(out_dir: &Path) -> PathBuf {
    out_dir.join("state.json")
}

fn table_file_path(out_dir: &Path, mysql_table: &str) -> PathBuf {
    out_dir.join(format!("{mysql_table}.tsv"))
}

fn normalize_kind(kind: LibgenDumpKind) -> &'static str {
    match kind {
        LibgenDumpKind::Fiction => "fiction",
        LibgenDumpKind::Compact => "compact",
    }
}

#[instrument(skip_all, fields(kind = ?kind, dump = %dump_path, out_dir = %out_dir))]
pub fn convert_dump_to_tsv(
    config: &AppConfig,
    kind: LibgenDumpKind,
    dump_path: &str,
    out_dir: &str,
) -> anyhow::Result<OfflineManifest> {
    let out_dir = Path::new(out_dir);
    create_dir_all(out_dir).with_context(|| format!("failed to create out dir `{}`", out_dir.display()))?;

    // Resume if state exists.
    let state_path = state_path(out_dir);
    let mut state: OfflineState = if state_path.exists() {
        let bytes = std::fs::read(&state_path)?;
        serde_json::from_slice(&bytes).context("failed to parse offline state")?
    } else {
        OfflineState { dump_offset: 0 }
    };

    let mut file = File::open(dump_path).with_context(|| format!("failed to open dump `{dump_path}`"))?;
    let total_bytes = file.metadata().ok().map(|m| m.len());
    if state.dump_offset > 0 {
        seek_to_offset(&mut file, state.dump_offset)?;
        info!(offset = state.dump_offset, "resuming conversion from offset");
    }

    let mut stmt_reader =
        StatementReader::new_with_offset(file, config.libgen.dump.max_statement_bytes, state.dump_offset);

    let kind_prefix = match kind {
        LibgenDumpKind::Fiction => config.libgen.tables.fiction_prefix.clone(),
        LibgenDumpKind::Compact => config.libgen.tables.compact_prefix.clone(),
    };
    let overall_prefix = config.postgres.table_prefix.clone().unwrap_or_default();

    let mut table_defs: BTreeMap<String, TableDef> = BTreeMap::new();
    let mut writers: BTreeMap<String, BufWriter<File>> = BTreeMap::new();

    let mut ticker = ProgressTicker::new(ProgressConfig {
        log_interval: Duration::from_secs(config.progress.log_interval_seconds),
    });

    while let Some(stmt) = stmt_reader.next_statement()? {
        let off = stmt_reader.offset();
        ticker.maybe_log(
            "libgen_offline_convert",
            off,
            total_bytes,
            || {
                info!(
                    offset = off,
                    tables_discovered = table_defs.len(),
                    tables_written = writers.len(),
                    "progress_detail"
                );
            },
        );

        if let Some(def) = parse_create_table(&stmt).with_context(|| {
            let preview = statement_preview(&stmt, config.libgen.dump.error_preview_bytes as usize);
            format!("failed parsing CREATE TABLE at offset_end={off}: {preview}")
        })? {
            table_defs.entry(def.name.clone()).or_insert(def);
            state.dump_offset = off;
            std::fs::write(&state_path, serde_json::to_vec_pretty(&state)?)?;
            continue;
        }

        let Some(insert) = parse_insert_into(&stmt).with_context(|| {
            let preview = statement_preview(&stmt, config.libgen.dump.error_preview_bytes as usize);
            format!("failed parsing INSERT INTO at offset_end={off}: {preview}")
        })? else {
            state.dump_offset = off;
            std::fs::write(&state_path, serde_json::to_vec_pretty(&state)?)?;
            continue;
        };

        // Ensure a writer exists for the table.
        let w = writers.entry(insert.table.clone()).or_insert_with(|| {
            let path = table_file_path(out_dir, &insert.table);
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .expect("open table tsv");
            BufWriter::new(f)
        });

        // Emit TSV rows in a COPY-friendly format:
        // format csv, delimiter \t, null \N, quote ", escape "
        for row in insert.rows {
            write_row_tsv(config, w, &row)?;
        }
        w.flush()?;

        state.dump_offset = off;
        std::fs::write(&state_path, serde_json::to_vec_pretty(&state)?)?;
    }

    let manifest = OfflineManifest {
        kind: normalize_kind(kind).to_string(),
        dump_path: dump_path.to_string(),
        schema: config.postgres.schema_libgen.clone(),
        overall_prefix,
        kind_prefix,
        tables: table_defs.into_values().collect(),
    };
    std::fs::write(
        manifest_path(out_dir),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

fn write_row_tsv<W: Write>(config: &AppConfig, w: &mut W, row: &[Value]) -> anyhow::Result<()> {
    for (i, v) in row.iter().enumerate() {
        if i > 0 {
            w.write_all(b"\t")?;
        }
        match v {
            Value::Null => w.write_all(b"\\N")?,
            Value::Text(s) => {
                let s = sanitize_text(config, s.clone());
                w.write_all(b"\"")?;
                // Escape by doubling quotes.
                for ch in s.chars() {
                    if ch == '"' {
                        w.write_all(b"\"\"")?;
                    } else {
                        let mut tmp = [0u8; 4];
                        w.write_all(ch.encode_utf8(&mut tmp).as_bytes())?;
                    }
                }
                w.write_all(b"\"")?;
            }
        }
    }
    w.write_all(b"\n")?;
    Ok(())
}

#[instrument(skip_all, fields(in_dir = %in_dir))]
pub async fn load_tsv_into_postgres(
    db: &Db,
    config: &AppConfig,
    in_dir: &str,
    import_run_id: i64,
) -> anyhow::Result<()> {
    load_tsv_into_postgres_inner(db, config, in_dir, import_run_id, None).await
}

async fn load_tsv_into_postgres_inner(
    db: &Db,
    config: &AppConfig,
    in_dir: &str,
    import_run_id: i64,
    interrupt_after_staged_tables: Option<usize>,
) -> anyhow::Result<()> {
    let in_dir = Path::new(in_dir);
    let bytes = std::fs::read(manifest_path(in_dir)).context("missing manifest.json")?;
    let manifest: OfflineManifest = serde_json::from_slice(&bytes).context("failed parsing manifest.json")?;

    match config.libgen.offline.load.strategy {
        LibgenOfflineLoadStrategy::StagingSwap => {
            load_tsv_staging_swap(
                db,
                config,
                in_dir,
                &manifest,
                import_run_id,
                interrupt_after_staged_tables,
            )
            .await
        }
    }
}

async fn load_tsv_staging_swap(
    db: &Db,
    config: &AppConfig,
    in_dir: &Path,
    manifest: &OfflineManifest,
    import_run_id: i64,
    interrupt_after_staged_tables: Option<usize>,
) -> anyhow::Result<()> {
    let schema_live = &manifest.schema;
    let schema_staging = format!(
        "{}_{}",
        config.libgen.offline.load.staging_schema_prefix,
        import_run_id
    );
    db.ensure_schema(&schema_staging).await?;

    let mut ticker = ProgressTicker::new(ProgressConfig {
        log_interval: Duration::from_secs(config.progress.log_interval_seconds),
    });

    let mut staged_tables: usize = 0;
    for (i, def) in manifest.tables.iter().enumerate() {
        let pg_table = format!("{}{}{}", manifest.overall_prefix, manifest.kind_prefix, def.name);
        let cols: Vec<String> = def.columns.iter().map(|c| c.name.clone()).collect();
        let tsv_path = table_file_path(in_dir, &def.name);
        if !tsv_path.exists() {
            continue;
        }

        let stage = db
            .get_offline_swap_stage(import_run_id, &pg_table)
            .await
            .unwrap_or(None);

        // Stage (create + load + index) unless already staged/swapped.
        if stage.as_deref() != Some("staged") && stage.as_deref() != Some("swapped") {
            info!(table = %pg_table, schema = %schema_staging, "staging_table");
            db.create_table_from_def(&schema_staging, &pg_table, def, false)
                .await
                .with_context(|| format!("failed creating staging table `{}`", pg_table))?;

            let total = std::fs::metadata(&tsv_path).ok().map(|m| m.len());
            ticker.maybe_log("libgen_offline_stage", i as u64, Some(manifest.tables.len() as u64), || {
                info!(table = %pg_table, file_bytes = total.unwrap_or(0), "staging_copy");
            });

        db.copy_in_tsv_file(
            &schema_staging,
            &pg_table,
            &cols,
            &tsv_path,
            config.execution.copy.file_send_chunk_bytes,
            config.libgen.dump.sanitize_nul_bytes,
            config.libgen.dump.nul_replacement.as_bytes(),
        )
        .await
        .with_context(|| format!("failed staging COPY for `{}`", pg_table))?;

            if config.postgres.indexing.create_after_load {
                let main_fields = if manifest.kind == "fiction" {
                    &config.postgres.indexing.main_fields.fiction
                } else {
                    &config.postgres.indexing.main_fields.compact
                };
                for field in main_fields {
                    if def.columns.iter().any(|c| c.name == *field) {
                        db.ensure_btree_index(
                            &schema_staging,
                            &pg_table,
                            field,
                            config.postgres.indexing.concurrent,
                        )
                        .await?;
                    }
                }
            }

            db.upsert_offline_swap_progress(
                import_run_id,
                schema_live,
                &schema_staging,
                &pg_table,
                "staged",
            )
            .await?;

            staged_tables += 1;
            if let Some(limit) = interrupt_after_staged_tables {
                if staged_tables >= limit {
                    anyhow::bail!("intentional interrupt after staging {staged_tables} table(s)");
                }
            }
        }

        // Swap staged table into place unless already swapped.
        let stage = db
            .get_offline_swap_stage(import_run_id, &pg_table)
            .await
            .unwrap_or(None);
        if stage.as_deref() != Some("swapped") {
            info!(table = %pg_table, "swapping_staged_table_into_live_schema");
            db.swap_table_from_staging(
                schema_live,
                &schema_staging,
                &pg_table,
                config.libgen.offline.load.keep_old_tables,
                &import_run_id.to_string(),
            )
            .await
            .with_context(|| format!("failed swapping `{}` from staging", pg_table))?;

            db.upsert_offline_swap_progress(
                import_run_id,
                schema_live,
                &schema_staging,
                &pg_table,
                "swapped",
            )
            .await?;
        }

        let total = std::fs::metadata(&tsv_path).ok().map(|m| m.len());
        ticker.maybe_log("libgen_offline_load", i as u64, Some(manifest.tables.len() as u64), || {
            info!(table = %pg_table, file_bytes = total.unwrap_or(0), "loaded_table");
        });
    }

    if config.libgen.offline.load.drop_old_tables_on_success && config.libgen.offline.load.keep_old_tables {
        for def in &manifest.tables {
            let pg_table = format!("{}{}{}", manifest.overall_prefix, manifest.kind_prefix, def.name);
            let old_name = format!("{pg_table}__old_{import_run_id}");
            db.drop_table_if_exists(schema_live, &old_name).await?;
        }
    }

    if config.libgen.offline.load.drop_staging_schema_on_success {
        db.drop_schema_if_exists_cascade(&schema_staging).await?;
    }

    Ok(())
}

pub async fn load_tsv_into_postgres_for_test_interrupt_after_staged_tables(
    db: &Db,
    config: &AppConfig,
    in_dir: &str,
    import_run_id: i64,
    interrupt_after_staged_tables: usize,
) -> anyhow::Result<()> {
    load_tsv_into_postgres_inner(
        db,
        config,
        in_dir,
        import_run_id,
        Some(interrupt_after_staged_tables),
    )
    .await
}
