use crate::cli::Args;
use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::{Db, ImportRunStatus};
use crate::libgen::ingest::{ingest_dump_rows, IngestMode, IngestPlan};
use crate::libgen::provision::provision_tables_from_dump;
use anyhow::Context as _;
use tracing::{info, instrument};

#[instrument(skip_all, fields(op = %op, kind = ?kind, dump = %dump))]
pub async fn register_run(
    args: &Args,
    config: &AppConfig,
    op: &str,
    kind: LibgenDumpKind,
    dump: String,
    dataset_id: Option<String>,
    dataset_version: Option<String>,
) -> anyhow::Result<()> {
    if args.dry_run || config.execution.dry_run_default {
        info!(%op, "dry-run: would register import run");
        return Ok(());
    }

    let db = Db::connect(config).await?;
    db.migrate().await?;

    let dataset_id = dataset_id
        .or_else(|| config.libgen.dump.dataset_id.clone())
        .unwrap_or_else(|| format!("libgen-{kind:?}"));

    let run_id = db
        .create_import_run(
            "libgen",
            &dataset_id,
            dataset_version.as_deref(),
            ImportRunStatus::InProgress,
            kind,
            &dump,
            config,
        )
        .await
        .context("failed to create bm_meta.import_run")?;

    info!(import_run_id = run_id, %op, "registered import run");

    // Phase 1 slice: discover schema and provision dedicated tables per dump kind.
    let defs = provision_tables_from_dump(&db, config, kind, &dump, run_id)
        .await
        .context("failed to provision tables from dump schema")?;
    info!(import_run_id = run_id, tables = defs.len(), "provisioned tables");

    let kind_prefix = match kind {
        LibgenDumpKind::Fiction => config.libgen.tables.fiction_prefix.clone(),
        LibgenDumpKind::Compact => config.libgen.tables.compact_prefix.clone(),
    };
    let overall_prefix = config.postgres.table_prefix.clone().unwrap_or_default();

    let conflict_columns = match kind {
        LibgenDumpKind::Fiction => config.libgen.incremental.primary_key_columns.fiction.clone(),
        LibgenDumpKind::Compact => config.libgen.incremental.primary_key_columns.compact.clone(),
    };

    let mode = match op {
        "update" => IngestMode::Update,
        _ => IngestMode::Ingest,
    };

    if mode == IngestMode::Update {
        if conflict_columns.is_empty() {
            anyhow::bail!(
                "libgen update requires libgen.incremental.primary_key_columns.{:?} to be set",
                kind
            );
        }
        // Ensure uniqueness so ON CONFLICT is valid and fast.
        for def in &defs {
            let pg_table = format!("{overall_prefix}{kind_prefix}{}", def.name);
            if conflict_columns
                .iter()
                .all(|c| def.columns.iter().any(|col| col.name == *c))
            {
                db.ensure_unique_index(
                    &config.postgres.schema_libgen,
                    &pg_table,
                    &conflict_columns,
                    config.postgres.indexing.concurrent,
                )
                .await
                .with_context(|| format!("failed creating unique index for `{}`", pg_table))?;
            }
        }
    }
    let plan = IngestPlan {
        kind,
        dump_path: dump.clone(),
        table_defs: defs,
        overall_prefix,
        kind_prefix,
        mode,
        conflict_columns,
    };

    ingest_dump_rows(&db, config, &plan, run_id)
        .await
        .context("failed ingesting dump rows")?;

    if config.postgres.indexing.create_after_load {
        let main_fields = match kind {
            LibgenDumpKind::Fiction => &config.postgres.indexing.main_fields.fiction,
            LibgenDumpKind::Compact => &config.postgres.indexing.main_fields.compact,
        };

        if !main_fields.is_empty() {
            info!(fields = main_fields.len(), "creating post-load indexes");
            for def in &plan.table_defs {
                let pg_table = plan.pg_table_for_mysql(&def.name);
                for field in main_fields {
                    if def.columns.iter().any(|c| c.name == *field) {
                        db.ensure_btree_index(
                            &config.postgres.schema_libgen,
                            &pg_table,
                            field,
                            config.postgres.indexing.concurrent,
                        )
                        .await
                        .with_context(|| format!("failed creating index for `{}`.`{}`", pg_table, field))?;
                    }
                }
            }
        }
    }

    db.finish_import_run(run_id, ImportRunStatus::Succeeded)
        .await
        .context("failed to update import run status")?;

    Ok(())
}

#[instrument(skip_all)]
pub async fn stats(config: &AppConfig) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;

    let fiction_prefix = format!(
        "{}{}",
        config.postgres.table_prefix.as_deref().unwrap_or(""),
        config.libgen.tables.fiction_prefix
    );
    let compact_prefix = format!(
        "{}{}",
        config.postgres.table_prefix.as_deref().unwrap_or(""),
        config.libgen.tables.compact_prefix
    );

    let fiction_tables = db
        .list_tables_with_prefix(&config.postgres.schema_libgen, &fiction_prefix)
        .await
        .context("failed listing fiction tables")?;
    let compact_tables = db
        .list_tables_with_prefix(&config.postgres.schema_libgen, &compact_prefix)
        .await
        .context("failed listing compact tables")?;

    info!(
        fiction_tables = fiction_tables.len(),
        compact_tables = compact_tables.len(),
        "libgen table stats"
    );

    let runs = db.recent_import_runs("libgen", 5).await?;
    if runs.is_empty() {
        info!("no import runs found");
        return Ok(());
    }

    for (id, dataset_id, dataset_version, status, started_at) in runs {
        let raw_count = db.raw_statement_count(id).await.unwrap_or(0);
        info!(
            import_run_id = id,
            dataset_id = dataset_id,
            dataset_version = dataset_version.as_deref().unwrap_or(""),
            status = status,
            started_at = %started_at,
            raw_statements = raw_count,
            "recent import run"
        );
    }

    Ok(())
}

#[instrument(skip_all, fields(kind = ?kind, mysql_table = %mysql_table, limit = limit))]
pub async fn sample(
    config: &AppConfig,
    kind: LibgenDumpKind,
    mysql_table: &str,
    limit: u32,
) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;

    let kind_prefix = match kind {
        LibgenDumpKind::Fiction => &config.libgen.tables.fiction_prefix,
        LibgenDumpKind::Compact => &config.libgen.tables.compact_prefix,
    };
    let overall_prefix = config.postgres.table_prefix.as_deref().unwrap_or("");
    let table = format!("{overall_prefix}{kind_prefix}{mysql_table}");

    let rows = db
        .sample_table(&config.postgres.schema_libgen, &table, limit)
        .await
        .with_context(|| format!("failed sampling `{}`", table))?;

    if rows.is_empty() {
        info!(table = %table, "no rows");
        return Ok(());
    }

    // Phase 1: log sample row count; richer output formatting can follow.
    info!(table = %table, rows = rows.len(), "sampled rows");
    Ok(())
}

#[instrument(skip_all, fields(kind = ?kind, mysql_table = %mysql_table))]
pub async fn validate(config: &AppConfig, kind: LibgenDumpKind, mysql_table: &str) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;

    let kind_prefix = match kind {
        LibgenDumpKind::Fiction => &config.libgen.tables.fiction_prefix,
        LibgenDumpKind::Compact => &config.libgen.tables.compact_prefix,
    };
    let overall_prefix = config.postgres.table_prefix.as_deref().unwrap_or("");
    let table = format!("{overall_prefix}{kind_prefix}{mysql_table}");

    let count = db
        .table_row_count(&config.postgres.schema_libgen, &table)
        .await
        .with_context(|| format!("failed counting `{}`", table))?;

    if count == 0 {
        anyhow::bail!("validation failed: `{}` has 0 rows", table);
    }
    info!(table = %table, rows = count, "validation ok");
    Ok(())
}
