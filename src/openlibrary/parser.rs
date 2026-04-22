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
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub birth_date: Option<String>,
    pub death_date: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub author_keys: Vec<String>,
    pub isbn_10: Vec<String>,
    pub isbn_13: Vec<String>,
    pub publishers: Vec<String>,
    pub publish_date: Option<String>,
    pub subjects: Vec<String>,
    pub subject_people: Vec<String>,
    pub subject_places: Vec<String>,
    pub subject_times: Vec<String>,
    pub covers: Vec<i64>,
    pub number_of_pages: Option<i32>,
    pub physical_format: Option<String>,
    pub languages: Vec<String>,
    pub lc_classifications: Vec<String>,
    pub dewey_decimal_class: Vec<String>,
    pub notes: Option<String>,
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

    let title = extract_text(&data, "title");
    let subtitle = extract_text(&data, "subtitle");
    let description = extract_text(&data, "description");
    let name = extract_text(&data, "name");
    let birth_date = extract_text(&data, "birth_date");
    let death_date = extract_text(&data, "death_date");
    let bio = extract_text(&data, "bio");
    let website = extract_text(&data, "website");
    let publish_date = extract_text(&data, "publish_date");
    let notes = extract_text(&data, "notes");

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

    let isbn_10 = extract_array(&data, "isbn_10");
    let isbn_13 = extract_array(&data, "isbn_13");
    let publishers = extract_array(&data, "publishers");
    let subjects = extract_array(&data, "subjects");
    let subject_people = extract_array(&data, "subject_people");
    let subject_places = extract_array(&data, "subject_places");
    let subject_times = extract_array(&data, "subject_times");

    let mut covers = Vec::new();
    if let Some(arr) = data.get("covers").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(i) = v.as_i64() {
                covers.push(i);
            }
        }
    }

    let number_of_pages = data.get("number_of_pages").and_then(|v| v.as_i64()).map(|i| i as i32);
    let physical_format = extract_text(&data, "physical_format");

    let mut languages = Vec::new();
    if let Some(arr) = data.get("languages").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(key) = v.get("key").and_then(|v| v.as_str()) {
                languages.push(key.to_string());
            }
        }
    }

    let lc_classifications = extract_array(&data, "lc_classifications");
    let dewey_decimal_class = extract_array(&data, "dewey_decimal_class");

    Ok(OlRecord {
        ol_type,
        ol_key,
        revision,
        last_modified,
        data,
        title,
        subtitle,
        description,
        name,
        birth_date,
        death_date,
        bio,
        website,
        author_keys,
        isbn_10,
        isbn_13,
        publishers,
        publish_date,
        subjects,
        subject_people,
        subject_places,
        subject_times,
        covers,
        number_of_pages,
        physical_format,
        languages,
        lc_classifications,
        dewey_decimal_class,
        notes,
    })
}

fn extract_text(data: &Value, field: &str) -> Option<String> {
    data.get(field).and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string())
        } else if let Some(obj) = v.as_object() {
            obj.get("value").and_then(|vv| vv.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    })
}

fn extract_array(data: &Value, field: &str) -> Vec<String> {
    let mut res = Vec::new();
    if let Some(arr) = data.get(field).and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                res.push(s.to_string());
            }
        }
    }
    res
}
