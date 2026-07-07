//! Natural-order string comparison.
//!
//! Sorts strings in "natural" order where embedded numbers sort by
//! numeric value rather than lexicographic order. E.g. `["file10",
//! "file2"]` sorts to `["file2", "file10"]` instead of `["file10",
//! "file2"]`.
//!
//! Useful for file listings (`ls -v` style), version comparisons, and
//! any UI where users expect numeric sub-strings to sort by magnitude.
//!
//! Compares chunk-by-chunk: digit runs are compared as integers; non-
//! digit runs are compared case-insensitively byte-by-byte. Unicode
//! non-ASCII characters are compared byte-by-byte (locale-aware
//! comparison is intentionally NOT supported).

/// Compare two strings in natural order. Returns:
/// - negative if `a` should sort before `b`
/// - positive if `a` should sort after `b`
/// - zero if `a` equals `b`
pub fn natural_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let av: Vec<u8> = a.bytes().collect();
    let bv: Vec<u8> = b.bytes().collect();
    let mut ai = 0;
    let mut bi = 0;
    loop {
        let (ac, bc) = match (av.get(ai), bv.get(bi)) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&x), Some(&y)) => (x, y),
        };
        ai += 1;
        bi += 1;
        if ac.is_ascii_digit() && bc.is_ascii_digit() {
            // Compare the digit runs as integers. Read digits independently
            // on each side — if one side runs out of digits before the other,
            // continue reading the remaining side so "10" > "2" comes out
            // correctly.
            let mut a_val: u64 = (ac - b'0') as u64;
            let mut b_val: u64 = (bc - b'0') as u64;
            loop {
                let a_digit = av
                    .get(ai)
                    .map_or(false, |&x| x.is_ascii_digit());
                let b_digit = bv
                    .get(bi)
                    .map_or(false, |&x| x.is_ascii_digit());
                if a_digit {
                    a_val = a_val
                        .saturating_mul(10)
                        .saturating_add((av[ai] - b'0') as u64);
                    ai += 1;
                }
                if b_digit {
                    b_val = b_val
                        .saturating_mul(10)
                        .saturating_add((bv[bi] - b'0') as u64);
                    bi += 1;
                }
                if !a_digit && !b_digit {
                    break;
                }
            }
            match a_val.cmp(&b_val) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        } else {
            // Compare characters (case-insensitive for ASCII)
            let an = ac.to_ascii_lowercase();
            let bn = bc.to_ascii_lowercase();
            match an.cmp(&bn) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
    }
}

/// Sort a slice of strings in natural order. Returns a new sorted Vec.
pub fn natural_sort<'a>(items: &[&'a str]) -> Vec<&'a str> {
    let mut sorted: Vec<&'a str> = items.to_vec();
    sorted.sort_by(|a, b| natural_compare(a, b));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_sort_naturally() {
        let files = vec!["file10.txt", "file2.txt", "file1.txt", "file20.txt"];
        let sorted = natural_sort(&files);
        assert_eq!(sorted, vec!["file1.txt", "file2.txt", "file10.txt", "file20.txt"]);
    }

    #[test]
    fn versions_sort_naturally() {
        let versions = vec!["v1.10.0", "v1.2.0", "v1.9.0", "v1.0.0"];
        let sorted = natural_sort(&versions);
        assert_eq!(sorted, vec!["v1.0.0", "v1.2.0", "v1.9.0", "v1.10.0"]);
    }

    #[test]
    fn case_insensitive_for_ascii() {
        assert_eq!(natural_compare("File", "file"), std::cmp::Ordering::Equal);
        assert_eq!(natural_compare("Foo", "bar"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn equal_strings_return_equal() {
        assert_eq!(natural_compare("hello", "hello"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn one_is_prefix_of_other() {
        assert_eq!(natural_compare("file", "file1"), std::cmp::Ordering::Less);
        assert_eq!(natural_compare("file1", "file"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn numbers_at_start() {
        assert_eq!(natural_compare("10", "2"), std::cmp::Ordering::Greater);
        assert_eq!(natural_compare("2", "10"), std::cmp::Ordering::Less);
    }

    #[test]
    fn handles_saturation() {
        // Very large numbers saturate; relative order preserved
        assert_eq!(
            natural_compare("99999999999999999999", "1"),
            std::cmp::Ordering::Greater
        );
    }
}