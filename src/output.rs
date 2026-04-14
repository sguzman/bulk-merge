use crate::config::{AppConfig, OutputFormat};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::info;

pub fn maybe_write_report_line<T: Serialize>(
    config: &AppConfig,
    kind: &str,
    payload: &T,
) -> anyhow::Result<()> {
    let Some(path) = &config.output.report_path else {
        return Ok(());
    };

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::json!({
        "kind": kind,
        "payload": payload,
    });
    writeln!(file, "{}", line)?;
    info!(report_path = %path, kind, "wrote report line");
    Ok(())
}

pub fn output_format(config: &AppConfig) -> OutputFormat {
    config.output.format
}

