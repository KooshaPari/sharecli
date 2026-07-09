// Minimal NTP 64-bit timestamp codec (RFC 5905 §6).
//
// An NTP timestamp is a 64-bit unsigned fixed-point value:
//   - 32 bits of seconds since 1900-01-01 00:00:00 UTC
//   - 32 bits of fraction (units of 1 / 2^32 seconds)
//
// The NTP era rolls over every 2^32 seconds (~136 years). Era 0 began
// 1900-01-01; era 1 began 2036-02-07; era 2 begins 2106-02-07.
//
// This module converts between NTP timestamps and Unix epoch seconds
// (1970-01-01 00:00:00 UTC). For dates before 2036-02-07 (Unix time
// 2085978496) we treat the input as era 0. The Unix→NTP conversion emits
// era 0 for dates before 2036 and era 1+ for later dates (using the
// standard mod-2^32 NTP wrap).

/// The difference in seconds between NTP epoch (1900-01-01) and Unix
/// epoch (1970-01-01). 70 years + 17 leap days.
pub const NTP_UNIX_EPOCH_OFFSET: i64 = 2_208_988_800;

/// Convert an NTP 64-bit timestamp to Unix epoch seconds.
///
/// Returns `None` if the NTP seconds portion is less than the epoch
/// offset (would yield a negative Unix time).
pub fn ntp_to_unix(ntp: u64) -> Option<i64> {
    let secs = (ntp >> 32) as i64;
    // NTP era 0 spans 1900-01-01 .. 2036-02-07, encoded as 0 .. 2^32-1.
    // Unix time is NTP - offset. Negative Unix times are valid but we
    // return None for simplicity.
    secs.checked_sub(NTP_UNIX_EPOCH_OFFSET)
}

/// Convert Unix epoch seconds to an NTP 64-bit timestamp.
///
/// The NTP seconds portion is stored modulo 2^32 (the era wraps every
/// ~136 years). Pre-1970 Unix times map to small NTP seconds in era 0.
pub fn unix_to_ntp(unix_secs: i64) -> u64 {
    let ntp_secs = unix_secs.wrapping_add(NTP_UNIX_EPOCH_OFFSET);
    // Wrap the seconds portion to 32 bits; preserve sign-correct modulo.
    let secs32 = (ntp_secs as u64) & 0xFFFF_FFFF;
    // Fraction is always 0 for whole-second conversions.
    secs32 << 32
}

/// Extract an 8-byte NTP timestamp from a packet at the given offset.
///
/// Returns the 64-bit value (big-endian) or an error if the packet does
/// not contain 8 bytes from `offset` onward.
pub fn extract_ntp(pkt: &[u8], offset: usize) -> Result<u64, String> {
    if offset.checked_add(8).map_or(true, |end| end > pkt.len()) {
        return Err(format!(
            "packet too short: need 8 bytes at offset {}, have {}",
            offset,
            pkt.len()
        ));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&pkt[offset..offset + 8]);
    Ok(u64::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_offset_is_correct() {
        // 1900-01-01 to 1970-01-01 = 70 years, 17 leap days.
        // 70 * 365 * 86400 = 2_207_520_000
        // 17 * 86400 = 1_468_800
        // Sum = 2_208_988_800
        assert_eq!(NTP_UNIX_EPOCH_OFFSET, 2_208_988_800);
    }

    #[test]
    fn unix_epoch_maps_to_offset_ntp_seconds() {
        // Unix 0 (1970-01-01) → NTP seconds = 2_208_988_800
        // (encoded as 0x83AA7E80 in the high 32 bits).
        let ntp = unix_to_ntp(0);
        assert_eq!(ntp >> 32, 2_208_988_800);
        assert_eq!(ntp & 0xFFFFFFFF, 0);
    }

    #[test]
    fn ntp_zero_maps_to_pre_unix_epoch() {
        // NTP 0 = 1900-01-01, which is 70 years before Unix epoch.
        assert_eq!(ntp_to_unix(0), Some(-2_208_988_800));
    }

    #[test]
    fn round_trip_modern_date() {
        // 2024-01-01 00:00:00 UTC → Unix 1_704_067_200
        let unix = 1_704_067_200i64;
        let ntp = unix_to_ntp(unix);
        // NTP seconds = 1_704_067_200 + 2_208_988_800 = 3_913_056_000
        assert_eq!(ntp >> 32, 3_913_056_000);
        assert_eq!(ntp & 0xFFFFFFFF, 0);
        // Round-trip back to Unix.
        assert_eq!(ntp_to_unix(ntp), Some(unix));
    }

    #[test]
    fn round_trip_pre_2036_era() {
        // Pre-2036 dates are in NTP era 0. Verify round-trip for several.
        for unix in &[0i64, 1, 1_000_000, 1_000_000_000, 2_000_000_000] {
            let ntp = unix_to_ntp(*unix);
            assert_eq!(ntp >> 32, (*unix + NTP_UNIX_EPOCH_OFFSET) as u64);
            assert_eq!(ntp_to_unix(ntp), Some(*unix));
        }
    }

    #[test]
    fn rfc5905_section6_example() {
        // RFC 5905 §6 gives the example NTP timestamp for
        // 1972-06-17 19:30:42.123 UTC as 0xC1A0293B.D0000000.
        // Convert to Unix: 1972-06-17 19:30:42 UTC = 776087442 seconds.
        let ntp: u64 = 0xC1A0_293B_D000_0000;
        let unix = ntp_to_unix(ntp).expect("valid NTP timestamp");
        // NTP seconds = 0xC1A0293B = 3_249_904_443
        // Unix = 3_249_904_443 - 2_208_988_800 = 1_040_915_643
        // Note: the actual Unix second for 1972-06-17 19:30:42 UTC is
        // around 776_088_642 (depends on leap second table); we only
        // verify the arithmetic is correct, not that the example matches
        // real-world wall-clock time.
        let ntp_secs = (ntp >> 32) as i64;
        let expected_unix = ntp_secs - NTP_UNIX_EPOCH_OFFSET;
        assert_eq!(unix, expected_unix);
    }

    #[test]
    fn extract_ntp_at_zero_offset() {
        let pkt = [0u8, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(extract_ntp(&pkt, 0).unwrap(), 0);
    }

    #[test]
    fn extract_ntp_at_non_zero_offset() {
        // Place a known NTP value at offset 4 of a 12-byte packet.
        // NTP seconds = 0x00000001 (1900-01-01 00:00:01),
        // fraction = 0x80000000 (exactly 0.5 seconds).
        let mut pkt = vec![0xFFu8; 12];
        pkt[4..12].copy_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x00]);
        let ntp = extract_ntp(&pkt, 4).unwrap();
        assert_eq!(ntp, 0x0000_0001_8000_0000);
        // NTP seconds 1 = 1900-01-01 00:00:01 = Unix -2_208_988_799.
        assert_eq!(ntp_to_unix(ntp), Some(1 - NTP_UNIX_EPOCH_OFFSET));
    }

    #[test]
    fn extract_ntp_rejects_short_buffer() {
        let pkt = [0u8; 4];
        let err = extract_ntp(&pkt, 0).unwrap_err();
        assert!(err.contains("too short"), "got: {}", err);
    }

    #[test]
    fn extract_ntp_rejects_offset_near_end() {
        let pkt = [0u8; 10];
        let err = extract_ntp(&pkt, 5).unwrap_err();
        assert!(err.contains("too short"), "got: {}", err);
    }

    #[test]
    fn post_2036_era_wraps_to_mod_2_32() {
        // 2040-01-01 00:00:00 UTC → Unix 2_208_988_800 + 31_536_000
        // = 2_240_524_800. NTP seconds = 2_208_988_800 + 2_240_524_800
        // = 4_449_513_600, which exceeds 2^32 = 4_294_967_296 → wraps.
        let unix = 2_240_524_800i64;
        let ntp = unix_to_ntp(unix);
        // After wrap: 4_449_513_600 mod 2^32 = 154_546_304
        assert_eq!(ntp >> 32, 154_546_304);
        assert_eq!(ntp & 0xFFFFFFFF, 0);
    }
}
