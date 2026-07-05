pub fn is_valid_pin(s: &str) -> bool {
    if s.len() != 4 && s.len() != 6 { return false; }
    s.chars().all(|c| c.is_ascii_digit())
}
pub fn normalize_pin(s: &str) -> String {
    let digits: Vec<char> = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() > 6 { return digits[digits.len() - 6..].iter().collect(); }
    let mut result: String = digits.iter().collect();
    while result.len() < 4 { result.insert(0, '0'); }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn valid_pin_4() { assert!(is_valid_pin("1234")); }
    #[test] fn valid_pin_6() { assert!(is_valid_pin("123456")); }
    #[test] fn invalid_too_short() { assert!(!is_valid_pin("12")); }
    #[test] fn invalid_too_long() { assert!(!is_valid_pin("1234567")); }
    #[test] fn invalid_alpha() { assert!(!is_valid_pin("12a4")); }
    #[test] fn normalize_short() { assert_eq!(normalize_pin("12"), "0012"); }
    #[test] fn normalize_too_long() { assert_eq!(normalize_pin("12345678"), "345678"); }
    #[test] fn normalize_strips() { assert_eq!(normalize_pin("12-34"), "1234"); }
}
