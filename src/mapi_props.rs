// Minimal Microsoft Outlook MAPI property stream parser.
//
// MAPI (Messaging Application Programming Interface) properties live inside
// PST/OST Personal Folder files. Two relevant property-stream formats are
// covered by [MS-OXPROPS] / [MS-OXCSTOR]:
//
//   * 16-bit (legacy, ANSI) property tag: 2-byte property type (lower)
//     followed by 2-byte property ID (higher). Little-endian throughout.
//   * 32-bit (Unicode) property tag: 4-byte property type followed by
//     4-byte property ID. Little-endian throughout.
//
// The property stream itself is a packed sequence of:
//
//   [tag: u32 | tag: u64]  [flags: u32]  [value: variable-length bytes]
//
// The on-disk encoding wraps the property's value in a per-property
// header that contains:
//
//   - the tag (2 bytes type + 2 bytes id, or 4 bytes type + 4 bytes id)
//   - the flags (4 bytes) — typically 0
//   - then either 8 bytes for fixed-size data (no length prefix) or
//     N bytes of variable data prefixed by its 4-byte length.
//
// We expose a deliberately small surface:
//
//   * `MapiProp { tag, flags, value }` — the parsed property.
//   * `parse(input, as_32bit)` — parse the whole stream into a `Vec<MapiProp>`.
//
// References:
//   - [MS-OXPROPS]: Property Definitions (tag layout, top 16 bits = type)
//   - [MS-OXCSTOR]: Personal Folder File Structure

/// A single parsed MAPI property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapiProp {
    /// 32-bit packed tag: high 16 bits = property ID, low 16 bits = property type.
    /// (For 16-bit tags, the upper 16 bits of the ID are zeros.)
    pub tag: u32,
    /// Per-property flags (usually `0`).
    pub flags: u16,
    /// The raw property value bytes. For variable-length properties this
    /// is the bytes after the length prefix; for fixed-length properties
    /// it is the raw 8-byte payload.
    pub value: Vec<u8>,
}

/// Known MAPI property types (low 16 bits of the tag).
/// Only the most common ones are enumerated; everything else maps to `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapiType {
    ShortString,
    LongString,
    Binary,
    Short,
    Long,
    Bool,
    SysTime,
    Error,
    Object,
    Currency,
    Double,
    Float,
    None,
    ClassId,
    MvShortString,
    MvLongString,
    MvBinary,
    MvShort,
    MvLong,
    MvSysTime,
    Unknown(u16),
}

impl MapiType {
    pub fn from_u16(v: u16) -> Self {
        match v {
            0x001e => MapiType::ShortString,
            0x001f => MapiType::LongString,
            0x0102 => MapiType::Binary,
            0x0002 => MapiType::Short,
            0x0003 => MapiType::Long,
            0x000b => MapiType::Bool,
            0x0040 => MapiType::SysTime,
            0x000a => MapiType::Error,
            0x000d => MapiType::Object,
            0x0006 => MapiType::Currency,
            0x0005 => MapiType::Double,
            0x0004 => MapiType::Float,
            0x0000 => MapiType::None,
            0x0048 => MapiType::ClassId,
            0x101e => MapiType::MvShortString,
            0x101f => MapiType::MvLongString,
            0x1102 => MapiType::MvBinary,
            0x1002 => MapiType::MvShort,
            0x1003 => MapiType::MvLong,
            0x1040 => MapiType::MvSysTime,
            other => MapiType::Unknown(other),
        }
    }

    pub fn as_u16(self) -> u16 {
        match self {
            MapiType::ShortString => 0x001e,
            MapiType::LongString => 0x001f,
            MapiType::Binary => 0x0102,
            MapiType::Short => 0x0002,
            MapiType::Long => 0x0003,
            MapiType::Bool => 0x000b,
            MapiType::SysTime => 0x0040,
            MapiType::Error => 0x000a,
            MapiType::Object => 0x000d,
            MapiType::Currency => 0x0006,
            MapiType::Double => 0x0005,
            MapiType::Float => 0x0004,
            MapiType::None => 0x0000,
            MapiType::ClassId => 0x0048,
            MapiType::MvShortString => 0x101e,
            MapiType::MvLongString => 0x101f,
            MapiType::MvBinary => 0x1102,
            MapiType::MvShort => 0x1002,
            MapiType::MvLong => 0x1003,
            MapiType::MvSysTime => 0x1040,
            MapiType::Unknown(v) => v,
        }
    }
}

/// Pack a (type, id) pair into a 32-bit MAPI tag (little-endian wire format
/// means the type is the low 16 bits and the id is the high 16 bits).
pub fn pack_tag(prop_type: u16, prop_id: u16) -> u32 {
    ((prop_id as u32) << 16) | (prop_type as u32)
}

/// Unpack a 32-bit MAPI tag into (type, id).
pub fn unpack_tag(tag: u32) -> (u16, u16) {
    let t = (tag & 0xFFFF) as u16;
    let i = ((tag >> 16) & 0xFFFF) as u16;
    (t, i)
}

/// Parse a packed MAPI property stream.
///
/// `input` is the raw stream bytes. `as_32bit` selects between the 16-bit
/// (legacy ANSI) tag layout (`tag` is 4 bytes on the wire) and the 32-bit
/// (Unicode) tag layout (`tag` is 8 bytes on the wire):
///
///   16-bit: [tag: u32] [flags: u16] [value: 4-byte length + N bytes]
///   32-bit: [tag: u64] [flags: u16] [value: 4-byte length + N bytes]
///
/// Each property record on the wire is:
///
///   [tag bytes]  [flags: u16 LE]  [value: u32 LE length + length bytes]
///
/// Returns the parsed list of properties or a `String` error if the input
/// is truncated or has invalid structure.
pub fn parse(input: &[u8], as_32bit: bool) -> Result<Vec<MapiProp>, String> {
    let mut out = Vec::new();
    let mut pos = 0;
    let tag_size = if as_32bit { 8 } else { 4 };
    while pos < input.len() {
        // Need at least the tag + flags + length = tag_size + 2 + 4.
        if pos + tag_size + 2 + 4 > input.len() {
            return Err(format!(
                "truncated MAPI stream at offset {} (need {} more bytes for header)",
                pos,
                tag_size + 2 + 4
            ));
        }
        let tag = if as_32bit {
            u64::from_le_bytes(input[pos..pos + 8].try_into().unwrap()) as u32
        } else {
            u32::from_le_bytes(input[pos..pos + 4].try_into().unwrap())
        };
        pos += tag_size;
        let flags = u16::from_le_bytes(input[pos..pos + 2].try_into().unwrap());
        pos += 2;
        let len = u32::from_le_bytes(input[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + len > input.len() {
            return Err(format!(
                "truncated MAPI value at offset {} (need {} bytes, have {})",
                pos,
                len,
                input.len() - pos
            ));
        }
        let value = input[pos..pos + len].to_vec();
        pos += len;
        out.push(MapiProp { tag, flags, value });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prop_bytes(tag: u32, flags: u16, value: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
        out.extend_from_slice(value);
        out
    }

    fn prop_bytes_32bit(tag: u32, flags: u16, value: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        // 32-bit tag layout: 4 bytes type + 4 bytes id (id first as u32 LE on wire
        // but we treat `tag` as already packed (id<<16 | type)). For consistency we
        // emit the same 4-byte pattern repeated for both halves. In practice the
        // packed `tag` is duplicated; real streams use distinct halves. The parser
        // does NOT interpret tag halves here, so it is safe for round-trip testing.
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
        out.extend_from_slice(value);
        out
    }

    // ---- pack_tag / unpack_tag ----

    #[test]
    fn tag_pack_roundtrip_short_string_subject() {
        // PidTagSubject ([MS-OXPROPS] 0x0037, type PT_UNICODE=0x001f in our enum)
        let tag = pack_tag(MapiType::LongString.as_u16(), 0x0037);
        assert_eq!(unpack_tag(tag), (0x001f, 0x0037));
    }

    #[test]
    fn tag_pack_known_pid_tag_message_class() {
        // PidTagMessageClass is 0x001A (PT_TSTRING) — test a different ID.
        let tag = pack_tag(0x001a, 0x001a);
        assert_eq!(tag, 0x001a001a);
        assert_eq!(unpack_tag(tag), (0x001a, 0x001a));
    }

    #[test]
    fn mapi_type_from_u16_maps_binary() {
        // PT_BINARY = 0x0102 ([MS-OXPROPS] section 2.2.1).
        assert_eq!(MapiType::from_u16(0x0102), MapiType::Binary);
    }

    #[test]
    fn mapi_type_unknown_passthrough() {
        assert_eq!(MapiType::from_u16(0xABCD), MapiType::Unknown(0xABCD));
        assert_eq!(MapiType::Unknown(0xABCD).as_u16(), 0xABCD);
    }

    // ---- parse (16-bit / as_32bit = false) ----

    #[test]
    fn parse_empty_stream_returns_empty_vec() {
        assert!(parse(&[], false).unwrap().is_empty());
    }

    #[test]
    fn parse_single_property_round_trip() {
        let bytes = prop_bytes(pack_tag(0x001f, 0x0037), 0, b"hello");
        let props = parse(&bytes, false).unwrap();
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].tag, pack_tag(0x001f, 0x0037));
        assert_eq!(props[0].flags, 0);
        assert_eq!(props[0].value, b"hello");
    }

    #[test]
    fn parse_two_properties_in_sequence() {
        let mut bytes = Vec::new();
        bytes.extend(prop_bytes(pack_tag(0x001f, 0x0037), 0, b"hi"));
        bytes.extend(prop_bytes(pack_tag(0x0003, 0x0e08), 0, &42i32.to_le_bytes()));
        let props = parse(&bytes, false).unwrap();
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].value, b"hi");
        assert_eq!(props[1].value, 42i32.to_le_bytes());
        let (t, i) = unpack_tag(props[1].tag);
        assert_eq!(t, 0x0003);
        assert_eq!(i, 0x0e08);
    }

    #[test]
    fn parse_empty_value_is_ok() {
        let bytes = prop_bytes(pack_tag(0x001f, 0x0037), 0, b"");
        let props = parse(&bytes, false).unwrap();
        assert_eq!(props.len(), 1);
        assert!(props[0].value.is_empty());
    }

    #[test]
    fn parse_truncated_header_reports_error() {
        // Tag + flags + length, but no length bytes follow.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&pack_tag(0x001f, 0x0037).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&10u32.to_le_bytes()); // claims 10 bytes
                                                       // No payload bytes.
        let err = parse(&bytes, false).unwrap_err();
        assert!(err.contains("truncated"), "got error: {}", err);
    }

    #[test]
    fn parse_truncated_value_reports_error() {
        let bytes = prop_bytes(pack_tag(0x001f, 0x0037), 0, b"abcdef");
        // Chop off 3 bytes of payload.
        let truncated = &bytes[..bytes.len() - 3];
        let err = parse(truncated, false).unwrap_err();
        assert!(err.contains("truncated"), "got error: {}", err);
    }

    #[test]
    fn parse_flags_round_trip() {
        let bytes = prop_bytes(pack_tag(0x0102, 0x0002), 0xCAFE, &[1, 2, 3, 4]);
        let props = parse(&bytes, false).unwrap();
        assert_eq!(props[0].flags, 0xCAFE);
    }

    // ---- parse (32-bit / as_32bit = true) ----

    #[test]
    fn parse_32bit_property_round_trip() {
        let bytes = prop_bytes_32bit(pack_tag(0x001f, 0x0037), 0, b"hello");
        let props = parse(&bytes, true).unwrap();
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].value, b"hello");
    }

    #[test]
    fn parse_32bit_two_properties() {
        let mut bytes = Vec::new();
        bytes.extend(prop_bytes_32bit(pack_tag(0x001f, 0x0037), 0, b"a"));
        bytes.extend(prop_bytes_32bit(pack_tag(0x0003, 0x0e08), 1, &7i32.to_le_bytes()));
        let props = parse(&bytes, true).unwrap();
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].flags, 0);
        assert_eq!(props[1].flags, 1);
        assert_eq!(props[1].value, 7i32.to_le_bytes());
    }
}
