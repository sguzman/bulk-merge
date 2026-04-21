use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug)]
pub struct OlRecord {
    pub ol_type: String,
    pub ol_key: String,
    pub revision: i32,
    pub last_modified: DateTime<Utc>,
    pub data: Value,
}

pub fn parse_line(line: &str) -> anyhow::Result<OlRecord> {
    let mut parts = line.splitn(5, '\t');
    let ol_type = parts.next().context("missing type")?.to_string();
    let ol_key = parts.next().context("missing key")?.to_string();
    let revision_str = parts.next().context("missing revision")?;
    let last_modified_str = parts.next().context("missing last_modified")?;
    let json_str = parts.next().context("missing json")?;

    let revision = revision_str
        .parse::<i32>()
        .context("failed to parse revision")?;

    let last_modified = if last_modified_str.ends_with('Z') {
        DateTime::parse_from_rfc3339(last_modified_str)?
            .with_timezone(&Utc)
    } else {
        // OpenLibrary timestamps often lack timezone but are UTC.
        // Try with and without microseconds.
        let naive = chrono::NaiveDateTime::parse_from_str(last_modified_str, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(last_modified_str, "%Y-%m-%dT%H:%M:%S")
            })
            .context("failed to parse last_modified")?;
        DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
    };

    let data = serde_json::from_str(json_str).context("failed to parse JSON data")?;

    Ok(OlRecord {
        ol_type,
        ol_key,
        revision,
        last_modified,
        data,
    })
}
