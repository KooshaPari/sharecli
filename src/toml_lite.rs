// Minimal TOML parser. Supports tables, dotted keys, quoted/bare strings, integers,
// floats, booleans, dates, arrays, and inline tables. Output is a flat
// Vec<(KeyPath, Value)> where KeyPath is the dotted path. Use `group_by_table` to
// pivot into a BTreeMap<String, BTreeMap<String, Value>> for ergonomic access.
//
// This is NOT a full TOML implementation (no escape characters beyond \" and \\,
// no multi-line strings, no array-of-tables [[]]). It is enough for round-tripping
// app config files.
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Date(String),
    Array(Vec<Value>),
    InlineTable(BTreeMap<String, Value>),
}

pub type KeyPath = Vec<String>;

#[derive(Debug, PartialEq, Clone)]
pub struct Entry { pub key: KeyPath, pub value: Value }

pub fn parse(input: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    let mut current_table: Vec<String> = Vec::new();
    for (i, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if name.starts_with('[') {
                return Err(format!("line {}: array-of-tables not supported", i + 1));
            }
            current_table = name.split('.').map(|s| s.trim().to_string()).collect();
            if current_table.iter().any(|s| s.is_empty()) {
                return Err(format!("line {}: empty table segment", i + 1));
            }
            continue;
        }
        let eq = line.find('=').ok_or_else(|| format!("line {}: missing '='", i + 1))?;
        let key_str = line[..eq].trim();
        let value_str = line[eq+1..].trim();
        let key: KeyPath = key_str.split('.').map(|s| s.trim().to_string()).collect();
        let mut full_key = current_table.clone();
        full_key.extend(key);
        let value = parse_value(value_str)?;
        out.push(Entry { key: full_key, value });
    }
    Ok(out)
}

fn parse_value(s: &str) -> Result<Value, String> {
    let s = s.trim();
    if s.is_empty() { return Err("empty value".into()); }
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"').ok_or("unterminated string")?;
        return Ok(Value::Str(rest[..end].to_string()));
    }
    if s == "true" { return Ok(Value::Bool(true)); }
    if s == "false" { return Ok(Value::Bool(false)); }
    if let Some(rest) = s.strip_prefix('[') {
        if !rest.ends_with(']') { return Err("unterminated array".into()); }
        let inner = &rest[..rest.len()-1];
        let mut items = Vec::new();
        let mut depth = 0i32;
        let mut start = 0usize;
        for (i, c) in inner.char_indices() {
            match c {
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    items.push(parse_value(inner[start..i].trim())?);
                    start = i + 1;
                }
                _ => {}
            }
        }
        let last = inner[start..].trim();
        if !last.is_empty() { items.push(parse_value(last)?); }
        return Ok(Value::Array(items));
    }
    if let Some(rest) = s.strip_prefix('{') {
        if !rest.ends_with('}') { return Err("unterminated inline table".into()); }
        let inner = &rest[..rest.len()-1];
        let mut entries: BTreeMap<String, Value> = BTreeMap::new();
        let mut depth = 0i32;
        let mut start = 0usize;
        for (i, c) in inner.char_indices() {
            match c {
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    let chunk = inner[start..i].trim();
                    let eq = chunk.find('=').ok_or_else(|| format!("inline table entry: {}", chunk))?;
                    let k = chunk[..eq].trim().to_string();
                    let v = parse_value(chunk[eq+1..].trim())?;
                    entries.insert(k, v);
                    start = i + 1;
                }
                _ => {}
            }
        }
        let last = inner[start..].trim();
        if !last.is_empty() {
            let eq = last.find('=').ok_or_else(|| format!("inline table entry: {}", last))?;
            let k = last[..eq].trim().to_string();
            let v = parse_value(last[eq+1..].trim())?;
            entries.insert(k, v);
        }
        return Ok(Value::InlineTable(entries));
    }
    if s.contains('-') && s.len() >= 8 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        return Ok(Value::Date(s.to_string()));
    }
    if s.contains('.') || s.contains('e') || s.contains('E') {
        if let Ok(f) = s.parse::<f64>() { return Ok(Value::Float(f)); }
    }
    if let Ok(i) = s.parse::<i64>() { return Ok(Value::Int(i)); }
    Err(format!("cannot parse value: {}", s))
}

pub fn group_by_table(entries: &[Entry]) -> BTreeMap<String, BTreeMap<String, Value>> {
    let mut map: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for e in entries {
        let (table, leaf) = if e.key.len() > 1 {
            let tbl = e.key[..e.key.len()-1].join(".");
            let leaf = e.key.last().cloned().unwrap_or_default();
            (tbl, leaf)
        } else {
            ("".to_string(), e.key.first().cloned().unwrap_or_default())
        };
        map.entry(table).or_default().insert(leaf, e.value.clone());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() { assert_eq!(parse(""), Ok(vec![])); }
    #[test] fn root_int() {
        let v = parse("foo = 1").unwrap();
        assert_eq!(v, vec![Entry { key: vec!["foo".into()], value: Value::Int(1) }]);
    }
    #[test] fn table_section() {
        let v = parse("[s]\nfoo=1\n").unwrap();
        assert_eq!(v[0].key, vec![String::from("s"), String::from("foo")]);
        assert_eq!(v[0].value, Value::Int(1));
    }
    #[test] fn dotted_table() {
        let v = parse("[a.b]\nfoo=1\n").unwrap();
        assert_eq!(v[0].key, vec![String::from("a"), String::from("b"), String::from("foo")]);
    }
    #[test] fn string_value() {
        let v = parse("foo = \"hello\"").unwrap();
        assert_eq!(v[0].value, Value::Str("hello".into()));
    }
    #[test] fn bool_values() {
        let v = parse("a=true\nb=false\n").unwrap();
        assert_eq!(v[0].value, Value::Bool(true));
        assert_eq!(v[1].value, Value::Bool(false));
    }
    #[test] fn float_value() {
        let v = parse("x = 3.14").unwrap();
        assert_eq!(v[0].value, Value::Float(3.14));
    }
    #[test] fn array_value() {
        let v = parse("xs = [1, 2, 3]").unwrap();
        assert_eq!(v[0].value, Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }
    #[test] fn inline_table() {
        let v = parse("pt = { x = 1, y = 2 }").unwrap();
        if let Value::InlineTable(m) = &v[0].value {
            assert_eq!(m.get("x").unwrap(), &Value::Int(1));
        } else { panic!(); }
    }
    #[test] fn date_value() {
        let v = parse("d = 2026-07-05").unwrap();
        assert_eq!(v[0].value, Value::Date("2026-07-05".into()));
    }
    #[test] fn comments_skipped() {
        let v = parse("# comment\nfoo=1\n").unwrap();
        assert_eq!(v.len(), 1);
    }
    #[test] fn group_view() {
        let v = parse("[s]\na=1\nb=2\n[s]\nc=3\n").unwrap();
        let g = group_by_table(&v);
        assert_eq!(g.get("s").unwrap().get("a").unwrap(), &Value::Int(1));
        assert_eq!(g.get("s").unwrap().get("c").unwrap(), &Value::Int(3));
    }
    #[test] fn missing_equals() {
        assert!(parse("foo").is_err());
    }
}