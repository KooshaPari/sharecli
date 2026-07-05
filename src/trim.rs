pub fn trim_start(s: &str) -> &str { let mut i = 0; while i < s.len() && s.as_bytes()[i].is_ascii_whitespace() { i += 1; } &s[i..] }
pub fn trim_end(s: &str) -> &str { let mut i = s.len(); while i > 0 && s.as_bytes()[i-1].is_ascii_whitespace() { i -= 1; } &s[..i] }
pub fn trim(s: &str) -> &str { trim_end(trim_start(s)) }
pub fn split_lines(s: &str) -> Vec<&str> { s.split('\n').collect() }
pub fn split_words(s: &str) -> Vec<&str> { s.split_whitespace().collect() }
pub fn pad_left(s: &str, width: usize, fill: char) -> String {
    if s.len() >= width { return s.to_string(); }
    let mut out = String::new(); for _ in 0..(width - s.len()) { out.push(fill); } out.push_str(s); out
}
pub fn pad_right(s: &str, width: usize, fill: char) -> String {
    if s.len() >= width { return s.to_string(); }
    let mut out = s.to_string(); while out.len() < width { out.push(fill); } out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn trim_spaces() { assert_eq!(trim("  hello  "), "hello"); }
    #[test] fn trim_start_only() { assert_eq!(trim_start("  hello"), "hello"); assert_eq!(trim_start("hello"), "hello"); }
    #[test] fn trim_end_only() { assert_eq!(trim_end("hello  "), "hello"); }
    #[test] fn split_lines_basic() { assert_eq!(split_lines("a\nb\nc"), vec!["a", "b", "c"]); }
    #[test] fn split_words_basic() { assert_eq!(split_words("hello world foo"), vec!["hello", "world", "foo"]); }
    #[test] fn pad_left_basic() { assert_eq!(pad_left("hi", 5, '0'), "000hi"); }
    #[test] fn pad_right_basic() { assert_eq!(pad_right("hi", 5, '-'), "hi---"); }
    #[test] fn pad_no_change() { assert_eq!(pad_left("hello", 3, ' '), "hello"); }
}
