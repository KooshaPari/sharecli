// Minimal ISO Base Media File Format (ISOBMFF / MP4) box parser.
//
// Spec: ISO/IEC 14496-12 (boxes / files).
//
// Top-level box header (8 bytes, big-endian):
//   bytes 0..=3   size (32 bits, big-endian)
//   bytes 4..=7   type (4 ASCII chars, e.g. "ftyp", "moov", "mdat")
//
// If size == 1, the real size is read from the next 8 bytes
// (64-bit largesize). If size == 0, the box extends to EOF.
//
// For container boxes ("moov", "trak", "mdia", "minf", "stbl",
// "udta", "edts", "dinf", "stsd") the parser recurses into the
// payload and populates `children`. For other boxes the payload is
// stored verbatim in `payload` and `children` is empty.
//
// This module is intentionally minimal. It parses only the structural
// shape (header + children) and exposes `find_box` for retrieval.
// It does NOT decode box-specific bodies (e.g. `mvhd` version
// fields, `tkhd` flags, `stts`/`stsc`/`stsz` sample tables). Those
// stay opaque in `payload`.

/// Container box types whose payload is itself a sequence of boxes.
const CONTAINER_TYPES: &[&str] = &[
    "moov", "trak", "mdia", "minf", "stbl", "udta", "edts", "dinf", "stsd",
];

/// Minimum size of a valid box header.
pub const HEADER_SIZE: usize = 8;

/// One parsed box (may be a leaf or a container).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Box {
    /// 4-character ASCII box type (e.g. `"ftyp"`, `"moov"`).
    pub box_type: String,
    /// Total box size in bytes (header + payload).
    pub size: usize,
    /// Raw payload bytes (always present, even for container boxes).
    pub payload: Vec<u8>,
    /// Children for container boxes; empty for leaf boxes.
    pub children: Vec<Box>,
}

impl Box {
    fn is_container(&self) -> bool {
        CONTAINER_TYPES.contains(&self.box_type.as_str())
    }
}

/// Parse a sequence of top-level boxes from `input`. The parser
/// walks input until `input.len()` (or `end`, if given) is reached.
/// All top-level boxes must end inside the supplied range.
///
/// Returns an error if:
///   * a box header is truncated,
///   * a box overruns the available range,
///   * the type field is not 4 ASCII bytes.
pub fn parse_boxes(input: &[u8], end: Option<usize>) -> Result<Vec<Box>, String> {
    let limit = end.unwrap_or(input.len());
    if limit > input.len() {
        return Err(format!(
            "end {} exceeds input length {}",
            limit,
            input.len()
        ));
    }

    let mut out = Vec::new();
    let mut offset = 0usize;

    while offset < limit {
        if offset + HEADER_SIZE > limit {
            return Err(format!(
                "truncated MP4 box header at offset {} (have {} bytes)",
                offset,
                limit - offset
            ));
        }
        let size32 = u32::from_be_bytes([
            input[offset],
            input[offset + 1],
            input[offset + 2],
            input[offset + 3],
        ]) as usize;
        let box_type_bytes = &input[offset + 4..offset + HEADER_SIZE];

        if !is_ascii_type(box_type_bytes) {
            return Err(format!(
                "non-ASCII box type at offset {}: {:?}",
                offset, box_type_bytes
            ));
        }
        let box_type = std::str::from_utf8(box_type_bytes)
            .map_err(|e| format!("invalid UTF-8 in box type at offset {}: {}", offset, e))?
            .to_string();

        let (total_size, header_len) = if size32 == 1 {
            // 64-bit largesize lives in the next 8 bytes.
            if offset + HEADER_SIZE + 8 > limit {
                return Err(format!(
                    "truncated largesize at offset {} (size=1 sentinel)",
                    offset
                ));
            }
            let hi = u32::from_be_bytes([
                input[offset + HEADER_SIZE],
                input[offset + HEADER_SIZE + 1],
                input[offset + HEADER_SIZE + 2],
                input[offset + HEADER_SIZE + 3],
            ]) as u64;
            let lo = u32::from_be_bytes([
                input[offset + HEADER_SIZE + 4],
                input[offset + HEADER_SIZE + 5],
                input[offset + HEADER_SIZE + 6],
                input[offset + HEADER_SIZE + 7],
            ]) as u64;
            let largesize = (hi << 32) | lo;
            if largesize < (HEADER_SIZE as u64 + 8) {
                return Err(format!(
                    "invalid largesize {} at offset {}",
                    largesize, offset
                ));
            }
            (largesize as usize, HEADER_SIZE + 8)
        } else if size32 == 0 {
            // Box extends to EOF.
            (limit - offset, HEADER_SIZE)
        } else {
            if size32 < HEADER_SIZE {
                return Err(format!(
                    "invalid MP4 box size {} (smaller than header) at offset {}",
                    size32, offset
                ));
            }
            (size32, HEADER_SIZE)
        };

        let end_offset = offset
            .checked_add(total_size)
            .ok_or_else(|| format!("box size overflow at offset {}", offset))?;
        if end_offset > limit {
            return Err(format!(
                "MP4 box overruns available bytes at offset {} (size {} but only {} available)",
                offset,
                total_size,
                limit - offset
            ));
        }
        let payload = input[offset + header_len..end_offset].to_vec();

        let mut bx = Box {
            box_type,
            size: total_size,
            payload: payload.clone(),
            children: Vec::new(),
        };

        if bx.is_container() {
            bx.children = parse_boxes(&payload, None)?;
        }

        out.push(bx);
        offset = end_offset;
    }

    Ok(out)
}

/// Recursively search `boxes` for the first box whose `box_type`
/// equals `box_type`. Returns `None` if no matching box is found.
pub fn find_box<'a>(boxes: &'a [Box], box_type: &str) -> Option<&'a Box> {
    for b in boxes {
        if b.box_type == box_type {
            return Some(b);
        }
        if let Some(found) = find_box(&b.children, box_type) {
            return Some(found);
        }
    }
    None
}

fn is_ascii_type(b: &[u8]) -> bool {
    if b.len() != 4 {
        return false;
    }
    b.iter().all(|c| (0x20..=0x7E).contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_bytes(box_type: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let total = HEADER_SIZE + payload.len();
        let mut v = Vec::new();
        v.extend_from_slice(&(total as u32).to_be_bytes());
        v.extend_from_slice(box_type);
        v.extend_from_slice(payload);
        v
    }

    #[test]
    fn parse_ftyp_only() {
        // ftyp box: "isom" brand + minor version 0x200 + compatible "mp41"
        let mut payload = Vec::new();
        payload.extend_from_slice(b"isom");
        payload.extend_from_slice(&0x0000_0200u32.to_be_bytes());
        payload.extend_from_slice(b"mp41");
        let bytes = box_bytes(b"ftyp", &payload);

        let boxes = parse_boxes(&bytes, None).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].box_type, "ftyp");
        assert_eq!(boxes[0].size, bytes.len());
        assert_eq!(boxes[0].payload, payload);
        assert!(boxes[0].children.is_empty());
    }

    #[test]
    fn parse_moov_with_children() {
        // Build a moov containing a single mvhd child with 100 bytes of payload.
        let mvhd_payload = vec![0xAB; 100];
        let mvhd = box_bytes(b"mvhd", &mvhd_payload);
        let moov = box_bytes(b"moov", &mvhd);

        let boxes = parse_boxes(&moov, None).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].box_type, "moov");
        assert!(boxes[0].is_container());

        let mvhd_parsed = find_box(&boxes, "mvhd").unwrap();
        assert_eq!(mvhd_parsed.box_type, "mvhd");
        assert_eq!(mvhd_parsed.payload, mvhd_payload);
        assert_eq!(mvhd_parsed.size, mvhd.len());
    }

    #[test]
    fn parse_multiple_top_level_boxes() {
        let ftyp = box_bytes(b"ftyp", b"isom\x00\x00\x02\x00mp41");
        let moov = box_bytes(
            b"moov",
            &box_bytes(b"trak", &box_bytes(b"tkhd", &[0u8; 8])),
        );

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ftyp);
        bytes.extend_from_slice(&moov);

        let boxes = parse_boxes(&bytes, None).unwrap();
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].box_type, "ftyp");
        assert_eq!(boxes[1].box_type, "moov");
        // nested trak + tkhd walk should succeed
        assert!(find_box(&boxes, "tkhd").is_some());
    }

    #[test]
    fn find_box_returns_none_when_absent() {
        let bytes = box_bytes(b"ftyp", b"isom");
        let boxes = parse_boxes(&bytes, None).unwrap();
        assert!(find_box(&boxes, "moov").is_none());
    }

    #[test]
    fn malformed_size_rejects_smaller_than_header() {
        // size = 4 (less than HEADER_SIZE)
        let mut bytes = vec![0, 0, 0, 4, b'f', b't', b'y', b'p'];
        bytes.extend_from_slice(b"isom");
        assert!(parse_boxes(&bytes, None).is_err());
    }

    #[test]
    fn malformed_size_rejects_overrun() {
        // Header claims 32 bytes but only 8 follow.
        let mut bytes = vec![0, 0, 0, 32, b'f', b't', b'y', b'p'];
        bytes.extend_from_slice(&[0u8; 8]);
        assert!(parse_boxes(&bytes, None).is_err());
    }

    #[test]
    fn truncated_header_rejects() {
        let bytes = vec![0, 0, 0]; // only 3 bytes
        assert!(parse_boxes(&bytes, None).is_err());
    }

    #[test]
    fn zero_size_box_extends_to_end_of_input() {
        // size=0 sentinel: box runs to EOF.
        let mut bytes = vec![0, 0, 0, 0, b'm', b'd', b'a', b't'];
        bytes.extend_from_slice(&[1, 2, 3, 4, 5]);
        let boxes = parse_boxes(&bytes, None).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].box_type, "mdat");
        assert_eq!(boxes[0].size, bytes.len());
        assert_eq!(boxes[0].payload, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn largesize_box_parses() {
        // size=1 sentinel + 64-bit largesize covering header(8)+largesize(8)+payload(200) = 216
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_be_bytes()); // size sentinel
        bytes.extend_from_slice(b"mdat");
        bytes.extend_from_slice(&216u64.to_be_bytes()); // 64-bit largesize
        bytes.extend_from_slice(&vec![0u8; 200]);

        let boxes = parse_boxes(&bytes, None).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].box_type, "mdat");
        assert_eq!(boxes[0].size, 216);
        assert_eq!(boxes[0].payload.len(), 200);
    }

    #[test]
    fn end_parameter_truncates_parse() {
        let ftyp = box_bytes(b"ftyp", b"isom");
        let moov = box_bytes(b"moov", &[]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ftyp);
        bytes.extend_from_slice(&moov);

        // Only parse the first box.
        let boxes = parse_boxes(&bytes, Some(ftyp.len())).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].box_type, "ftyp");
    }

    #[test]
    fn non_ascii_type_rejects() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&12u32.to_be_bytes());
        bytes.extend_from_slice(&[0xFF, b't', b'y', b'p']);
        bytes.extend_from_slice(&[0u8; 4]);
        assert!(parse_boxes(&bytes, None).is_err());
    }

    #[test]
    fn find_box_walks_recursively() {
        // Build: ftyp + moov[trak[tkhd, mdia[mdhd]]]
        let tkhd = box_bytes(b"tkhd", &[0u8; 8]);
        let mdhd = box_bytes(b"mdhd", &[0u8; 4]);
        let mut mdia_payload = Vec::new();
        mdia_payload.extend_from_slice(&mdhd);
        let mdia = box_bytes(b"mdia", &mdia_payload);
        let mut trak_payload = Vec::new();
        trak_payload.extend_from_slice(&tkhd);
        trak_payload.extend_from_slice(&mdia);
        let trak = box_bytes(b"trak", &trak_payload);
        let moov = box_bytes(b"moov", &trak);
        let ftyp = box_bytes(b"ftyp", b"isom");

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ftyp);
        bytes.extend_from_slice(&moov);

        let boxes = parse_boxes(&bytes, None).unwrap();
        assert!(find_box(&boxes, "mdhd").is_some());
        let mdhd_box = find_box(&boxes, "mdhd").unwrap();
        assert_eq!(mdhd_box.payload, vec![0u8; 4]);
    }
}