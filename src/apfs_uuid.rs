// APFS UUID: 16 bytes displayed as 32 hex chars in canonical big-endian form.
// Same as RFC 4122 textual representation but stored as raw bytes (no variant bits enforcement).
pub struct Uuid([u8; 16]);
impl Uuid {
    pub fn from_bytes(bytes: [u8; 16]) -> Self { Self(bytes) }
    pub fn as_bytes(&self) -> &[u8; 16] { &self.0 }
    pub fn nil() -> Self { Self([0u8; 16]) }
    pub fn is_nil(&self) -> bool { self.0.iter().all(|&b| b == 0) }
    pub fn to_hex_string(&self) -> String {
        let mut s = String::with_capacity(32);
        for b in &self.0 { s.push_str(&format!("{:02x}", b)); }
        s
    }
    pub fn from_hex(s: &str) -> Result<Self, String> {
        if s.len() != 32 { return Err(format!("expected 32 hex chars, got {}", s.len())); }
        let mut out = [0u8; 16];
        for i in 0..16 {
            let pair = &s[i*2..i*2+2];
            out[i] = u8::from_str_radix(pair, 16).map_err(|_| format!("bad hex at {}", i*2))?;
        }
        Ok(Self(out))
    }
    pub fn to_hyphenated(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],
            self.0[8], self.0[9],
            self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15],
        )
    }
    pub fn from_hyphenated(s: &str) -> Result<Self, String> {
        // Strip hyphens
        let stripped: String = s.chars().filter(|&c| c != '-').collect();
        Self::from_hex(&stripped)
    }
}
impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn nil_uuid() {
        let u = Uuid::nil();
        assert!(u.is_nil());
        assert_eq!(u.to_hex_string(), "00000000000000000000000000000000");
    }
    #[test] fn round_trip_hex() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let u = Uuid::from_bytes(bytes);
        let s = u.to_hex_string();
        let back = Uuid::from_hex(&s).unwrap();
        assert_eq!(back.as_bytes(), &bytes);
    }
    #[test] fn hyphenated_round_trip() {
        let bytes = [0xab; 16];
        let u = Uuid::from_bytes(bytes);
        let h = u.to_hyphenated();
        assert_eq!(h, "abababab-abab-abab-abab-abababababab");
        let back = Uuid::from_hyphenated(&h).unwrap();
        assert_eq!(back.as_bytes(), &bytes);
    }
    #[test] fn hex_bad_length() {
        assert!(Uuid::from_hex("abc").is_err());
    }
    #[test] fn hex_bad_chars() {
        assert!(Uuid::from_hex("zzzz0000000000000000000000000000").is_err());
    }
    #[test] fn display_format() {
        let bytes = [0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb];
        let u = Uuid::from_bytes(bytes);
        assert_eq!(format!("{}", u), "deadbeef00112233445566778899aabb");
    }
    #[test] fn hyphenated_canonical_format() {
        // RFC 4122 example: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
        let u = Uuid::from_hex("6ba7b8109dad11d180b400c04fd430c8").unwrap();
        assert_eq!(u.to_hyphenated(), "6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    }
    #[test] fn from_hyphenated_with_uppercase() {
        let u = Uuid::from_hyphenated("DEADBEEF-0011-2233-4455-6677-8899AABB").unwrap();
        // Lowercase conversion: parse_hex rejects uppercase since I used from_str_radix
        // But here we strip hyphens first, so uppercase passes via from_str_radix if it's case-insensitive.
        // Wait, from_str_radix is lowercase by default. Let me test with lowercase only.
        assert_eq!(u.to_hex_string(), "deadbeef00112233445566778899aabb");
    }
}
