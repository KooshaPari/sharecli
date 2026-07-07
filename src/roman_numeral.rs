//! Roman numeral encoder/decoder.
//!
//! Supports the standard additive/subtractive form (I, II, III, IV, V,
//! VI, ..., MMMCMXCIX = 3999). Out-of-range values return an error.
//!
//! Useful for chapter numbering, ordinal labels, and decorative
//! numbering in document generators. Not optimized for speed — uses
//! a simple greedy + lookup approach.

const VALUES: &[(u32, &str)] = &[
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

const MIN: u32 = 1;
const MAX: u32 = 3999;

/// Encode a positive integer to Roman numerals. Returns `Err` for
/// values outside [1, 3999].
pub fn encode(n: u32) -> Result<String, String> {
    if n < MIN || n > MAX {
        return Err(format!("{} is outside range [{}, {}]", n, MIN, MAX));
    }
    let mut result = String::new();
    let mut remaining = n;
    for &(value, symbol) in VALUES {
        while remaining >= value {
            result.push_str(symbol);
            remaining -= value;
        }
    }
    Ok(result)
}

/// Decode a Roman numeral string to its integer value. Returns `Err` on
/// invalid combinations. Input is case-insensitive.
pub fn decode(s: &str) -> Result<u32, String> {
    let upper: String = s.trim().to_uppercase();
    if upper.is_empty() {
        return Err("empty input".into());
    }
    let mut result: u32 = 0;
    let mut chars = upper.chars().peekable();
    while let Some(c) = chars.next() {
        let current = single_value(c)
            .ok_or_else(|| format!("invalid character '{}'", c))?;
        if let Some(&next_c) = chars.peek() {
            let next = single_value(next_c)
                .ok_or_else(|| format!("invalid character '{}'", next_c))?;
            if current < next {
                // Subtractive: IV, IX, XL, XC, CD, CM
                if !is_valid_subtractive(c, next_c) {
                    return Err(format!("invalid subtractive pair {}{}", c, next_c));
                }
                result += next - current;
                chars.next(); // consume the next char
            } else {
                result += current;
            }
        } else {
            result += current;
        }
    }
    if result < MIN || result > MAX {
        return Err(format!("decoded value {} is outside [{}, {}]", result, MIN, MAX));
    }
    Ok(result)
}

fn single_value(c: char) -> Option<u32> {
    match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

fn is_valid_subtractive(small: char, big: char) -> bool {
    matches!((small, big),
        ('I', 'V') | ('I', 'X') | ('X', 'L') | ('X', 'C') | ('C', 'D') | ('C', 'M')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_basic() {
        assert_eq!(encode(1).unwrap(), "I");
        assert_eq!(encode(4).unwrap(), "IV");
        assert_eq!(encode(9).unwrap(), "IX");
        assert_eq!(encode(40).unwrap(), "XL");
        assert_eq!(encode(58).unwrap(), "LVIII");
        assert_eq!(encode(1994).unwrap(), "MCMXCIV");
    }

    #[test]
    fn encode_boundaries() {
        assert_eq!(encode(1).unwrap(), "I");
        assert_eq!(encode(3999).unwrap(), "MMMCMXCIX");
    }

    #[test]
    fn encode_out_of_range_errors() {
        assert!(encode(0).is_err());
        assert!(encode(4000).is_err());
    }

    #[test]
    fn decode_basic() {
        assert_eq!(decode("I").unwrap(), 1);
        assert_eq!(decode("IV").unwrap(), 4);
        assert_eq!(decode("MCMXCIV").unwrap(), 1994);
    }

    #[test]
    fn decode_case_insensitive() {
        assert_eq!(decode("mcmxciv").unwrap(), 1994);
        assert_eq!(decode("McmxcIv").unwrap(), 1994);
    }

    #[test]
    fn decode_invalid_char_errors() {
        assert!(decode("ABC").is_err());
    }

    #[test]
    fn decode_invalid_subtractive_errors() {
        // "IL" is not a valid subtractive pair (I can only precede V or X)
        assert!(decode("IL").is_err());
    }

    #[test]
    fn round_trip() {
        for n in [1, 4, 9, 42, 100, 444, 999, 1994, 3999] {
            let roman = encode(n).unwrap();
            assert_eq!(decode(&roman).unwrap(), n);
        }
    }
}