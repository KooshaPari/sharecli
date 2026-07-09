// Minimal SSH known_hosts file parser.
//
// Format (one entry per line):
//   [markers] hostname-pattern key-type key-bytes
//
// Markers (optional, comma-separated) include @cert-authority, @revoked,
// @revoked-old, plus hashed-hostname markers like |1|salt|hash. Hostname
// patterns may use `*` and `?` wildcards and `[!...]` negation (OpenSSH
// pattern semantics). Comments start with `#`. Blank lines are skipped.
//
// This module parses the textual representation and provides host matching
// for the common wildcard forms. It does NOT validate key-type semantics
// beyond parsing the key-type string, nor does it understand the
// @cert-authority marker beyond recording its presence.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub markers: Vec<String>,
    pub pattern: String,
    pub key_type: String,
    pub key_bytes: Vec<u8>,
}

/// Parse a known_hosts file content into a list of entries.
///
/// Returns an error string if a non-comment, non-blank line is malformed
/// (wrong number of fields, bad base64, missing key-type, etc.).
pub fn parse(input: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for (lineno, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let entry = parse_line(line).map_err(|e| format!("line {}: {}", lineno + 1, e))?;
        out.push(entry);
    }
    Ok(out)
}

fn parse_line(line: &str) -> Result<Entry, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 3 {
        return Err(format!("expected at least 3 fields, got {}", tokens.len()));
    }

    // The last two tokens are always key-type and key-bytes (base64).
    let key_type = tokens[tokens.len() - 2].to_string();
    let key_b64 = tokens[tokens.len() - 1];
    let key_bytes = base64_decode(key_b64).map_err(|e| format!("invalid base64 for key: {}", e))?;

    // Everything before the last two tokens is the host portion. The host
    // portion may itself be space-separated if there are multiple patterns
    // (e.g. "host1,host2 key-type key-bytes"). OpenSSH permits comma-
    // separated host patterns on a single line. We expose only the first
    // pattern here for matching; multiple patterns would require the caller
    // to split. We collect markers (prefixed with @) into markers[]; the
    // first non-marker token becomes the pattern (only the first comma-
    // separated pattern is kept).
    let host_portion = tokens[..tokens.len() - 2].join(" ");
    let (markers, pattern) = split_host_portion(&host_portion)?;

    if pattern.is_empty() {
        return Err("empty hostname pattern".into());
    }

    Ok(Entry { markers, pattern, key_type, key_bytes })
}

fn split_host_portion(s: &str) -> Result<(Vec<String>, String), String> {
    let mut markers = Vec::new();
    for tok in s.split_whitespace() {
        if tok.starts_with('@') {
            markers.push(tok.to_string());
        } else {
            // First non-marker token is the host pattern. Only the first
            // comma-separated pattern is kept.
            let first = tok.split(',').next().unwrap_or("").to_string();
            return Ok((markers, first));
        }
    }
    Ok((markers, String::new()))
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    // OpenSSH key bytes use standard base64. We use a tiny in-house decoder
    // to avoid a dependency; the alphabet matches RFC 4648.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err("length not a multiple of 4".into());
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut i = 0;
    while i < bytes.len() {
        let a = bytes[i];
        let b = bytes[i + 1];
        let c_raw = bytes[i + 2];
        let d_raw = bytes[i + 3];
        let pad = (c_raw == b'=') as usize + (d_raw == b'=') as usize;
        let av = table[a as usize];
        let bv = table[b as usize];
        let cv = if pad >= 1 { 0 } else { table[c_raw as usize] };
        let dv = if pad >= 2 { 0 } else { table[d_raw as usize] };
        if av == 255 || bv == 255 || cv == 255 || dv == 255 {
            return Err("non-base64 character".into());
        }
        out.push((av << 2) | (bv >> 4));
        if pad < 1 {
            out.push((bv << 4) | (cv >> 2));
        }
        if pad < 2 {
            out.push((cv << 6) | dv);
        }
        i += 4;
    }
    Ok(out)
}

/// Returns true if the given hostname matches this entry's pattern.
///
/// Supports OpenSSH-style wildcard patterns:
///   - `*` matches any number of characters (including zero)
///   - `?` matches exactly one character
///   - `[abc]` matches one character from the set
///   - `[!abc]` or `[^abc]` matches one character NOT in the set
///   - `[a-z]` matches a character range
///
/// A plain pattern (no wildcards) matches only itself.
pub fn match_host(entry: &Entry, hostname: &str) -> bool {
    wildcard_match(&entry.pattern, hostname)
}

fn wildcard_match(pattern: &str, s: &str) -> bool {
    let p = pattern.as_bytes();
    let h = s.as_bytes();
    let mut pi = 0usize;
    let mut hi = 0usize;
    let mut star_pi: Option<usize> = None;
    let mut star_hi: usize = 0;
    while hi < h.len() {
        if pi < p.len() {
            match p[pi] {
                b'*' => {
                    star_pi = Some(pi);
                    star_hi = hi;
                    pi += 1;
                    continue;
                }
                b'?' => {
                    pi += 1;
                    hi += 1;
                    continue;
                }
                b'[' => {
                    if let Some((members, allow, next_pi)) = parse_bracket_class(p, pi) {
                        let in_set = members.contains(&h[hi]);
                        let matched = if allow { in_set } else { !in_set };
                        if matched {
                            pi = next_pi;
                            hi += 1;
                            continue;
                        }
                    } else if pi < p.len() && p[pi] == h[hi] {
                        // Malformed bracket — treat '[' as literal.
                        pi += 1;
                        hi += 1;
                        continue;
                    }
                }
                c if c == h[hi] => {
                    pi += 1;
                    hi += 1;
                    continue;
                }
                _ => {}
            }
        }
        if let Some(sp) = star_pi {
            pi = sp + 1;
            star_hi += 1;
            hi = star_hi;
            continue;
        }
        return false;
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }
    pi == p.len()
}

/// Parse a bracket character class starting at p[start] (which must be '[').
/// Returns (members, allow, next_index_past_bracket) where `allow` is true for
/// positive classes (`[abc]`), false for negated (`[!abc]` / `[^abc]`).
/// Returns None if the bracket expression is malformed (no closing `]`).
fn parse_bracket_class(p: &[u8], start: usize) -> Option<(Vec<u8>, bool, usize)> {
    if start >= p.len() || p[start] != b'[' {
        return None;
    }
    let mut i = start + 1;
    let negate = i < p.len() && (p[i] == b'!' || p[i] == b'^');
    if negate {
        i += 1;
    }
    let mut members: Vec<u8> = Vec::new();
    let mut last: Option<u8> = None;
    while i < p.len() && p[i] != b']' {
        let c = p[i];
        if c == b'-' && last.is_some() && i + 1 < p.len() && p[i + 1] != b']' {
            let lo = last.unwrap();
            let hi_b = p[i + 1];
            for b in lo..=hi_b {
                members.push(b);
            }
            last = None;
            i += 2;
            continue;
        }
        members.push(c);
        last = Some(c);
        i += 1;
    }
    if i >= p.len() {
        return None;
    }
    Some((members, !negate, i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A known valid ed25519 host key (test fixture from OpenSSH test suite
    // style — synthetic for unit test purposes).
    const ED25519_KEY: &str =
        "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";

    #[test]
    fn parses_simple_entry() {
        let input = format!("github.com ssh-ed25519 {}\n", ED25519_KEY);
        let entries = parse(&input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].pattern, "github.com");
        assert_eq!(entries[0].key_type, "ssh-ed25519");
        assert!(!entries[0].key_bytes.is_empty());
        assert!(entries[0].markers.is_empty());
    }

    #[test]
    fn parses_marker_entry() {
        let input = format!("@revoked example.com ssh-rsa {}\n", ED25519_KEY);
        let entries = parse(&input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].markers, vec!["@revoked".to_string()]);
        assert_eq!(entries[0].pattern, "example.com");
    }

    #[test]
    fn parses_cert_authority_marker() {
        let input = format!("@cert-authority *.example.com ssh-ed25519 {}\n", ED25519_KEY);
        let entries = parse(&input).unwrap();
        assert_eq!(entries[0].markers, vec!["@cert-authority".to_string()]);
        assert_eq!(entries[0].pattern, "*.example.com");
    }

    #[test]
    fn parses_hashed_hostname_marker() {
        // |1|base64salt|base64hash is the OpenSSH hashed hostname format.
        let input = format!(
            "|1|HjSMTCuqSUxYhGy3xZc5aA==|AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= ssh-ed25519 {}\n",
            ED25519_KEY
        );
        let entries = parse(&input).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].pattern.starts_with("|1|"));
        assert_eq!(entries[0].key_type, "ssh-ed25519");
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let input = format!(
            "# this is a comment\n\
             \n\
             # another comment\n\
             github.com ssh-ed25519 {}\n\
             \n\
             # trailing comment\n",
            ED25519_KEY
        );
        let entries = parse(&input).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].pattern, "github.com");
    }

    #[test]
    fn rejects_malformed_line_too_few_fields() {
        let input = "github.com ssh-ed25519\n";
        let err = parse(input).unwrap_err();
        assert!(err.contains("expected at least 3 fields"), "got: {}", err);
    }

    #[test]
    fn rejects_invalid_base64_key() {
        let input = "github.com ssh-ed25519 !!!notbase64!!!\n";
        let err = parse(input).unwrap_err();
        assert!(err.contains("invalid base64"), "got: {}", err);
    }

    #[test]
    fn match_host_exact() {
        let entry = Entry {
            markers: vec![],
            pattern: "github.com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "github.com"));
        assert!(!match_host(&entry, "api.github.com"));
        assert!(!match_host(&entry, "GitHub.com"));
    }

    #[test]
    fn match_host_star_wildcard() {
        let entry = Entry {
            markers: vec![],
            pattern: "*.example.com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "api.example.com"));
        assert!(match_host(&entry, "foo.example.com"));
        assert!(!match_host(&entry, "example.com"));
        assert!(!match_host(&entry, "example.org"));
    }

    #[test]
    fn match_host_question_wildcard() {
        let entry = Entry {
            markers: vec![],
            pattern: "host?.com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "host1.com"));
        assert!(match_host(&entry, "hosta.com"));
        assert!(!match_host(&entry, "host12.com"));
        assert!(!match_host(&entry, "host.com"));
    }

    #[test]
    fn match_host_bracket_negation() {
        let entry = Entry {
            markers: vec![],
            pattern: "host[!abc].com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "hostd.com"));
        assert!(match_host(&entry, "hostx.com"));
        assert!(!match_host(&entry, "hosta.com"));
        assert!(!match_host(&entry, "hostb.com"));
        assert!(!match_host(&entry, "hostc.com"));
    }

    #[test]
    fn match_host_bracket_range() {
        let entry = Entry {
            markers: vec![],
            pattern: "host[0-9].com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "host0.com"));
        assert!(match_host(&entry, "host5.com"));
        assert!(match_host(&entry, "host9.com"));
        assert!(!match_host(&entry, "hosta.com"));
    }

    #[test]
    fn match_host_bracket_caret_negation() {
        let entry = Entry {
            markers: vec![],
            pattern: "host[^abc].com".into(),
            key_type: "ssh-ed25519".into(),
            key_bytes: vec![],
        };
        assert!(match_host(&entry, "hostd.com"));
        assert!(!match_host(&entry, "hosta.com"));
    }
}
