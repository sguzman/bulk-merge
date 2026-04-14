use crate::config::LibgenDumpKind;
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub mysql_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, thiserror::Error)]
pub enum MySqlDumpError {
    #[error("statement exceeded max size ({max_bytes} bytes)")]
    StatementTooLarge { max_bytes: u64 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

/// Read SQL statements from a MySQL dump stream.
///
/// This is a minimal statement splitter that attempts to honor single-quoted
/// strings and backslash escapes when searching for `;`.
pub fn read_statements<R: Read>(
    mut reader: R,
    max_statement_bytes: u64,
) -> Result<Vec<String>, MySqlDumpError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let mut statements: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_single_quote = false;
    let mut prev_was_backslash = false;

    for ch in buf.chars() {
        cur.push(ch);

        if in_single_quote {
            if prev_was_backslash {
                prev_was_backslash = false;
                continue;
            }
            if ch == '\\' {
                prev_was_backslash = true;
                continue;
            }
            if ch == '\'' {
                in_single_quote = false;
            }
        } else {
            if ch == '\'' {
                in_single_quote = true;
            } else if ch == ';' {
                if cur.len() as u64 > max_statement_bytes {
                    return Err(MySqlDumpError::StatementTooLarge { max_bytes: max_statement_bytes });
                }
                statements.push(cur.trim().to_string());
                cur.clear();
            }
        }

        if cur.len() as u64 > max_statement_bytes {
            // fail fast before unbounded growth
            return Err(MySqlDumpError::StatementTooLarge { max_bytes: max_statement_bytes });
        }
    }

    Ok(statements)
}

pub fn parse_create_table(stmt: &str) -> Result<Option<TableDef>, MySqlDumpError> {
    let trimmed = strip_leading_comments(stmt).trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let upper = trimmed.to_ascii_uppercase();
    if !upper.starts_with("CREATE TABLE") {
        return Ok(None);
    }

    let table_name = extract_create_table_name(trimmed)
        .ok_or_else(|| MySqlDumpError::Parse("failed to parse CREATE TABLE name".to_string()))?;

    let cols = extract_columns(trimmed)?;

    Ok(Some(TableDef {
        name: table_name,
        columns: cols,
    }))
}

fn strip_leading_comments(input: &str) -> &str {
    let mut s = input;
    loop {
        let t = s.trim_start();
        if t.starts_with("--") {
            if let Some(pos) = t.find('\n') {
                s = &t[pos + 1..];
                continue;
            }
        }
        if t.starts_with("/*") {
            if let Some(pos) = t.find("*/") {
                s = &t[pos + 2..];
                continue;
            }
        }
        return t;
    }
}

fn extract_create_table_name(stmt: &str) -> Option<String> {
    // Very small subset:
    // CREATE TABLE `name` ( ...
    // CREATE TABLE IF NOT EXISTS `name` ( ...
    let mut rest = stmt.trim_start();
    rest = rest.strip_prefix("CREATE TABLE")?.trim_start();
    if rest.to_ascii_uppercase().starts_with("IF NOT EXISTS") {
        rest = rest[13..].trim_start();
    }

    let (name, _) = parse_identifier(rest)?;
    Some(name)
}

fn extract_columns(stmt: &str) -> Result<Vec<ColumnDef>, MySqlDumpError> {
    let start = stmt.find('(').ok_or_else(|| MySqlDumpError::Parse("missing '('".to_string()))?;
    let end = find_matching_paren(stmt, start)
        .ok_or_else(|| MySqlDumpError::Parse("missing ')'".to_string()))?;
    let inside = &stmt[start + 1..end];

    let parts = split_top_level_commas(inside);
    let mut cols: Vec<ColumnDef> = Vec::new();

    for part in parts {
        let item = part.trim();
        if item.is_empty() {
            continue;
        }
        let upper = item.to_ascii_uppercase();
        if upper.starts_with("PRIMARY KEY")
            || upper.starts_with("UNIQUE KEY")
            || upper.starts_with("KEY ")
            || upper.starts_with("CONSTRAINT")
            || upper.starts_with("FULLTEXT KEY")
        {
            continue;
        }

        let (name, rest) = match parse_identifier(item) {
            Some(v) => v,
            None => continue,
        };

        let mysql_type = rest
            .trim_start()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();

        if mysql_type.is_empty() {
            continue;
        }

        cols.push(ColumnDef { name, mysql_type });
    }

    if cols.is_empty() {
        return Err(MySqlDumpError::Parse("no columns parsed from CREATE TABLE".to_string()));
    }
    Ok(cols)
}

fn parse_identifier(input: &str) -> Option<(String, &str)> {
    let s = input.trim_start();
    if s.starts_with('`') {
        let end = s[1..].find('`')?;
        let name = s[1..1 + end].to_string();
        let rest = &s[1 + end + 1..];
        Some((name, rest))
    } else {
        let mut end = 0usize;
        for (idx, ch) in s.char_indices() {
            if ch.is_whitespace() || ch == '(' {
                break;
            }
            end = idx + ch.len_utf8();
        }
        if end == 0 {
            return None;
        }
        Some((s[..end].to_string(), &s[end..]))
    }
}

fn find_matching_paren(s: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_single = false;
    let mut prev_backslash = false;
    for (i, ch) in s.char_indices().skip(open_index) {
        if in_single {
            if prev_backslash {
                prev_backslash = false;
                continue;
            }
            if ch == '\\' {
                prev_backslash = true;
                continue;
            }
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        if ch == '\'' {
            in_single = true;
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_single = false;
    let mut prev_backslash = false;

    for ch in s.chars() {
        if in_single {
            cur.push(ch);
            if prev_backslash {
                prev_backslash = false;
                continue;
            }
            if ch == '\\' {
                prev_backslash = true;
                continue;
            }
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }

        match ch {
            '\'' => {
                in_single = true;
                cur.push(ch);
            }
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(cur);
                cur = String::new();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    parts
}

pub fn table_prefix_for_kind(
    kind: LibgenDumpKind,
    fiction_prefix: &str,
    compact_prefix: &str,
) -> String {
    match kind {
        LibgenDumpKind::Fiction => fiction_prefix.to_string(),
        LibgenDumpKind::Compact => compact_prefix.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_create_table_basic() {
        let stmt = r#"
CREATE TABLE `fiction` (
  `ID` int(11) NOT NULL,
  `Title` varchar(255) DEFAULT NULL,
  `Year` int(11) DEFAULT NULL,
  PRIMARY KEY (`ID`),
  KEY `Title` (`Title`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8;
"#;

        let def = parse_create_table(stmt).unwrap().unwrap();
        assert_eq!(def.name, "fiction");
        assert_eq!(
            def.columns,
            vec![
                ColumnDef {
                    name: "ID".to_string(),
                    mysql_type: "int(11)".to_string()
                },
                ColumnDef {
                    name: "Title".to_string(),
                    mysql_type: "varchar(255)".to_string()
                },
                ColumnDef {
                    name: "Year".to_string(),
                    mysql_type: "int(11)".to_string()
                }
            ]
        );
    }

    #[test]
    fn split_commas_respects_parens() {
        let inside = "`a` int(11), `b` decimal(10,2), PRIMARY KEY (`a`,`b`), `c` text";
        let parts = split_top_level_commas(inside);
        assert_eq!(parts.len(), 4);
    }
}
