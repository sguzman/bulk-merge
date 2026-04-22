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

    // Extracted fields
    pub title: Option<String>,
    pub name: Option<String>,
    pub author_keys: Vec<String>,
    pub isbn_10: Vec<String>,
    pub isbn_13: Vec<String>,
    pub publishers: Vec<String>,
    pub publish_date: Option<String>,
    pub subjects: Vec<String>,
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
        let naive = chrono::NaiveDateTime::parse_from_str(last_modified_str, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(last_modified_str, "%Y-%m-%dT%H:%M:%S")
            })
            .context("failed to parse last_modified")?;
        DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
    };

    let data: Value = serde_json::from_str(json_str).context("failed to parse JSON data")?;

    let title = data.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    let name = data.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let publish_date = data.get("publish_date").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut author_keys = Vec::new();
    if let Some(authors) = data.get("authors").and_then(|v| v.as_array()) {
        for a in authors {
            if let Some(key) = a.get("key").and_then(|v| v.as_str()) {
                author_keys.push(key.to_string());
            } else if let Some(author) = a.get("author") {
                if let Some(key) = author.get("key").and_then(|v| v.as_str()) {
                    author_keys.push(key.to_string());
                }
            }
        }
    }

    let mut isbn_10 = Vec::new();
    if let Some(isbns) = data.get("isbn_10").and_then(|v| v.as_array()) {
        for isbn in isbns {
            if let Some(s) = isbn.as_str() {
                isbn_10.push(s.to_string());
            }
        }
    }

    let mut isbn_13 = Vec::new();
    if let Some(isbns) = data.get("isbn_13").and_then(|v| v.as_array()) {
        for isbn in isbns {
            if let Some(s) = isbn.as_str() {
                isbn_13.push(s.to_string());
            }
        }
    }

    let mut publishers = Vec::new();
    if let Some(pubs) = data.get("publishers").and_then(|v| v.as_array()) {
        for p in pubs {
            if let Some(s) = p.as_str() {
                publishers.push(s.to_string());
            }
        }
    }

    let mut subjects = Vec::new();
    if let Some(subs) = data.get("subjects").and_then(|v| v.as_array()) {
        for s in subs {
            if let Some(val) = s.as_str() {
                subjects.push(val.to_string());
            }
        }
    }

    Ok(OlRecord {
        ol_type,
        ol_key,
        revision,
        last_modified,
        data,
        title,
        name,
        author_keys,
        isbn_10,
        isbn_13,
        publishers,
        publish_date,
        subjects,
    })
}
