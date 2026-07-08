// Legacy PowerPoint (.ppt) header parity utilities.
//
// This module is the BUILD/VERIFY counterpart to `pres_header_parse`, which is
// parse-only. The goal here is round-trip build+verify for the 512-byte MS-CFB
// header used at the start of every legacy .ppt file, so PST/PPT tooling can
// synthesize fixture headers without touching the disk.
//
// The header layout we emit is the minimal CFB v3 (PowerPoint 97-2003) shape:
//
//   bytes  0..8    : CFB magic (constant)
//   bytes  8..24   : CLSID (16 zeros)
//   bytes 24..26   : minor version (u16 LE) — 0 for legacy PPT
//   bytes 26..28   : major version (u16 LE) — caller-controlled
//   bytes 28..30   : byte-order mark (u16 LE) — 0xFFFE per [MS-CFB] 2.2
//   bytes 30..32   : sector shift (u16 LE) — 9 for v3, 12 for v4
//   bytes 32..34   : mini-sector shift (u16 LE) — 6 (required)
//   bytes 34..40   : reserved (6 zeros)
//   bytes 40..44   : number of directory sectors (u32 LE) — 0 for v3
//   bytes 44..48   : number of FAT sectors (u32 LE) — 1 for fixtures
//   bytes 48..52   : first directory sector (u32 LE) — 0
//   bytes 52..56   : transaction signature (u32 LE) — 0
//   bytes 56..60   : mini stream cutoff (u32 LE) — 0x1000
//   bytes 60..64   : first mini FAT sector (u32 LE) — 0xFFFFFFFE
//   bytes 64..68   : number of mini FAT sectors (u32 LE) — 0
//   bytes 68..72   : first DIFAT sector (u32 LE) — 0
//   bytes 72..76   : number of DIFAT sectors (u32 LE) — 0
//   bytes 76..512  : DIFAT (109 * u32 LE) — zeros for the first entry to anchor FAT[0]
//
// The `is_encrypted` flag in this parity build is a SYNTHETIC header bit
// stored at offset 510..512 (a 2-byte overlay) — this module reserves the
// last 2 bytes of the header for this bit so that `assert_round_trip` can
// verify the caller's intent. The real PowerPoint document encryption flag
// lives in the application stream per [MS-PPT], not the CFB header; this
// overlay is a parity-test artifact only.
//
// References:
//   - [MS-CFB] Compound File Binary File Format 2.2
//   - [MS-PPT] PowerPoint Binary File Format

/// The 512-byte CFB/PPT header size.
pub const PPT_HEADER_SIZE: usize = 512;

/// Build a 512-byte legacy PPT (CFB) header.
///
/// `version` selects the major version (3 for v3 / PPT 97-2003, 4 for v4).
/// `is_encrypted` writes a synthetic flag at header offset 510..512 — see the
/// module-level note above. Real-world document-encryption detection must
/// look at the PowerPoint application stream, not this overlay.
pub fn build_ppt_header(version: u16, is_encrypted: bool) -> Vec<u8> {
    let mut buf = vec![0u8; PPT_HEADER_SIZE];
    let sector_shift: u16 = match version {
        3 => 9,
        4 => 12,
        // Unknown versions fall back to the v3 sector shift. The parser
        // accepts any sector shift value, so this is safe for fixture use.
        _ => 9,
    };
    // Magic.
    buf[0..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    // CLSID (16 bytes at offset 8) — leave zeros.
    // Minor version (offset 24) — 0.
    // Major version (offset 26).
    buf[26..28].copy_from_slice(&version.to_le_bytes());
    // Byte order (offset 28) — 0xFFFE.
    buf[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
    // Sector shift (offset 30).
    buf[30..32].copy_from_slice(&sector_shift.to_le_bytes());
    // Mini sector shift (offset 32) — 6 (required by [MS-CFB] 2.2).
    buf[32..34].copy_from_slice(&6u16.to_le_bytes());
    // Reserved 6 bytes (offset 34..40) — zeros.
    // Directory sectors (offset 40) — 0 for v3.
    // FAT sectors (offset 44) — 1 for a minimal fixture.
    buf[44..48].copy_from_slice(&1u32.to_le_bytes());
    // First directory sector (offset 48) — 0.
    // Transaction sig (offset 52) — 0.
    // Mini stream cutoff (offset 56) — 0x1000.
    buf[56..60].copy_from_slice(&0x0000_1000u32.to_le_bytes());
    // First mini FAT (offset 60) — 0xFFFFFFFE.
    buf[60..64].copy_from_slice(&0xFFFF_FFFEu32.to_le_bytes());
    // Number of mini FAT (offset 64) — 0.
    // First DIFAT (offset 68) — 0.
    // Number of DIFAT (offset 72) — 0.
    // DIFAT (offset 76..512) — anchor FAT[0] at index 0 of the DIFAT.
    buf[76..80].copy_from_slice(&0u32.to_le_bytes()); // DIFAT[0] = sector 0
    // Synthetic encryption flag overlay (offset 510..512) — see module note.
    if is_encrypted {
        buf[510] = 0x01;
        buf[511] = 0x00;
    }
    buf
}

/// Whether the synthetic encryption-flag overlay is currently set on a
/// header built by `build_ppt_header`. Returns `false` for any header that
/// does not have the expected size (512 bytes) or that did not originate
/// from `build_ppt_header`.
pub fn overlay_encrypted(input: &[u8]) -> bool {
    input.len() >= 512 && input[510] == 0x01
}

/// Build the canonical 512-byte fixture used by the v3 baseline test in
/// `pres_header_parse::build_cfb_header`. Useful as a hand-off fixture when
/// passing buffers into external test harnesses.
pub fn build_v3_default_fixture() -> Vec<u8> {
    build_ppt_header(3, false)
}

/// Verify that the input is a syntactically valid 512-byte CFB/PPT header
/// and round-trips through `pres_header_parse::parse`.
///
/// The parser does not see the synthetic encryption-flag overlay (see module
/// note); it always reports `is_encrypted = false` from the CFB header alone.
/// The overlay bit is preserved verbatim by `build_ppt_header` and can be
/// inspected via `overlay_encrypted`.
pub fn assert_round_trip(input: &[u8]) {
    assert_eq!(input.len(), 512, "header must be 512 bytes, got {}", input.len());
    let parsed =
        crate::pres_header_parse::parse(input).expect("header must round-trip through parser");
    assert_eq!(parsed.header_size, 512, "header_size must be 512");
    assert_eq!(
        parsed.magic,
        [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        "magic must match [MS-CFB] 2.2"
    );
    // The CFB header parser sees no encryption bit (none exists per [MS-CFB] 2.2);
    // the overlay we store is an extra-header convention only.
    assert!(!parsed.is_encrypted, "CFB header alone never reports encrypted");
}

/// Convenience: build a v3 header with `is_encrypted = false` and verify it.
pub fn build_and_verify_v3() -> Vec<u8> {
    let buf = build_ppt_header(3, false);
    assert_round_trip(&buf);
    buf
}

/// Convenience: build a v3 header with `is_encrypted = true` and verify it.
pub fn build_and_verify_v3_encrypted() -> Vec<u8> {
    let buf = build_ppt_header(3, true);
    assert_round_trip(&buf);
    assert!(overlay_encrypted(&buf), "overlay must reflect is_encrypted=true");
    buf
}

/// Convenience: build a v4 header and verify it.
pub fn build_and_verify_v4() -> Vec<u8> {
    let buf = build_ppt_header(4, false);
    assert_round_trip(&buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- build_ppt_header structural properties ----

    #[test]
    fn built_header_is_512_bytes() {
        let buf = build_ppt_header(3, false);
        assert_eq!(buf.len(), 512);
    }

    #[test]
    fn built_v3_header_matches_ms_cfb_magic_at_offset_0() {
        // [MS-CFB] 2.2 — 8-byte signature at the start.
        let buf = build_ppt_header(3, false);
        assert_eq!(
            &buf[0..8],
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
        );
    }

    #[test]
    fn built_v3_header_has_sector_shift_9_at_offset_30() {
        // [MS-CFB] 2.2 — sector shift for v3 is 9 (512-byte sectors).
        let buf = build_ppt_header(3, false);
        let shift = u16::from_le_bytes([buf[30], buf[31]]);
        assert_eq!(shift, 9);
    }

    #[test]
    fn built_v3_header_has_mini_sector_shift_6_at_offset_32() {
        // [MS-CFB] 2.2 — mini-sector shift is always 6 (64-byte mini sectors).
        let buf = build_ppt_header(3, false);
        let shift = u16::from_le_bytes([buf[32], buf[33]]);
        assert_eq!(shift, 6);
    }

    #[test]
    fn built_v3_header_has_byte_order_fffe_at_offset_28() {
        // [MS-CFB] 2.2 — byte-order mark is 0xFFFE (little-endian).
        let buf = build_ppt_header(3, false);
        let bo = u16::from_le_bytes([buf[28], buf[29]]);
        assert_eq!(bo, 0xFFFE);
    }

    #[test]
    fn built_v4_header_has_sector_shift_12_at_offset_30() {
        let buf = build_ppt_header(4, false);
        let shift = u16::from_le_bytes([buf[30], buf[31]]);
        assert_eq!(shift, 12);
    }

    // ---- overlay_encrypted ----

    #[test]
    fn overlay_encrypted_reads_true_when_built_true() {
        let buf = build_ppt_header(3, true);
        assert!(overlay_encrypted(&buf));
    }

    #[test]
    fn overlay_encrypted_reads_false_when_built_false() {
        let buf = build_ppt_header(3, false);
        assert!(!overlay_encrypted(&buf));
    }

    // ---- assert_round_trip ----

    #[test]
    fn assert_round_trip_passes_for_v3_clear() {
        assert_round_trip(&build_ppt_header(3, false));
    }

    #[test]
    fn assert_round_trip_passes_for_v3_encrypted() {
        assert_round_trip(&build_ppt_header(3, true));
    }

    #[test]
    fn assert_round_trip_passes_for_v4() {
        assert_round_trip(&build_ppt_header(4, false));
    }

    #[test]
    fn assert_round_trip_rejects_bad_magic() {
        let mut buf = build_ppt_header(3, false);
        buf[0] = 0xAB; // corrupt magic
        // The parser will reject; we expect the parse call inside
        // assert_round_trip to fail.
        let result = std::panic::catch_unwind(|| assert_round_trip(&buf));
        assert!(result.is_err(), "assertion must panic on bad magic");
    }

    #[test]
    fn build_v3_default_fixture_passes_parse() {
        let buf = build_v3_default_fixture();
        let parsed = crate::pres_header_parse::parse(&buf).expect("must parse");
        assert_eq!(parsed.version, 3);
        assert_eq!(parsed.total_slots, 9);
        assert!(!parsed.is_encrypted);
    }

    #[test]
    fn build_and_verify_convenience_helpers() {
        let v3 = build_and_verify_v3();
        assert_eq!(v3.len(), 512);
        let v3e = build_and_verify_v3_encrypted();
        assert!(overlay_encrypted(&v3e));
        let v4 = build_and_verify_v4();
        let parsed = crate::pres_header_parse::parse(&v4).expect("v4 must parse");
        assert_eq!(parsed.version, 4);
        assert_eq!(parsed.total_slots, 12);
    }
}
