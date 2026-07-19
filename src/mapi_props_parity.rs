// MAPI property stream parity utilities.
//
// This module is the BUILD/VERIFY counterpart to `mapi_props`, which is parse-only.
// The goal here is round-trip build+verify for a single MAPI property stream
// record so callers (PST/OST tooling, MAPI test harnesses) can synthesize
// streams without touching the on-disk producer.
//
// On-disk MAPI property record layout (per [MS-OXPROPS], [MS-OXCSTOR]):
//
//   [tag: u32 LE | tag: u64 LE]  [flags: u16 LE]  [len: u32 LE]  [value: len bytes]
//
// For the 16-bit (legacy ANSI) tag layout, the tag is 4 bytes on the wire and
// is interpreted as `(id_high_u16 | type_low_u16)`. For the 32-bit (Unicode)
// tag layout, the tag is 8 bytes on the wire — this parity module always
// emits BOTH 4-byte halves equal to the packed `tag`, matching the
// `prop_bytes_32bit` test helper in `mapi_props.rs`.
//
// References:
//   - [MS-OXPROPS]: Property Definitions (tag layout, top 16 bits = id)
//   - [MS-OXCSTOR]: Personal Folder File Structure

/// Build a single MAPI property record (16-bit tag layout, as_32bit = false).
///
/// Wire format:
///
///   [tag: u32 LE] [flags: u16 LE] [len: u32 LE] [value: len bytes]
///
/// `tag` is a packed MAPI tag (low 16 bits = property type, high 16 bits = id).
/// `flags` is the per-property flags field (typically 0).
/// `value` is the property payload; if `len(value)` exceeds `u32::MAX` it is
/// truncated at `u32::MAX` bytes (callers wanting full control should chunk
/// before calling).
pub fn build_entry(tag: u32, flags: u16, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 2 + 4 + value.len());
    out.extend_from_slice(&tag.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    let len = value.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    out
}

/// Build a single MAPI property record using the 32-bit (Unicode) tag layout.
///
/// Wire format:
///
///   [tag_half1: u32 LE] [tag_half2: u32 LE] [flags: u16 LE] [len: u32 LE] [value: len bytes]
///
/// For parity testing we emit both halves equal to the packed `tag`, matching
/// the layout used by `mapi_props::prop_bytes_32bit`. Real Unicode streams
/// carry distinct halves; this helper is appropriate for synthetic test
/// fixtures only.
pub fn build_entry_32bit(tag: u32, flags: u16, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 2 + 4 + value.len());
    out.extend_from_slice(&tag.to_le_bytes());
    out.extend_from_slice(&tag.to_le_bytes()); // second half identical for parity
    out.extend_from_slice(&flags.to_le_bytes());
    let len = value.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    out
}

/// Verify that a built entry round-trips through the parse function and
/// yields the same (tag, flags, value).
///
/// This relies on `mapi_props::parse` to do the structural decoding. The
/// assertion is strict: tag, flags, and value must all match exactly.
pub fn assert_round_trip(tag: u32, flags: u16, value: &[u8]) {
    let bytes = build_entry(tag, flags, value);
    let props = crate::util::mapi_props::parse(&bytes, false).expect("re-parse must succeed");
    assert_eq!(props.len(), 1, "expected 1 record, got {}", props.len());
    let p = &props[0];
    assert_eq!(p.tag, tag, "tag mismatch: 0x{:08x} != 0x{:08x}", p.tag, tag);
    assert_eq!(p.flags, flags, "flags mismatch: {} != {}", p.flags, flags);
    assert_eq!(p.value, value, "value mismatch");
}

/// Verify round-trip for the 32-bit (Unicode) tag layout.
pub fn assert_round_trip_32bit(tag: u32, flags: u16, value: &[u8]) {
    let bytes = build_entry_32bit(tag, flags, value);
    let props = crate::util::mapi_props::parse(&bytes, true).expect("re-parse must succeed");
    assert_eq!(props.len(), 1, "expected 1 record, got {}", props.len());
    let p = &props[0];
    assert_eq!(p.tag, tag, "tag mismatch: 0x{:08x} != 0x{:08x}", p.tag, tag);
    assert_eq!(p.flags, flags, "flags mismatch: {} != {}", p.flags, flags);
    assert_eq!(p.value, value, "value mismatch");
}

/// Build a multi-property stream by concatenating 16-bit-tagged entries.
pub fn build_stream(entries: &[(u32, u16, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for &(tag, flags, value) in entries {
        out.extend_from_slice(&build_entry(tag, flags, value));
    }
    out
}

/// Build a multi-property stream with the 32-bit (Unicode) tag layout.
pub fn build_stream_32bit(entries: &[(u32, u16, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for &(tag, flags, value) in entries {
        out.extend_from_slice(&build_entry_32bit(tag, flags, value));
    }
    out
}

/// Verify a multi-property stream built by `build_stream` parses back to
/// the same entries (in the same order).
pub fn assert_stream_round_trip(entries: &[(u32, u16, &[u8])]) {
    let bytes = build_stream(entries);
    let props = crate::util::mapi_props::parse(&bytes, false).expect("re-parse must succeed");
    assert_eq!(props.len(), entries.len(), "entry count mismatch");
    for (i, (tag, flags, value)) in entries.iter().enumerate() {
        assert_eq!(props[i].tag, *tag, "entry {i}: tag mismatch");
        assert_eq!(props[i].flags, *flags, "entry {i}: flags mismatch");
        assert_eq!(props[i].value, *value, "entry {i}: value mismatch");
    }
}

/// Same as `assert_stream_round_trip` but for the 32-bit (Unicode) layout.
pub fn assert_stream_round_trip_32bit(entries: &[(u32, u16, &[u8])]) {
    let bytes = build_stream_32bit(entries);
    let props = crate::util::mapi_props::parse(&bytes, true).expect("re-parse must succeed");
    assert_eq!(props.len(), entries.len(), "entry count mismatch");
    for (i, (tag, flags, value)) in entries.iter().enumerate() {
        assert_eq!(props[i].tag, *tag, "entry {i}: tag mismatch");
        assert_eq!(props[i].flags, *flags, "entry {i}: flags mismatch");
        assert_eq!(props[i].value, *value, "entry {i}: value mismatch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- build_entry ----

    #[test]
    fn build_entry_wire_layout_is_exact() {
        // tag = 0x0037001f (id=0x0037 subject, type=0x001f PT_UNICODE), flags=0, value=b"hi"
        let tag = 0x0037_001fu32;
        let bytes = build_entry(tag, 0, b"hi");
        assert_eq!(
            &bytes[..],
            &[0x1f, 0x00, 0x37, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, b'h', b'i'],
            "wire format must be LE: tag(4)|flags(2)|len(4)|value"
        );
    }

    #[test]
    fn build_entry_empty_value_emits_zero_length() {
        let bytes = build_entry(0x0001_0002, 0, b"");
        // 4 + 2 + 4 = 10 bytes, len = 0
        assert_eq!(bytes.len(), 10);
        assert_eq!(&bytes[6..10], &[0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn build_entry_with_flags_non_zero() {
        let tag = 0x0102_0002u32; // PT_BINARY id=0x0002
        let bytes = build_entry(tag, 0xCAFE, &[1, 2, 3, 4]);
        // flags as u16 LE
        assert_eq!(&bytes[4..6], &[0xFE, 0xCA]);
        // len = 4
        assert_eq!(&bytes[6..10], &[0x04, 0x00, 0x00, 0x00]);
        assert_eq!(&bytes[10..], &[1, 2, 3, 4]);
    }

    // ---- assert_round_trip ----

    #[test]
    fn round_trip_subject_string() {
        // PidTagSubject = 0x0037 (PT_UNICODE = 0x001f) per [MS-OXPROPS].
        let tag = crate::util::mapi_props::pack_tag(0x001f, 0x0037);
        assert_round_trip(tag, 0, "Hello, world!".as_bytes());
    }

    #[test]
    fn round_trip_with_non_zero_flags() {
        let tag = crate::util::mapi_props::pack_tag(0x0102, 0x0002);
        assert_round_trip(tag, 0xCAFE, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn round_trip_empty_value() {
        let tag = crate::util::mapi_props::pack_tag(0x001f, 0x0037);
        assert_round_trip(tag, 0, b"");
    }

    #[test]
    fn round_trip_long_value() {
        let tag = crate::util::mapi_props::pack_tag(0x0102, 0x0002);
        let value = vec![0xAAu8; 1024];
        assert_round_trip(tag, 0, &value);
    }

    // ---- build_entry_32bit + assert_round_trip_32bit ----

    #[test]
    fn build_entry_32bit_tag_layout_repeats_halves() {
        let tag = 0xAABB_CCDDu32;
        let bytes = build_entry_32bit(tag, 0, b"x");
        // First 4 = tag, next 4 = tag (parity fixture repeat)
        assert_eq!(&bytes[0..4], &tag.to_le_bytes());
        assert_eq!(&bytes[4..8], &tag.to_le_bytes());
        // Then flags(2) = 0, len(2) = 1
        assert_eq!(&bytes[8..10], &[0x00, 0x00]);
        assert_eq!(&bytes[10..14], &[0x01, 0x00, 0x00, 0x00]);
        assert_eq!(bytes[14], b'x');
    }

    #[test]
    fn round_trip_32bit_subject() {
        let tag = crate::util::mapi_props::pack_tag(0x001f, 0x0037);
        assert_round_trip_32bit(tag, 0, "hi".as_bytes());
    }

    // ---- multi-property streams ----

    #[test]
    fn stream_round_trip_two_entries() {
        let entries: &[(u32, u16, &[u8])] = &[
            (crate::util::mapi_props::pack_tag(0x001f, 0x0037), 0, b"hello".as_slice()),
            (crate::util::mapi_props::pack_tag(0x0003, 0x0e08), 1, &42i32.to_le_bytes()),
        ];
        assert_stream_round_trip(entries);
    }

    #[test]
    fn stream_round_trip_32bit_three_entries() {
        let a = crate::util::mapi_props::pack_tag(0x001f, 0x0037);
        let b = crate::util::mapi_props::pack_tag(0x0102, 0x0002);
        let c = crate::util::mapi_props::pack_tag(0x000b, 0x0001);
        let entries: &[(u32, u16, &[u8])] =
            &[(a, 0, b"a".as_slice()), (b, 1, &[0xFFu8]), (c, 2, &[1u8])];
        assert_stream_round_trip_32bit(entries);
    }

    #[test]
    fn stream_round_trip_empty() {
        assert_stream_round_trip(&[]);
    }
}
