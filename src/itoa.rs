const DIGITS: &[u8; 10] = b"0123456789";

pub fn u32_to_str(n: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    let mut v = n;
    while v > 0 {
        buf[i] = DIGITS[(v % 10) as usize];
        v /= 10;
        i += 1;
    }
    buf[..i].reverse();
    std::str::from_utf8(&buf[..i]).unwrap().to_string()
}

pub fn i32_to_str(n: i32) -> String {
    if n >= 0 {
        return u32_to_str(n as u32);
    }
    if n == i32::MIN {
        return "-2147483648".into();
    }
    format!("-{}", u32_to_str((-n) as u32))
}

pub fn u64_to_str(n: u64) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut v = n;
    while v > 0 {
        buf[i] = DIGITS[(v % 10) as usize];
        v /= 10;
        i += 1;
    }
    buf[..i].reverse();
    std::str::from_utf8(&buf[..i]).unwrap().to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u32_basic() {
        assert_eq!(u32_to_str(0), "0");
        assert_eq!(u32_to_str(1), "1");
        assert_eq!(u32_to_str(42), "42");
        assert_eq!(u32_to_str(4294967295), "4294967295");
    }
    #[test]
    fn i32_basic() {
        assert_eq!(i32_to_str(0), "0");
        assert_eq!(i32_to_str(-1), "-1");
        assert_eq!(i32_to_str(123), "123");
        assert_eq!(i32_to_str(-2147483648), "-2147483648");
    }
    #[test]
    fn u64_basic() {
        assert_eq!(u64_to_str(0), "0");
        assert_eq!(u64_to_str(18446744073709551615), "18446744073709551615");
    }
}
