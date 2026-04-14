use anyhow::Context as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = bulk_merge::cli::Args::parse();

    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| "config/bulk-merge.toml".into());
    let config = bulk_merge::config::AppConfig::load(&config_path)
        .with_context(|| format!("failed to load config from `{config_path}`"))?;

    bulk_merge::logging::init_tracing(&config, &args)?;

    bulk_merge::cli::run(args, config).await
}

