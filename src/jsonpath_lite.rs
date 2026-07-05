pub fn extract_id(s: &str) -> Option<String> {
    let needle = "\"id\":\"";
    if let Some(start) = s.find(needle) {
        let rest = &s[start + needle.len()..];
        if let Some(end) = rest.find('"') { return Some(rest[..end].to_string()); }
    }
    None
}

pub fn extract_field(s: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", field);
    if let Some(start) = s.find(&needle) {
        let rest = &s[start + needle.len()..];
        if let Some(end) = rest.find('"') { return Some(rest[..end].to_string()); }
    }
    None
}

pub fn extract_number(s: &str, field: &str) -> Option<i64> {
    let needle = format!("\"{}\":", field);
    if let Some(start) = s.find(&needle) {
        let rest = s[start + needle.len()..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
        return rest[..end].parse().ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn extract_id_present() { assert_eq!(extract_id(r#"{"id":"abc-123","name":"x"}"#), Some("abc-123".into())); }
    #[test] fn extract_id_missing() { assert_eq!(extract_id(r#"{"name":"x"}"#), None); }
    #[test] fn extract_field_present() { assert_eq!(extract_field(r#"{"name":"alice"}"#, "name"), Some("alice".into())); }
    #[test] fn extract_field_missing() { assert_eq!(extract_field(r#"{"other":1}"#, "name"), None); }
    #[test] fn extract_number() { assert_eq!(extract_number(r#"{"count":42}"#, "count"), Some(42)); }
    #[test] fn extract_number_missing() { assert_eq!(extract_number(r#"{"x":1}"#, "y"), None); }
    #[test] fn extract_number_negative() { assert_eq!(extract_number(r#"{"v":-7}"#, "v"), Some(-7)); }
}
