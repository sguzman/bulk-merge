use crate::config::LibgenDumpKind;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub mysql_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, thiserror::Error)]
pub enum MySqlDumpError {
    #[error("statement exceeded max size (max={max_bytes} current={current_bytes} offset_end={offset_end})")]
    StatementTooLarge {
        max_bytes: u64,
        current_bytes: u64,
        offset_end: u64,
        preview: String,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

pub struct StatementReader<R> {
    reader: BufReader<R>,
    max_statement_bytes: u64,
    offset: u64,
    buf: Vec<u8>,
    in_single_quote: bool,
    prev_was_backslash: bool,
    scan_pos: usize,
}

impl<R: Read> StatementReader<R> {
    pub fn new(inner: R, max_statement_bytes: u64) -> Self {
        Self::new_with_offset(inner, max_statement_bytes, 0)
    }

    pub fn new_with_offset(inner: R, max_statement_bytes: u64, initial_offset: u64) -> Self {
        Self {
            reader: BufReader::new(inner),
            max_statement_bytes,
            offset: initial_offset,
            buf: Vec::with_capacity(1024 * 64),
            in_single_quote: false,
            prev_was_backslash: false,
            scan_pos: 0,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn next_statement(&mut self) -> Result<Option<String>, MySqlDumpError> {
        loop {
            if self.buf.len() as u64 > self.max_statement_bytes {
                return Err(MySqlDumpError::StatementTooLarge {
                    max_bytes: self.max_statement_bytes,
                    current_bytes: self.buf.len() as u64,
                    offset_end: self.offset,
                    preview: statement_preview_bytes(&self.buf, 256),
                });
            }

            if let Some(pos) = self.find_statement_terminator_pos() {
                let stmt_bytes: Vec<u8> = self.buf.drain(..=pos).collect();
                let stmt = String::from_utf8_lossy(&stmt_bytes).trim().to_string();
                // Reset quote scanning state after draining a full statement.
                self.in_single_quote = false;
                self.prev_was_backslash = false;
                self.scan_pos = 0;
                if stmt.is_empty() {
                    continue;
                }
                return Ok(Some(stmt));
            }

            let available = self.reader.fill_buf()?;
            if available.is_empty() {
                if self.buf.is_empty() {
                    return Ok(None);
                }
                let stmt = String::from_utf8_lossy(&self.buf).trim().to_string();
                self.buf.clear();
                self.in_single_quote = false;
                self.prev_was_backslash = false;
                self.scan_pos = 0;
                if stmt.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(stmt));
            }
            self.buf.extend_from_slice(available);
            let len = available.len();
            self.reader.consume(len);
            self.offset += len as u64;
        }
    }

    fn find_statement_terminator_pos(&mut self) -> Option<usize> {
        let mut idx: usize = self.scan_pos.min(self.buf.len());
        while idx < self.buf.len() {
            let b = self.buf[idx];
            if self.in_single_quote {
                if self.prev_was_backslash {
                    self.prev_was_backslash = false;
                    idx += 1;
                    continue;
                }
                if b == b'\\' {
                    if idx + 1 >= self.buf.len() {
                        // Need more bytes to determine escape target.
                        self.scan_pos = idx;
                        return None;
                    }
                    self.prev_was_backslash = true;
                    idx += 1;
                    continue;
                }
                if b == b'\'' {
                    // MySQL can escape quotes by doubling them ('').
                    if idx + 1 >= self.buf.len() {
                        // Need more bytes to determine whether this is an escaped quote.
                        self.scan_pos = idx;
                        return None;
                    }
                    if idx + 1 < self.buf.len() && self.buf[idx + 1] == b'\'' {
                        idx += 2;
                        continue;
                    }
                    self.in_single_quote = false;
                    idx += 1;
                    continue;
                }
                idx += 1;
                continue;
            }

            if b == b'\'' {
                self.in_single_quote = true;
                idx += 1;
                continue;
            }
            if b == b';' {
                self.scan_pos = idx;
                return Some(idx);
            }
            idx += 1;
        }
        self.scan_pos = idx;
        None
    }
}

pub fn statement_preview(stmt: &str, max_bytes: usize) -> String {
    statement_preview_bytes(stmt.as_bytes(), max_bytes)
}

fn statement_preview_bytes(bytes: &[u8], max_bytes: usize) -> String {
    let n = bytes.len().min(max_bytes.max(1));
    let s = String::from_utf8_lossy(&bytes[..n]).to_string();
    if bytes.len() > n {
        format!("{s}…")
    } else {
        s
    }
}

pub fn seek_to_offset(file: &mut std::fs::File, offset: u64) -> Result<(), MySqlDumpError> {
    file.seek(SeekFrom::Start(offset))?;
    Ok(())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Null,
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertStatement {
    pub table: String,
    pub rows: Vec<Vec<Value>>,
}

pub fn parse_insert_into_values_input<'a>(
    stmt: &'a str,
) -> Result<Option<(String, &'a str)>, MySqlDumpError> {
    let trimmed = strip_leading_comments(stmt).trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let upper = trimmed.to_ascii_uppercase();
    if !upper.starts_with("INSERT INTO") {
        return Ok(None);
    }

    let mut rest = trimmed.trim_start();
    rest = rest[11..].trim_start(); // len("INSERT INTO") == 11
    let (table, after_table) = parse_identifier(rest)
        .ok_or_else(|| MySqlDumpError::Parse("failed to parse INSERT INTO table name".to_string()))?;

    let upper_after = after_table.to_ascii_uppercase();
    let values_pos = upper_after.find("VALUES").ok_or_else(|| {
        MySqlDumpError::Parse("INSERT INTO without VALUES is not supported".to_string())
    })?;
    let after_values = after_table[values_pos + 6..].trim_start();
    Ok(Some((table, after_values)))
}

pub fn parse_insert_into(stmt: &str) -> Result<Option<InsertStatement>, MySqlDumpError> {
    let trimmed = strip_leading_comments(stmt).trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let upper = trimmed.to_ascii_uppercase();
    if !upper.starts_with("INSERT INTO") {
        return Ok(None);
    }

    // Expected minimal subset:
    // INSERT INTO `table` VALUES (...),(...);
    // We ignore any explicit column list for now.
    let Some((table, mut after_values)) = parse_insert_into_values_input(trimmed)? else {
        return Ok(None);
    };

    // Parse a comma-separated list of parenthesized rows.
    let mut rows: Vec<Vec<Value>> = Vec::new();
    while !after_values.is_empty() {
        after_values = after_values.trim_start();
        if after_values.starts_with(';') {
            break;
        }
        if after_values.starts_with(',') {
            after_values = after_values[1..].trim_start();
            continue;
        }
        if !after_values.starts_with('(') {
            break;
        }

        let (row, rest_row) = parse_row(after_values)?;
        rows.push(row);
        after_values = rest_row.trim_start();
        if after_values.starts_with(';') {
            break;
        }
    }

    if rows.is_empty() {
        return Err(MySqlDumpError::Parse(
            "INSERT INTO parsed but yielded zero rows".to_string(),
        ));
    }

    Ok(Some(InsertStatement { table, rows }))
}

pub fn parse_insert_into_for_each_row<F>(
    stmt: &str,
    mut on_row: F,
) -> Result<Option<(String, u64)>, MySqlDumpError>
where
    F: FnMut(Vec<Value>) -> Result<(), MySqlDumpError>,
{
    let Some((table, mut after_values)) = parse_insert_into_values_input(stmt)? else {
        return Ok(None);
    };

    let mut rows_seen: u64 = 0;
    while !after_values.is_empty() {
        after_values = after_values.trim_start();
        if after_values.starts_with(';') {
            break;
        }
        if after_values.starts_with(',') {
            after_values = after_values[1..].trim_start();
            continue;
        }
        if !after_values.starts_with('(') {
            break;
        }

        let (row, rest_row) = parse_row(after_values)?;
        on_row(row)?;
        rows_seen += 1;
        after_values = rest_row.trim_start();
        if after_values.starts_with(';') {
            break;
        }
    }

    if rows_seen == 0 {
        return Err(MySqlDumpError::Parse(
            "INSERT INTO parsed but yielded zero rows".to_string(),
        ));
    }

    Ok(Some((table, rows_seen)))
}

pub(crate) fn parse_row(input: &str) -> Result<(Vec<Value>, &str), MySqlDumpError> {
    let s = input.trim_start();
    if !s.starts_with('(') {
        return Err(MySqlDumpError::Parse("expected '('".to_string()));
    }

    let mut idx = 1usize;
    let bytes = s.as_bytes();
    let mut values: Vec<Value> = Vec::new();
    loop {
        idx = skip_ws(bytes, idx);
        if idx >= bytes.len() {
            return Err(MySqlDumpError::Parse("unterminated row".to_string()));
        }
        if bytes[idx] == b')' {
            idx += 1;
            break;
        }

        let (val, next) = parse_value(s, idx)?;
        values.push(val);
        idx = skip_ws(bytes, next);
        if idx >= bytes.len() {
            return Err(MySqlDumpError::Parse("unterminated row".to_string()));
        }
        if bytes[idx] == b',' {
            idx += 1;
            continue;
        }
        if bytes[idx] == b')' {
            idx += 1;
            break;
        }
        return Err(MySqlDumpError::Parse("unexpected token in row".to_string()));
    }

    Ok((values, &s[idx..]))
}

fn parse_value<'a>(s: &'a str, start: usize) -> Result<(Value, usize), MySqlDumpError> {
    let bytes = s.as_bytes();
    if start >= bytes.len() {
        return Err(MySqlDumpError::Parse("unexpected end".to_string()));
    }

    match bytes[start] {
        b'\'' => {
            let mut out: Vec<u8> = Vec::new();
            let mut i = start + 1;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b'\\' {
                    if i + 1 >= bytes.len() {
                        return Err(MySqlDumpError::Parse("dangling escape".to_string()));
                    }
                    let esc = bytes[i + 1];
                    match esc {
                        b'0' => out.push(0),
                        b'b' => out.push(8),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'Z' => out.push(0x1a),
                        b'\\' => out.push(b'\\'),
                        b'\'' => out.push(b'\''),
                        b'"' => out.push(b'"'),
                        other => out.push(other),
                    }
                    i += 2;
                    continue;
                }
                if b == b'\'' {
                    // MySQL can escape single quotes by doubling them ('').
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                        out.push(b'\'');
                        i += 2;
                        continue;
                    }
                    let text = String::from_utf8_lossy(&out).to_string();
                    return Ok((Value::Text(text), i + 1));
                }
                out.push(b);
                i += 1;
            }
            Err(MySqlDumpError::Parse("unterminated string".to_string()))
        }
        b'\\' => {
            let rem = &s[start..];
            if rem.starts_with("\\N") {
                Ok((Value::Null, start + 2))
            } else {
                let tok = read_token(s, start);
                Ok((Value::Text(tok.clone()), start + tok.len()))
            }
        }
        b'N' | b'n' => {
            let rem = &s[start..];
            if rem.to_ascii_uppercase().starts_with("NULL") {
                Ok((Value::Null, start + 4))
            } else {
                Ok((Value::Text(read_token(s, start)), start + read_token(s, start).len()))
            }
        }
        _ => {
            let tok = read_token(s, start);
            Ok((Value::Text(tok.clone()), start + tok.len()))
        }
    }
}

fn read_token(s: &str, start: usize) -> String {
    let bytes = s.as_bytes();
    let mut end = start;
    while end < bytes.len() {
        let b = bytes[end];
        if b == b',' || b == b')' || b.is_ascii_whitespace() {
            break;
        }
        end += 1;
    }
    s[start..end].to_string()
}

fn skip_ws(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
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

pub fn compute_pg_table_name(
    overall_prefix: &str,
    kind_prefix: &str,
    kind: LibgenDumpKind,
    mysql_table: &str,
) -> String {
    let kind_str = kind.to_string().to_lowercase();
    if mysql_table == kind_str || mysql_table.starts_with(&format!("{}_", kind_str)) {
        format!("{}{}{}", overall_prefix, kind_prefix, mysql_table)
    } else {
        format!("{}{}{}_{}", overall_prefix, kind_prefix, kind_str, mysql_table)
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

    #[test]
    fn parse_insert_into_basic() {
        let stmt = "INSERT INTO `fiction` VALUES (1,'hi',NULL),(2,'a\\'b','x');";
        let ins = parse_insert_into(stmt).unwrap().unwrap();
        assert_eq!(ins.table, "fiction");
        assert_eq!(ins.rows.len(), 2);
        assert_eq!(ins.rows[0][0], Value::Text("1".to_string()));
        assert_eq!(ins.rows[0][1], Value::Text("hi".to_string()));
        assert_eq!(ins.rows[0][2], Value::Null);
        assert_eq!(ins.rows[1][1], Value::Text("a'b".to_string()));
    }

    #[test]
    fn statement_reader_emits_statements() {
        let input = b"CREATE TABLE `t` (`a` int);\nINSERT INTO `t` VALUES (1,'x');\n";
        let mut r = StatementReader::new(&input[..], 10_000);
        let s1 = r.next_statement().unwrap().unwrap();
        assert!(s1.to_ascii_uppercase().starts_with("CREATE TABLE"));
        let s2 = r.next_statement().unwrap().unwrap();
        assert!(s2.to_ascii_uppercase().starts_with("INSERT INTO"));
        let s3 = r.next_statement().unwrap();
        assert!(s3.is_none());
    }

    #[test]
    fn statement_reader_enforces_max_size_with_context() {
        let input = b"INSERT INTO `t` VALUES (1,'abcdefghijklmnopqrstuvwxyz');\n";
        let mut r = StatementReader::new(&input[..], 10);
        let err = r.next_statement().expect_err("expected too-large error");
        match err {
            MySqlDumpError::StatementTooLarge {
                max_bytes,
                current_bytes,
                offset_end,
                preview,
            } => {
                assert_eq!(max_bytes, 10);
                assert!(current_bytes > 10);
                assert!(offset_end > 0);
                assert!(!preview.is_empty());
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
