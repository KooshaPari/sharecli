//! CRC-8 / CRC-16 / CRC-32 checksums (forward-only, table-free implementations).
//!
//! Three CRC variants covering the common embedded / protocol use cases:
//! - [`crc8`]   — polynomial 0x07 (ATM HEC, SMBus, many 1-Wire devices)
//! - [`crc16_ccitt`] — polynomial 0x1021, init 0xFFFF (XMODEM, Bluetooth HCI,
//!   many serial protocols)
//! - [`crc32_ieee`] — polynomial 0xEDB88320 (Ethernet, gzip, PNG, zip — often
//!   called "CRC-32" without qualification)
//!
//! All implementations use the bit-serial algorithm — slow but correct
//! without a 256-entry table. For high-throughput use, callers should
//! swap to a table-driven implementation. None of these functions
//! support running checksums across multiple buffers.

/// CRC-8 with polynomial 0x07 (LSB-first, no reflection).
///
/// Suitable for SMBus PEC, 1-Wire CRC, and other byte-oriented integrity
/// checks. Returns the CRC over `data` starting from `init` (default 0).
pub fn crc8(data: &[u8], init: u8) -> u8 {
    let mut crc = init;
    for &b in data {
        crc ^= b;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// CRC-16-CCITT (polynomial 0x1021, init 0xFFFF, no reflection).
///
/// Used by XMODEM, Bluetooth HCI CMD/EVT, and many older serial protocols.
/// Pass `init = 0xFFFF` for the standard XMODEM variant, or `0x0000` for
/// the "false" CCITT variant.
pub fn crc16_ccitt(data: &[u8], init: u16) -> u16 {
    let mut crc = init;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// CRC-32-IEEE (polynomial 0xEDB88320, init 0xFFFFFFFF, finalize XOR 0xFFFFFFFF).
///
/// The same algorithm used by `zlib`, PNG chunks, gzip trailers, and
/// Ethernet frames. The init/finalize inversion is applied internally so
/// callers can pass raw data and get the canonical CRC-32 value.
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc8_known_vectors() {
        // Standard CRC-8 (poly 0x07) test vector: "123456789" -> 0xF4
        assert_eq!(crc8(b"123456789", 0), 0xF4);
    }

    #[test]
    fn crc8_empty_with_zero_init() {
        assert_eq!(crc8(b"", 0), 0);
    }

    #[test]
    fn crc8_continuous() {
        // Verify that crc(a) followed by crc(a)(init=result) on b == crc(a+b)
        let a = b"hello";
        let b = b"world";
        let mut all = Vec::new();
        all.extend_from_slice(a);
        all.extend_from_slice(b);
        let crc_a = crc8(a, 0);
        let crc_all = crc8(&all, 0);
        let crc_b_after_a = crc8(b, crc_a);
        assert_eq!(crc_all, crc_b_after_a);
    }

    #[test]
    fn crc16_ccitt_xmodem_vector() {
        // XMODEM CRC-16 CCITT (poly 0x1021, init 0x0000) on "123456789" -> 0x31C3
        // Note: standard XMODEM uses init 0x0000 not 0xFFFF here.
        assert_eq!(crc16_ccitt(b"123456789", 0x0000), 0x31C3);
    }

    #[test]
    fn crc16_ccitt_false_variant() {
        // The "false" CCITT variant uses init 0xFFFF: same input -> different value
        assert_ne!(crc16_ccitt(b"123456789", 0xFFFF), 0x31C3);
    }

    #[test]
    fn crc32_ieee_known_vector() {
        // Standard CRC-32 IEEE 802.3 on "123456789" -> 0xCBF43926
        assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn crc32_ieee_empty_input() {
        // CRC-32 of empty input (after init/finalize inversion) -> 0
        assert_eq!(crc32_ieee(b""), 0);
    }

    #[test]
    fn crc32_ieee_distinct_for_distinct_input() {
        assert_ne!(crc32_ieee(b"hello"), crc32_ieee(b"world"));
    }
}
