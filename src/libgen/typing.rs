use crate::config::LibgenUnrepresentablePolicy;
use crate::db::PgTargetType;

pub(crate) fn coerce_value_best_effort(
    ty: PgTargetType,
    s: String,
    policy: LibgenUnrepresentablePolicy,
) -> anyhow::Result<Option<String>> {
    let original = s.clone();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if ty != PgTargetType::Text && (trimmed == "\\N" || trimmed == "N" || trimmed == "NULL" || trimmed == "\\\\N") {
        return Ok(None);
    }
    // For date/timestamp columns, any backslash-containing value or known MySQL "zero" dates are treated as NULL.
    if matches!(ty, PgTargetType::Timestamp | PgTargetType::Date | PgTargetType::Timestamptz)
        && (trimmed == "N" || trimmed == "\\N" || trimmed == "0000-00-00 00:00:00" || trimmed == "0000-00-00" || trimmed.contains('\\'))
    {
        return Ok(None);
    }

    let coerced = match ty {
        PgTargetType::Text => Some(s),
        PgTargetType::Int4 => trimmed.parse::<i32>().ok().map(|v| v.to_string()),
        PgTargetType::Int8 => trimmed.parse::<i64>().ok().map(|v| v.to_string()),
        PgTargetType::Numeric => {
            // Phase 1: validate that it parses as a number; keep original string to avoid
            // precision changes.
            trimmed.parse::<f64>().ok().map(|_| trimmed.to_string())
        }
        PgTargetType::Float8 => trimmed.parse::<f64>().ok().map(|v| v.to_string()),
        PgTargetType::Bool => {
            let u = trimmed.to_ascii_lowercase();
            match u.as_str() {
                "1" | "true" | "t" | "yes" | "y" => Some("true".to_string()),
                "0" | "false" | "f" | "no" | "n" => Some("false".to_string()),
                _ => None,
            }
        }
        PgTargetType::Timestamp => {
            // Validate common MySQL formats and reject invalid "zero" dates like
            // 0000-00-00 00:00:00 (Postgres can't represent them).
            if trimmed == "0000-00-00 00:00:00" || trimmed == "0000-00-00" {
                None
            } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S")
            {
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            } else if let Ok(dt) =
                chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S%.f")
            {
                Some(dt.format("%Y-%m-%d %H:%M:%S%.f").to_string())
            } else if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
                Some(d.and_hms_opt(0, 0, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            }
        }
        PgTargetType::Date => {
            if trimmed == "0000-00-00" {
                None
            } else if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
                Some(d.format("%Y-%m-%d").to_string())
            } else {
                None
            }
        }
        PgTargetType::Timestamptz => Some(s),
        PgTargetType::Jsonb => Some(s),
        PgTargetType::TextArray => Some(s),
        PgTargetType::Int8Array => Some(s),
    };

    match (coerced, policy) {
        (Some(v), _) => Ok(Some(v)),
        (None, LibgenUnrepresentablePolicy::Null) => Ok(None),
        (None, LibgenUnrepresentablePolicy::Text) => Ok(Some(original)),
        (None, LibgenUnrepresentablePolicy::Error) => {
            anyhow::bail!("value `{}` is not representable as {:?}", original, ty)
        }
    }
}
