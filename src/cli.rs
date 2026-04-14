use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::{Db, ImportRunStatus};
use anyhow::Context as _;
use clap::{Parser, Subcommand, ValueEnum};
use tracing::{info, instrument};

#[derive(Parser, Debug, Clone)]
#[command(name = "bulk-merge", version, about = "Convert bibliographic metadata dumps into PostgreSQL")]
pub struct Args {
    /// Path to TOML config file
    #[arg(long)]
    pub config: Option<String>,

    /// Override default dry-run behavior for mutating commands
    #[arg(long)]
    pub dry_run: bool,

    /// Override logging level (e.g. trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,

    /// Override logging format (human|json)
    #[arg(long)]
    pub log_format: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Initialize database schemas and bm_meta bookkeeping tables
    InitDb,
    /// LibGen-specific operations (Phase 1)
    Libgen {
        #[command(subcommand)]
        command: LibgenCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum LibgenCommand {
    /// Ingest a LibGen dump (fiction or compact) into PostgreSQL
    Ingest {
        #[arg(long)]
        kind: LibgenDumpKindArg,
        #[arg(long)]
        dump: String,
        /// Stable dataset identifier for resumability/incrementals (defaults to config if present)
        #[arg(long)]
        dataset_id: Option<String>,
        /// Optional dataset version label for this import run
        #[arg(long)]
        dataset_version: Option<String>,
    },
    /// Update a LibGen table incrementally from a newer dump (Phase 1 target)
    Update {
        #[arg(long)]
        kind: LibgenDumpKindArg,
        #[arg(long)]
        dump: String,
        #[arg(long)]
        dataset_id: Option<String>,
        #[arg(long)]
        dataset_version: Option<String>,
    },
    /// Print basic stats about the LibGen schemas/tables (placeholder until ingestion exists)
    Stats,
    /// Print a small sample of rows (placeholder until ingestion exists)
    Sample {
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Validate minimal invariants (placeholder until ingestion exists)
    Validate,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum LibgenDumpKindArg {
    Fiction,
    Compact,
}

impl From<LibgenDumpKindArg> for LibgenDumpKind {
    fn from(value: LibgenDumpKindArg) -> Self {
        match value {
            LibgenDumpKindArg::Fiction => LibgenDumpKind::Fiction,
            LibgenDumpKindArg::Compact => LibgenDumpKind::Compact,
        }
    }
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

pub async fn run(args: Args, config: AppConfig) -> anyhow::Result<()> {
    let command = args.command.clone();
    match command {
        Command::InitDb => cmd_init_db(&config).await,
        Command::Libgen { command } => match command {
            LibgenCommand::Ingest {
                kind,
                dump,
                dataset_id,
                dataset_version,
            } => cmd_libgen_register_run(
                &args,
                &config,
                "libgen",
                ImportRunStatus::InProgress,
                kind.into(),
                dump,
                dataset_id,
                dataset_version,
                "ingest",
            )
            .await,
            LibgenCommand::Update {
                kind,
                dump,
                dataset_id,
                dataset_version,
            } => cmd_libgen_register_run(
                &args,
                &config,
                "libgen",
                ImportRunStatus::InProgress,
                kind.into(),
                dump,
                dataset_id,
                dataset_version,
                "update",
            )
            .await,
            LibgenCommand::Stats => cmd_libgen_placeholder(&config, "stats").await,
            LibgenCommand::Sample { limit } => cmd_libgen_sample_placeholder(&config, limit).await,
            LibgenCommand::Validate => cmd_libgen_placeholder(&config, "validate").await,
        },
    }
}

#[instrument(skip_all)]
async fn cmd_init_db(config: &AppConfig) -> anyhow::Result<()> {
    info!("connecting to postgres");
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!("database initialized");
    Ok(())
}

#[instrument(skip_all, fields(kind = ?kind, dump = %dump, op = %op))]
async fn cmd_libgen_register_run(
    args: &Args,
    config: &AppConfig,
    source_name: &str,
    status: ImportRunStatus,
    kind: LibgenDumpKind,
    dump: String,
    dataset_id: Option<String>,
    dataset_version: Option<String>,
    op: &str,
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
            source_name,
            &dataset_id,
            dataset_version.as_deref(),
            status,
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
async fn cmd_libgen_placeholder(config: &AppConfig, op: &str) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!(%op, "not implemented yet (Phase 1: LibGen ingestion in progress)");
    Ok(())
}

#[instrument(skip_all, fields(limit = limit))]
async fn cmd_libgen_sample_placeholder(config: &AppConfig, limit: u32) -> anyhow::Result<()> {
    let db = Db::connect(config).await?;
    db.migrate().await?;
    info!(limit, "not implemented yet (Phase 1: LibGen ingestion in progress)");
    Ok(())
}
