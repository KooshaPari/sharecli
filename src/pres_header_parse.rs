// Minimal Microsoft Office PowerPoint legacy presentation file (.ppt) header parser.
//
// The legacy PowerPoint binary format (.ppt) is a Compound File Binary (CFB)
// container — same as old .doc Word files and legacy Outlook message stores.
// At the root of every CFB there is a 512-byte header that begins with a
// fixed 8-byte magic and 24 bytes of reserved / header-extension data.
//
// Per [MS-CFB] section 2.2 (Compound File Binary File Format):
//
//   bytes  0..8    : 0xD0CF11E0A1B11AE1 (signature)
//   bytes  8..24   : CLSID (all zeros for legacy CFB)
//   bytes 24..26   : minor version (u16 LE) — 0x003E for v4 (legacy PPT 97-2003)
//   bytes 26..28   : major version (u16 LE) — 0x0003 for v3, 0x0004 for v4
//   bytes 28..30   : byte order (u16 LE) — 0xFFFE little-endian
//   bytes 30..32   : sector shift (u16 LE) — 9 for v3 (512-byte sectors), 12 for v4 (4096-byte sectors)
//   bytes 32..34   : mini sector shift (u16 LE) — 6 (64-byte mini sectors)
//   bytes 34..40   : reserved (6 bytes)
//   bytes 40..44   : number of directory sectors (u32 LE) — 0 for v3
//   bytes 44..48   : number of FAT sectors (u32 LE)
//   bytes 48..52   : first directory sector location (u32 LE)
//   bytes 52..56   : transaction signature (u32 LE) — 0
//   bytes 56..60   : mini stream cutoff (u32 LE) — 0x00001000 (4096)
//   bytes 60..64   : first mini FAT sector location (u32 LE)
//   bytes 64..68   : number of mini FAT sectors (u32 LE)
//   bytes 68..72   : first DIFAT sector location (u32 LE)
//   bytes 72..76   : number of DIFAT sectors (u32 LE)
//   bytes 76..512  : DIFAT (109 * u32 LE) — first 109 FAT sector locations
//
// Note that this module returns ONLY the magic + a subset of the file-level
// fields sufficient to identify a legacy PPT and decide whether the file is
// encrypted (a feature introduced by PowerPoint 2007 and signaled via the
// document encryption flag at the application-stream level). The CFB header
// itself does NOT carry an encryption bit; this module reports `is_encrypted`
// based on a downstream convention (encryption is indicated by the application
// stream, not the CFB). For full PowerPoint semantic parsing, see [MS-PPT].
//
// References:
//   - [MS-CFB]  Compound File Binary File Format
//   - [MS-PPT]  PowerPoint Binary File Format (legacy `.ppt`)

/// Parsed legacy PPT (CFB) header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresHeader {
    /// The 8-byte CFB magic, exactly `[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]`.
    pub magic: [u8; 8],
    /// The CFB header is always 512 bytes; we report `header_size = 512`.
    pub header_size: u32,
    /// Sector shift (`u16` LE at offset 30). `9` for CFB v3 (PowerPoint 97-2003),
    /// `12` for CFB v4. Implies sector size = `1 << sector_shift`.
    pub total_slots: u16,
    /// CFB major version (`u16` LE at offset 26). `3` or `4`.
    pub version: u16,
    /// Number of FAT sectors (`u32` LE at offset 44). Reported for diagnostic use;
    /// `is_encrypted` is set conservatively based on the application-stream
    /// convention (always false from the CFB header alone — this field is reserved
    /// for callers that re-check the document encryption flag).
    pub is_encrypted: bool,
}

/// The 8-byte CFB magic ([MS-CFB] 2.2).
pub const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Legacy PPT files use CFB v3 (sector shift 9 = 512-byte sectors).
pub const CFB_V3_SECTOR_SHIFT: u16 = 9;
/// Some legacy containers use CFB v4 (sector shift 12 = 4096-byte sectors).
pub const CFB_V4_SECTOR_SHIFT: u16 = 12;

/// Parse the 512-byte legacy PPT (CFB) header.
///
/// `input` must be at least 512 bytes. Returns the magic, sector shift, version,
/// and a conservative `is_encrypted = false` flag — the CFB header itself does
/// not carry an encryption bit (per [MS-CFB] 2.2); callers wishing to detect
/// document-level encryption must read the document summary stream.
pub fn parse(input: &[u8]) -> Result<PresHeader, String> {
    if input.len() < 512 {
        return Err(format!("PPT header requires 512 bytes, got {}", input.len()));
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&input[0..8]);
    if magic != CFB_MAGIC {
        return Err(format!("invalid CFB magic: expected {:02X?}, got {:02X?}", CFB_MAGIC, magic));
    }
    // CFB header fields, all little-endian per [MS-CFB] 2.2.
    let major_version = u16::from_le_bytes([input[26], input[27]]);
    let byte_order = u16::from_le_bytes([input[28], input[29]]);
    if byte_order != 0xFFFE {
        return Err(format!(
            "invalid CFB byte-order mark: expected 0xFFFE, got 0x{:04X}",
            byte_order
        ));
    }
    let sector_shift = u16::from_le_bytes([input[30], input[31]]);
    let mini_sector_shift = u16::from_le_bytes([input[32], input[33]]);
    if mini_sector_shift != 6 {
        return Err(format!(
            "invalid CFB mini-sector shift: expected 6, got {}",
            mini_sector_shift
        ));
    }
    Ok(PresHeader {
        magic,
        header_size: 512,
        total_slots: sector_shift,
        version: major_version,
        is_encrypted: false,
    })
}

/// Compute the CFB sector size from a sector shift. `1 << sector_shift`.
pub fn sector_size(sector_shift: u16) -> usize {
    1usize << (sector_shift as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cfb_header(major_version: u16, sector_shift: u16) -> [u8; 512] {
        let mut buf = [0u8; 512];
        // Magic.
        buf[0..8].copy_from_slice(&CFB_MAGIC);
        // CLSID (16 bytes) at offset 8 — leave zeros.
        // Minor version (offset 24) — leave 0.
        // Major version (offset 26).
        buf[26..28].copy_from_slice(&major_version.to_le_bytes());
        // Byte order (offset 28) — 0xFFFE little-endian.
        buf[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
        // Sector shift (offset 30).
        buf[30..32].copy_from_slice(&sector_shift.to_le_bytes());
        // Mini sector shift (offset 32) — 6.
        buf[32..34].copy_from_slice(&6u16.to_le_bytes());
        // Reserved 6 bytes (offset 34..40) — leave zeros.
        // Number of directory sectors (offset 40) — 0 for v3.
        // Number of FAT sectors (offset 44) — 1 for tests.
        buf[44..48].copy_from_slice(&1u32.to_le_bytes());
        // First directory sector (offset 48) — 0.
        // Transaction sig (offset 52) — 0.
        // Mini stream cutoff (offset 56) — 0x1000.
        buf[56..60].copy_from_slice(&0x00001000u32.to_le_bytes());
        // First mini FAT (offset 60) — 0xFFFFFFFE (ENDOFCHAIN).
        buf[60..64].copy_from_slice(&0xFFFFFFFEu32.to_le_bytes());
        // Number of mini FAT (offset 64) — 0.
        // First DIFAT (offset 68) — 0.
        // Number of DIFAT (offset 72) — 0.
        // DIFAT (offset 76..512) — leave zeros.
        buf
    }

    // ---- CFB magic constant ----

    #[test]
    fn cfb_magic_matches_ms_cfb_spec() {
        // [MS-CFB] 2.2 specifies the 8-byte signature.
        assert_eq!(CFB_MAGIC, [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    }

    // ---- parse ----

    #[test]
    fn parse_v3_ppt_header_round_trip() {
        let buf = build_cfb_header(3, CFB_V3_SECTOR_SHIFT);
        let h = parse(&buf).unwrap();
        assert_eq!(h.magic, CFB_MAGIC);
        assert_eq!(h.header_size, 512);
        assert_eq!(h.total_slots, 9);
        assert_eq!(h.version, 3);
        assert!(!h.is_encrypted);
    }

    #[test]
    fn parse_v4_header_accepts_4096_byte_sectors() {
        let buf = build_cfb_header(4, CFB_V4_SECTOR_SHIFT);
        let h = parse(&buf).unwrap();
        assert_eq!(h.total_slots, 12);
        assert_eq!(h.version, 4);
    }

    #[test]
    fn parse_rejects_short_input() {
        let short = vec![0u8; 100];
        let err = parse(&short).unwrap_err();
        assert!(err.contains("512"), "got: {}", err);
    }

    #[test]
    fn parse_rejects_bad_magic() {
        let mut buf = build_cfb_header(3, CFB_V3_SECTOR_SHIFT);
        buf[0] = 0xAB;
        let err = parse(&buf).unwrap_err();
        assert!(err.contains("magic"), "got: {}", err);
    }

    #[test]
    fn parse_rejects_bad_byte_order() {
        let mut buf = build_cfb_header(3, CFB_V3_SECTOR_SHIFT);
        buf[28] = 0x00;
        let err = parse(&buf).unwrap_err();
        assert!(err.contains("byte-order"), "got: {}", err);
    }

    #[test]
    fn parse_rejects_bad_mini_sector_shift() {
        let mut buf = build_cfb_header(3, CFB_V3_SECTOR_SHIFT);
        // Set mini sector shift to 7 (invalid per [MS-CFB]).
        buf[32..34].copy_from_slice(&7u16.to_le_bytes());
        let err = parse(&buf).unwrap_err();
        assert!(err.contains("mini-sector"), "got: {}", err);
    }

    #[test]
    fn sector_size_matches_shift() {
        assert_eq!(sector_size(9), 512);
        assert_eq!(sector_size(12), 4096);
    }

    #[test]
    fn parse_accepts_extra_bytes_after_header() {
        // Real files have at least 512 bytes; extra is fine.
        let mut buf = build_cfb_header(3, CFB_V3_SECTOR_SHIFT).to_vec();
        buf.extend_from_slice(&[0u8; 100]);
        let h = parse(&buf).unwrap();
        assert_eq!(h.header_size, 512);
    }
}
