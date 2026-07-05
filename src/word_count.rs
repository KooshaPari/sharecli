// Word/line/byte/char counting utilities with various whitespace options.
#[derive(Debug, PartialEq)]
pub struct Counts {
    pub bytes: usize,
    pub chars: usize,
    pub words: usize,
    pub lines: usize,
}
pub fn count_default(input: &str) -> Counts {
    count_with(input, b" \t\n")
}
pub fn count_with(input: &str, whitespace: &[u8]) -> Counts {
    let bytes = input.len();
    let chars = input.chars().count();
    let lines = if input.is_empty() { 0 } else { input.split('\n').count() };
    let mut words = 0usize;
    let mut in_word = false;
    for c in input.chars() {
        let is_ws = c.is_whitespace() || whitespace.contains(&(c as u32 as u8));
        if !is_ws && !in_word { words += 1; in_word = true; }
        else if is_ws { in_word = false; }
    }
    Counts { bytes, chars, words, lines }
}
pub fn word_count(input: &str) -> usize { count_default(input).words }
pub fn line_count(input: &str) -> usize { count_default(input).lines }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() {
        let c = count_default("");
        assert_eq!(c.bytes, 0);
        assert_eq!(c.chars, 0);
        assert_eq!(c.words, 0);
        assert_eq!(c.lines, 0);
    }
    #[test] fn single_word() {
        let c = count_default("hello");
        assert_eq!(c.words, 1);
        assert_eq!(c.chars, 5);
        assert_eq!(c.bytes, 5);
    }
    #[test] fn multi_word() {
        let c = count_default("hello world foo bar");
        assert_eq!(c.words, 4);
    }
    #[test] fn multiple_lines() {
        let c = count_default("line 1\nline 2\nline 3");
        assert_eq!(c.lines, 3);
        assert_eq!(c.words, 6);
    }
    #[test] fn trailing_newline_counts() {
        let c = count_default("hello\n");
        assert_eq!(c.lines, 2);
    }
    #[test] fn unicode_chars() {
        let c = count_default("héllo wörld");
        assert_eq!(c.chars, 11);
        assert_eq!(c.bytes, 13); // é and ö are 2 bytes each in UTF-8 (1+2+1+1+1+1+1+2+1+1+1)
        assert_eq!(c.words, 2);
    }
    #[test] fn tab_whitespace() {
        let c = count_default("a\tb\tc");
        assert_eq!(c.words, 3);
    }
    #[test] fn multiple_spaces() {
        let c = count_default("a   b");
        assert_eq!(c.words, 2);
    }
    #[test] fn only_whitespace() {
        let c = count_default("   \n   ");
        assert_eq!(c.words, 0);
        assert_eq!(c.lines, 2);
    }
    #[test] fn word_count_helper() {
        assert_eq!(word_count("one two three"), 3);
    }
    #[test] fn line_count_helper() {
        assert_eq!(line_count("a\nb\nc"), 3);
    }
    #[test] fn custom_whitespace() {
        let c = count_with("a,b,c", b",");
        assert_eq!(c.words, 3);
    }
    #[test] fn mixed_punct() {
        let c = count_default("Don't worry, be happy!");
        assert_eq!(c.words, 4);
    }
}
