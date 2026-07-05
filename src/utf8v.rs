pub fn is_valid(s: &[u8]) -> bool {
    let mut i = 0;
    while i < s.len() {
        let b = s[i];
        let len = if b < 0x80 { 1 }
            else if b & 0xe0 == 0xc0 { 2 }
            else if b & 0xf0 == 0xe0 { 3 }
            else if b & 0xf8 == 0xf0 { 4 }
            else { return false; };
        if i + len > s.len() { return false; }
        if len > 1 {
            for j in 1..len { if s[i + j] & 0xc0 != 0x80 { return false; } }
        }
        i += len;
    }
    true
}
pub fn count_chars(s: &[u8]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < s.len() {
        let b = s[i];
        let len = if b < 0x80 { 1 } else if b & 0xe0 == 0xc0 { 2 } else if b & 0xf0 == 0xe0 { 3 } else if b & 0xf8 == 0xf0 { 4 } else { 1 };
        i += len;
        count += 1;
    }
    count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_ascii_valid() { assert!(is_valid(b"hello")); }
    #[test] fn test_2byte_valid() { assert!(is_valid("hé".as_bytes())); }
    #[test] fn test_3byte_valid() { assert!(is_valid("中".as_bytes())); }
    #[test] fn test_invalid_truncated() { assert!(!is_valid(&[0xc3])); }
    #[test] fn test_invalid_continuation() { assert!(!is_valid(&[0xc3, 0x00])); }
    #[test] fn count_ascii() { assert_eq!(count_chars(b"hello"), 5); }
    #[test] fn count_multibyte() { assert_eq!(count_chars("héllo".as_bytes()), 5); }
}
