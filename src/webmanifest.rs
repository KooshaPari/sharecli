// Minimal Web App Manifest parser/validator.
//
// Spec: W3C Web App Manifest https://www.w3.org/TR/appmanifest/
//
// The manifest is a JSON document. Two members are required (per spec):
//   - `name` (or `short_name`)
//   - `start_url` (which itself must be a valid URL string)
//
// Common members:
//   - `short_name`, `description`, `id`, `lang`, `dir`
//   - `icons`: array of {src, sizes, type, purpose}
//   - `display`: "fullscreen" | "standalone" | "minimal-ui" | "browser"
//   - `orientation`, `theme_color`, `background_color`, `scope`
//   - `start_url`, `scope`
//
// `display` defaults to "browser" if omitted. `lang` has no default
// (but if absent the manifest is treated as English).
//
// `theme_color` and `background_color` must parse as CSS <color> — we
// accept `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb(...)`, `rgba(...)`,
// `hsl(...)`, `hsla(...)`, and the named-color "transparent".

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestIcon {
    pub src: String,
    /// Sizes list as raw text (e.g. `"48x48 96x96"`).
    pub sizes: String,
    pub mime_type: String,
    /// Space-separated list of purposes (e.g. `"any maskable"`).
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub id: String,
    pub lang: String,
    pub dir: String,
    pub start_url: String,
    pub scope: String,
    /// One of "fullscreen", "standalone", "minimal-ui", "browser".
    pub display: String,
    pub orientation: String,
    pub theme_color: String,
    pub background_color: String,
    pub icons: Vec<ManifestIcon>,
}

/// Extract the value of a top-level string field from a flat JSON object
/// body. Handles `"key":"value"` and `"key": "value"` (with optional
/// whitespace) and escaped quotes inside the value. Returns `None` if
/// the key is absent or the value is not a JSON string.
fn extract_string(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let mut idx = 0;
    while let Some(found) = body[idx..].find(&needle) {
        let abs = idx + found;
        // Make sure this is a key (preceded by `,`, `{`, or whitespace) and not
        // a substring of some larger key like `"short_name"` matching `"name"`.
        let before_ok = abs == 0
            || matches!(body.as_bytes()[abs - 1], b',' | b'{' | b'\n' | b' ' | b'\t' | b'\r');
        if !before_ok {
            idx = abs + 1;
            continue;
        }
        let mut p = abs + needle.len();
        // Skip whitespace.
        while p < body.len() && matches!(body.as_bytes()[p], b' ' | b'\t' | b'\r' | b'\n') {
            p += 1;
        }
        if p >= body.len() || body.as_bytes()[p] != b':' {
            idx = abs + 1;
            continue;
        }
        p += 1;
        while p < body.len() && matches!(body.as_bytes()[p], b' ' | b'\t' | b'\r' | b'\n') {
            p += 1;
        }
        if p >= body.len() || body.as_bytes()[p] != b'"' {
            return None;
        }
        p += 1;
        let mut out = String::new();
        let mut esc = false;
        while p < body.len() {
            let c = body.as_bytes()[p];
            if esc {
                // Minimal escape handling: just append the character.
                out.push(c as char);
                esc = false;
                p += 1;
                continue;
            }
            if c == b'\\' {
                esc = true;
                p += 1;
                continue;
            }
            if c == b'"' {
                return Some(out);
            }
            out.push(c as char);
            p += 1;
        }
        return None;
    }
    None
}

/// Parse a JSON object body into a flat `HashMap<String, JsonValue>` of the
/// top-level members we care about. Values we don't model (numbers, nested
/// arrays of non-icons) are skipped.
fn parse_object_members(body: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip whitespace and commas.
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n' | b',') {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'}' {
            break;
        }
        if bytes[i] != b'"' {
            break;
        }
        // Read key.
        i += 1;
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' {
                i += 1;
            }
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key = body[key_start..i].to_string();
        i += 1;
        // Skip ws and find ':'.
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            break;
        }
        i += 1;
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
        }
        // Read value: string or array/object (we just slurp the rest).
        if i >= bytes.len() {
            break;
        }
        let value_start = i;
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            out.insert(key, body[value_start..i].to_string());
        } else if bytes[i] == b'[' || bytes[i] == b'{' {
            let open = bytes[i];
            let _close = if open == b'[' { b']' } else { b'}' };
            let mut depth: i32 = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'"' => {
                        i += 1;
                        while i < bytes.len() && bytes[i] != b'"' {
                            if bytes[i] == b'\\' {
                                i += 1;
                            }
                            i += 1;
                        }
                        if i < bytes.len() {
                            i += 1;
                        }
                    }
                    b'[' | b'{' => {
                        depth += 1;
                        i += 1;
                    }
                    b']' | b'}' => {
                        depth -= 1;
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            out.insert(key, body[value_start..i].to_string());
        } else {
            // Number, bool, null — slurp until separator.
            while i < bytes.len() && !matches!(bytes[i], b',' | b'}' | b' ' | b'\t' | b'\r' | b'\n')
            {
                i += 1;
            }
            out.insert(key, body[value_start..i].to_string());
        }
    }
    out
}

/// Extract the array body for a given top-level key (without the surrounding
/// brackets). Returns an empty string if not present.
fn extract_array(body: &str, key: &str) -> Option<String> {
    let members = parse_object_members(body);
    let raw = members.get(key)?;
    let raw = raw.trim();
    if !raw.starts_with('[') || !raw.ends_with(']') {
        return None;
    }
    Some(raw[1..raw.len() - 1].to_string())
}

/// Parse the icons array. Returns `Vec<ManifestIcon>`.
pub fn parse_icons(body: &str) -> Vec<ManifestIcon> {
    let Some(arr) = extract_array(body, "icons") else { return Vec::new() };
    let mut icons = Vec::new();
    // Split top-level objects inside the array.
    let bytes = arr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n' | b',') {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b']' {
            break;
        }
        if bytes[i] != b'{' {
            break;
        }
        let obj_start = i;
        let mut depth: i32 = 1;
        i += 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'"' => {
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        if bytes[i] == b'\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                }
                b'{' => {
                    depth += 1;
                    i += 1;
                }
                b'}' => {
                    depth -= 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        let obj = &arr[obj_start..i];
        let m = ManifestIcon {
            src: extract_string(obj, "src").unwrap_or_default(),
            sizes: extract_string(obj, "sizes").unwrap_or_default(),
            mime_type: extract_string(obj, "type").unwrap_or_default(),
            purpose: extract_string(obj, "purpose").unwrap_or_default(),
        };
        icons.push(m);
    }
    icons
}

/// Validate a CSS color string. Accepts:
/// - `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`
/// - `rgb(...)`, `rgba(...)`, `hsl(...)`, `hsla(...)` (function-form)
/// - The keyword `transparent`
/// Returns `true` if the value is plausibly a valid CSS color.
pub fn is_css_color(s: &str) -> bool {
    let s = s.trim();
    if s.eq_ignore_ascii_case("transparent") || s.eq_ignore_ascii_case("currentcolor") {
        return true;
    }
    if let Some(rest) = s.strip_prefix('#') {
        return matches!(rest.len(), 3 | 4 | 6 | 8) && rest.chars().all(|c| c.is_ascii_hexdigit());
    }
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("rgb(")
        || lower.starts_with("rgba(")
        || lower.starts_with("hsl(")
        || lower.starts_with("hsla(")
        || lower.starts_with("color(")
    {
        return s.ends_with(')');
    }
    // Otherwise we don't validate named colors (would need a full table).
    false
}

/// Parse and validate a manifest JSON string. Returns `Err` if required
/// fields are missing or have wrong types.
pub fn parse(input: &str) -> Result<Manifest, String> {
    let text = input.strip_prefix('\u{FEFF}').unwrap_or(input).trim();
    if !text.starts_with('{') || !text.ends_with('}') {
        return Err("manifest must be a JSON object".into());
    }
    let body = &text[1..text.len() - 1];

    let name = extract_string(body, "name").unwrap_or_default();
    let short_name = extract_string(body, "short_name").unwrap_or_default();
    if name.is_empty() && short_name.is_empty() {
        return Err("manifest missing required name or short_name".into());
    }
    let start_url = extract_string(body, "start_url").unwrap_or_default();
    if start_url.is_empty() {
        return Err("manifest missing required start_url".into());
    }
    let display = extract_string(body, "display").unwrap_or_else(|| "browser".to_string());
    let theme_color = extract_string(body, "theme_color").unwrap_or_default();
    if !theme_color.is_empty() && !is_css_color(&theme_color) {
        return Err(format!("invalid theme_color: {theme_color:?}"));
    }
    let background_color = extract_string(body, "background_color").unwrap_or_default();
    if !background_color.is_empty() && !is_css_color(&background_color) {
        return Err(format!("invalid background_color: {background_color:?}"));
    }
    let icons = parse_icons(body);

    Ok(Manifest {
        name,
        short_name,
        description: extract_string(body, "description").unwrap_or_default(),
        id: extract_string(body, "id").unwrap_or_default(),
        lang: extract_string(body, "lang").unwrap_or_default(),
        dir: extract_string(body, "dir").unwrap_or_default(),
        start_url,
        scope: extract_string(body, "scope").unwrap_or_default(),
        display,
        orientation: extract_string(body, "orientation").unwrap_or_default(),
        theme_color,
        background_color,
        icons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_manifest_with_name_and_start_url() {
        let input = r#"{"name":"My App","start_url":"/"}"#;
        let m = parse(input).unwrap();
        assert_eq!(m.name, "My App");
        assert_eq!(m.start_url, "/");
        assert_eq!(m.display, "browser"); // spec default
        assert!(m.icons.is_empty());
    }

    #[test]
    fn manifest_with_short_name_instead_of_name() {
        let input = r#"{"short_name":"Mini","start_url":"/home"}"#;
        let m = parse(input).unwrap();
        assert_eq!(m.name, "");
        assert_eq!(m.short_name, "Mini");
        assert_eq!(m.start_url, "/home");
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert!(parse(r#"{"name":"X"}"#).is_err()); // no start_url
        assert!(parse(r#"{"start_url":"/"}"#).is_err()); // no name or short_name
    }

    #[test]
    fn rejects_non_object() {
        assert!(parse(r#"[]"#).is_err());
        assert!(parse("not json").is_err());
    }

    #[test]
    fn parses_display_and_colors() {
        let input = r##"{
            "name": "PWA",
            "short_name": "p",
            "start_url": "/",
            "display": "standalone",
            "theme_color": "#ff00aa",
            "background_color": "#fff"
        }"##;
        let m = parse(input).unwrap();
        assert_eq!(m.display, "standalone");
        assert_eq!(m.theme_color, "#ff00aa");
        assert_eq!(m.background_color, "#fff");
    }

    #[test]
    fn rejects_bad_theme_color() {
        let input = r#"{"name":"X","start_url":"/","theme_color":"not-a-color"}"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn css_color_accepts_known_forms() {
        assert!(is_css_color(r#"#fff"#));
        assert!(is_css_color(r#"#FFFFFF"#));
        assert!(is_css_color(r#"#11223344"#));
        assert!(is_css_color("rgb(0,0,0)"));
        assert!(is_css_color("rgba(0,0,0,0.5)"));
        assert!(is_css_color("hsl(120, 50%, 50%)"));
        assert!(is_css_color("transparent"));
        assert!(!is_css_color(""));
        assert!(!is_css_color(r#"#zzz"#));
        assert!(!is_css_color(r#"#ff"#)); // wrong length
        assert!(!is_css_color(r#"#fffff"#)); // wrong length
        assert!(!is_css_color(r#"#1234567"#)); // wrong length
    }

    #[test]
    fn parses_icons_array() {
        let input = r#"{
            "name": "App",
            "start_url": "/",
            "icons": [
                {"src": "/icon-48.png", "sizes": "48x48", "type": "image/png"},
                {"src": "/maskable.svg", "sizes": "any", "type": "image/svg+xml", "purpose": "maskable"}
            ]
        }"#;
        let m = parse(input).unwrap();
        assert_eq!(m.icons.len(), 2);
        assert_eq!(m.icons[0].src, "/icon-48.png");
        assert_eq!(m.icons[0].sizes, "48x48");
        assert_eq!(m.icons[0].mime_type, "image/png");
        assert_eq!(m.icons[1].purpose, "maskable");
    }

    #[test]
    fn extract_string_handles_escaped_quotes() {
        let body = r#""name":"He said \"hi\"""#;
        assert_eq!(extract_string(body, "name").as_deref(), Some("He said \"hi\""));
    }

    #[test]
    fn manifest_with_lang_dir_id_scope_description() {
        let input = r#"{
            "name": "Full",
            "short_name": "F",
            "id": "/?homescreen=1",
            "start_url": "/?source=pwa",
            "scope": "/",
            "lang": "en-US",
            "dir": "ltr",
            "description": "A demo PWA",
            "display": "fullscreen",
            "orientation": "portrait"
        }"#;
        let m = parse(input).unwrap();
        assert_eq!(m.id, "/?homescreen=1");
        assert_eq!(m.scope, "/");
        assert_eq!(m.lang, "en-US");
        assert_eq!(m.dir, "ltr");
        assert_eq!(m.description, "A demo PWA");
        assert_eq!(m.display, "fullscreen");
        assert_eq!(m.orientation, "portrait");
    }
}
