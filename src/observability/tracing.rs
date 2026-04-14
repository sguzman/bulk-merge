use crate::cli::Args;
use crate::config::{AppConfig, LogFormat};
use tracing_subscriber::EnvFilter;

pub fn init_tracing(config: &AppConfig, args: &Args) -> anyhow::Result<()> {
    let level = args
        .log_level
        .clone()
        .unwrap_or_else(|| config.logging.level.clone());

    let format = match args.log_format.as_deref() {
        Some("human") => LogFormat::Human,
        Some("json") => LogFormat::Json,
        Some(_) => config.logging.format,
        None => config.logging.format,
    };

    let env_filter = EnvFilter::try_new(level)?;

    let builder = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(config.logging.include_target)
        .with_file(config.logging.include_location)
        .with_line_number(config.logging.include_location);

    match format {
        LogFormat::Human => builder.init(),
        LogFormat::Json => builder.json().init(),
    }

    Ok(())
}

