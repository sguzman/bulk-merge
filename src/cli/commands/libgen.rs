use crate::cli::Args;
use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::{Db, ImportRunStatus};
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

    info!(import_run_id = run_id, %op, "registered import run (data load not implemented yet)");

    db.finish_import_run(run_id, ImportRunStatus::Pending)
        .await
        .context("failed to update import run status")?;

    Ok(())
}

#[instrument(skip_all, fields(op = %op))]
pub async fn placeholder(config: &AppConfig, op: &str) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!(%op, "not implemented yet (Phase 1: LibGen ingestion in progress)");
    Ok(())
}

#[instrument(skip_all, fields(limit = limit))]
pub async fn sample_placeholder(config: &AppConfig, limit: u32) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!(limit, "not implemented yet (Phase 1: LibGen ingestion in progress)");
    Ok(())
}

