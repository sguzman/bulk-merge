use clap::{Parser, Subcommand, ValueEnum};

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
        /// Dump kind to use for table prefix lookup
        #[arg(long)]
        kind: LibgenDumpKindArg,
        /// MySQL table name (without prefixes), e.g. `fiction` or `libgen_compact`
        #[arg(long)]
        mysql_table: String,
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Validate minimal invariants (placeholder until ingestion exists)
    Validate {
        #[arg(long)]
        kind: LibgenDumpKindArg,
        #[arg(long)]
        mysql_table: String,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum LibgenDumpKindArg {
    Fiction,
    Compact,
}
