// Base-N radix conversion (2..36) with arbitrary-precision representation via Vec<u32> limbs.
const LIMB_BITS: u32 = 32;
const LIMB_MASK: u64 = 0xffff_ffff;

pub fn encode(value: u64, base: u32) -> String {
    if base < 2 || base > 36 {
        return format!("__invalid_base_{}", base);
    }
    if value == 0 { return "0".to_string(); }
    let mut v = value;
    let mut out = String::new();
    while v > 0 {
        let d = (v % base as u64) as u32;
        let c = if d < 10 { b'0' + d as u8 } else { b'a' + (d - 10) as u8 };
        out.push(c as char);
        v /= base as u64;
    }
    out.chars().rev().collect()
}
pub fn decode(s: &str, base: u32) -> Result<u64, String> {
    if base < 2 || base > 36 { return Err(format!("base out of range: {}", base)); }
    let mut out: u64 = 0;
    for c in s.chars() {
        let d = match c {
            '0'..='9' => c as u32 - '0' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 10,
            'A'..='Z' => c as u32 - 'A' as u32 + 10,
            _ => return Err(format!("invalid char: {}", c)),
        };
        if d >= base { return Err(format!("digit {} >= base {}", d, base)); }
        out = out.checked_mul(base as u64).ok_or("overflow")?;
        out = out.checked_add(d as u64).ok_or("overflow")?;
    }
    Ok(out)
}
pub fn big_encode(value: &[u32], base: u32) -> String {
    if base < 2 || base > 36 { return format!("__invalid_base_{}", base); }
    if value.is_empty() || value.iter().all(|&x| x == 0) { return "0".to_string(); }
    // Process limbs high-to-low (big-endian).
    let mut limbs: Vec<u32> = value.to_vec();
    while !limbs.is_empty() && *limbs.last().unwrap() == 0 { limbs.pop(); }
    let mut out = String::new();
    while !limbs.is_empty() {
        let mut carry: u64 = 0;
        for i in (0..limbs.len()).rev() {
            let v = (carry << LIMB_BITS) | limbs[i] as u64;
            limbs[i] = (v / base as u64) as u32;
            carry = v % base as u64;
        }
        let c = if carry < 10 { b'0' + carry as u8 } else { b'a' + (carry - 10) as u8 };
        out.push(c as char);
        while !limbs.is_empty() && *limbs.last().unwrap() == 0 { limbs.pop(); }
    }
    out.chars().rev().collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn zero_in_all_bases() {
        assert_eq!(encode(0, 2), "0");
        assert_eq!(encode(0, 10), "0");
        assert_eq!(encode(0, 16), "0");
        assert_eq!(encode(0, 36), "0");
    }
    #[test] fn encode_decimal() {
        assert_eq!(encode(123, 10), "123");
        assert_eq!(encode(4567, 10), "4567");
    }
    #[test] fn encode_binary() {
        assert_eq!(encode(5, 2), "101");
        assert_eq!(encode(255, 2), "11111111");
    }
    #[test] fn encode_hex() {
        assert_eq!(encode(255, 16), "ff");
        assert_eq!(encode(0xdeadbeef, 16), "deadbeef");
    }
    #[test] fn encode_base36() {
        assert_eq!(encode(35, 36), "z");
        assert_eq!(encode(36, 36), "10");
    }
    #[test] fn round_trip() {
        for base in 2..=36 {
            for v in [0u64, 1, 42, 255, 9999, u64::MAX] {
                let e = encode(v, base);
                let d = decode(&e, base).unwrap();
                assert_eq!(d, v, "round trip failed for v={} base={}", v, base);
            }
        }
    }
    #[test] fn decode_case_insensitive() {
        assert_eq!(decode("DEADBEEF", 16).unwrap(), 0xdeadbeef);
        assert_eq!(decode("deadbeef", 16).unwrap(), 0xdeadbeef);
    }
    #[test] fn decode_invalid_char() {
        assert!(decode("12x3", 10).is_err());
    }
    #[test] fn decode_digit_too_large() {
        assert!(decode("9", 8).is_err());
    }
    #[test] fn big_encode_basic() {
        assert_eq!(big_encode(&[0], 10), "0");
        assert_eq!(big_encode(&[123], 10), "123");
    }
    #[test] fn big_encode_hex() {
        let v = [0xdeadbeefu32, 0x12345678];
        let s = big_encode(&v, 16);
        assert_eq!(decode(&s, 16).unwrap(), 0x12345678deadbeefu64);
    }
    #[test] fn invalid_base() {
        assert!(encode(10, 1).starts_with("__invalid_base_"));
        assert!(decode("10", 1).is_err());
        assert!(decode("10", 37).is_err());
    }
}
