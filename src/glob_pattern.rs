pub fn match_pattern(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    match_pattern_rec(&pat, 0, &txt, 0)
}
fn match_pattern_rec(pat: &[char], pi: usize, txt: &[char], ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }
    if pat[pi] == '*' {
        // skip consecutive stars
        let mut np = pi;
        while np < pat.len() && pat[np] == '*' { np += 1; }
        if np == pat.len() { return true; }
        for k in ti..=txt.len() {
            if match_pattern_rec(pat, np, txt, k) { return true; }
        }
        false
    } else if pat[pi] == '?' {
        if ti < txt.len() { match_pattern_rec(pat, pi + 1, txt, ti + 1) } else { false }
    } else if pat[pi] == '[' {
        // character class [abc] or [!abc]
        let mut p = pi + 1;
        let neg = p < pat.len() && pat[p] == '!';
        if neg { p += 1; }
        let mut chars_in_class: Vec<char> = Vec::new();
        while p < pat.len() && pat[p] != ']' {
            chars_in_class.push(pat[p]);
            p += 1;
        }
        let end = if p < pat.len() { p } else { return false; };
        let in_class = ti < txt.len() && chars_in_class.contains(&txt[ti]);
        let match_char = if neg { !in_class } else { in_class };
        if match_char { match_pattern_rec(pat, end + 1, txt, ti + 1) } else { false }
    } else {
        if ti < txt.len() && pat[pi] == txt[ti] { match_pattern_rec(pat, pi + 1, txt, ti + 1) } else { false }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exact() { assert!(match_pattern("hello", "hello")); }
    #[test] fn exact_mismatch() { assert!(!match_pattern("hello", "world")); }
    #[test] fn empty_pattern_empty_text() { assert!(match_pattern("", "")); }
    #[test] fn empty_pattern_nonempty() { assert!(!match_pattern("", "x")); }
    #[test] fn star_suffix() { assert!(match_pattern("*.txt", "doc.txt")); }
    #[test] fn star_prefix() { assert!(match_pattern("foo*", "foobar")); }
    #[test] fn star_middle() { assert!(match_pattern("a*c", "abc")); }
    #[test] fn question_mark() { assert!(match_pattern("h?llo", "hello")); }
    #[test] fn question_mismatch() { assert!(!match_pattern("h?llo", "hllo")); }
    #[test] fn char_class_basic() { assert!(match_pattern("[abc]", "a")); }
    #[test] fn char_class_negated() { assert!(match_pattern("[!abc]", "z")); }
    #[test] fn char_class_negate_fail() { assert!(!match_pattern("[!abc]", "a")); }
    #[test] fn only_star() { assert!(match_pattern("*", "anything")); }
    #[test] fn double_star() { assert!(match_pattern("**", "anything")); }
}
