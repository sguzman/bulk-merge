use crate::cli::args::{Command, LibgenCommand};
use crate::cli::Args;
use crate::config::{AppConfig, LibgenDumpKind};
use tracing::instrument;

mod init_db;
mod libgen;

#[instrument(skip_all)]
pub async fn dispatch(args: Args, config: AppConfig) -> anyhow::Result<()> {
    let command = args.command.clone();
    match command {
        Command::InitDb => init_db::run(&config).await,
        Command::Libgen { command } => match command {
            LibgenCommand::Ingest {
                kind,
                dump,
                dataset_id,
                dataset_version,
            } => libgen::register_run(
                &args,
                &config,
                "ingest",
                kind.into(),
                dump,
                dataset_id,
                dataset_version,
            )
            .await,
            LibgenCommand::Update {
                kind,
                dump,
                dataset_id,
                dataset_version,
            } => libgen::register_run(
                &args,
                &config,
                "update",
                kind.into(),
                dump,
                dataset_id,
                dataset_version,
            )
            .await,
            LibgenCommand::Stats => libgen::stats(&config).await,
            LibgenCommand::Sample {
                kind,
                mysql_table,
                limit,
            } => libgen::sample(&config, kind.into(), &mysql_table, limit).await,
            LibgenCommand::Validate { kind, mysql_table } => {
                libgen::validate(&config, kind.into(), &mysql_table).await
            }
            LibgenCommand::Convert { kind, dump, out_dir } => {
                libgen::offline_convert(&args, &config, kind.into(), dump, out_dir).await
            }
            LibgenCommand::Load {
                in_dir,
                dataset_id,
                dataset_version,
            } => libgen::offline_load(&args, &config, in_dir, dataset_id, dataset_version).await,
            LibgenCommand::LoadStatus { import_run_id } => {
                libgen::offline_load_status(&config, import_run_id).await
            }
        },
    }
}

impl From<crate::cli::args::LibgenDumpKindArg> for LibgenDumpKind {
    fn from(value: crate::cli::args::LibgenDumpKindArg) -> Self {
        match value {
            crate::cli::args::LibgenDumpKindArg::Fiction => LibgenDumpKind::Fiction,
            crate::cli::args::LibgenDumpKindArg::Compact => LibgenDumpKind::Compact,
        }
    }
}
