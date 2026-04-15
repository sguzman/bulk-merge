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
            // Accept common MySQL formats; let Postgres parse it.
            if trimmed.len() >= 10 {
                Some(trimmed.to_string())
            } else {
                None
            }
        }
        PgTargetType::Date => {
            if trimmed.len() >= 8 {
                Some(trimmed.to_string())
            } else {
                None
            }
        }
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
