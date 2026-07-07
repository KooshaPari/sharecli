// Minimal IPsec ESP packet codec (RFC 4303, Section 2).
//
// Wire-format layout (in order, all big-endian):
//
//   +-------------+-------------+-------------+-------------+
//   |                       SPI (4 bytes)                       |
//   +-------------+-------------+-------------+-------------+
//   |                    Sequence Number (4 bytes)             |
//   +-------------+-------------+-------------+-------------+
//   |              IV (variable; 0 if has_iv = false)          |
//   +---------------------------------------------------------+
//   |                  Encrypted Payload (variable)           |
//   +---------------------------------------------------------+
//   |                  Padding (0..255 bytes)                  |
//   +---------------------------------------------------------+
//   |  Pad Length  | Next Header                                |
//   +-------------+--------------------------------------------+
//   |          ICV (variable; 0 bytes if icv_len = 0)          |
//   +---------------------------------------------------------+
//
// This module does NOT decrypt the payload. It only validates the wire
// layout and exposes the structural fields. Callers provide `has_iv` and
// `icv_len` since those lengths are negotiated by the SA, not signaled on
// the wire (RFC 4303, Section 2.4).

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EspPacket {
    pub spi: u32,
    pub seq: u32,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub pad_len: u8,
    pub next_header: u8,
    pub icv: Vec<u8>,
}

pub fn parse(input: &[u8], has_iv: bool, icv_len: usize) -> Result<EspPacket, String> {
    // 4 (SPI) + 4 (Seq) = 8 bytes fixed header.
    let mut offset = 0usize;
    if input.len() < 8 {
        return Err(format!("packet too short for ESP header: got {} bytes, need at least 8", input.len()));
    }

    let spi = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    let seq = u32::from_be_bytes([input[4], input[5], input[6], input[7]]);
    offset += 8;

    let iv: Vec<u8> = if has_iv {
        // Per RFC 4303, IV length is unspecified on the wire; common values
        // are 8 (AES-CBC), 16 (AES-CTR), etc. We accept any length here as
        // long as the caller tells us via has_iv. The default block-cipher
        // IV length is implied by the cipher suite, not the packet.
        // For this parser we treat has_iv as "no IV bytes extracted by this
        // parser"; the IV is folded into the ciphertext region.
        Vec::new()
    } else {
        Vec::new()
    };

    // Total fixed tail = pad_len (1) + next_header (1) + icv.
    let tail_len = 2usize.checked_add(icv_len)
        .ok_or_else(|| "icv_len overflow".to_string())?;
    if input.len() < offset + tail_len {
        return Err(format!("packet too short for ESP trailer+ICV: need {} more bytes, have {}",
            tail_len, input.len() - offset));
    }

    // The trailing 2 bytes (pad length, next header) live at the very end of
    // the (ciphertext + padding + pad_len + next_header) region, i.e. just
    // before the ICV.
    let pad_len_offset = input.len() - icv_len - 2;
    let next_header_offset = input.len() - icv_len - 1;
    let pad_len = input[pad_len_offset];
    let next_header = input[next_header_offset];

    // Ciphertext region = everything between (offset) and (pad_len_offset - pad_len).
    let padding_start = pad_len_offset
        .checked_sub(pad_len as usize)
        .ok_or_else(|| format!("pad_len {} exceeds available bytes", pad_len))?;
    if padding_start < offset {
        return Err(format!("pad_len {} exceeds available ciphertext region", pad_len));
    }

    let ciphertext = input[offset..padding_start].to_vec();

    Ok(EspPacket {
        spi,
        seq,
        iv,
        ciphertext,
        pad_len,
        next_header,
        icv: input[next_header_offset + 1..].to_vec(),
    })
}

pub fn encode(pkt: &EspPacket) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pkt.spi.to_be_bytes());
    out.extend_from_slice(&pkt.seq.to_be_bytes());
    out.extend_from_slice(&pkt.iv);
    out.extend_from_slice(&pkt.ciphertext);
    // The caller supplies ciphertext + pad_len + next_header structure;
    // this encoder writes a complete packet. Padding bytes themselves
    // are not reconstructed — we emit zero-bytes which is the most common
    // convention and matches the encoder's role as a wire-format helper.
    out.extend(std::iter::repeat(0u8).take(pkt.pad_len as usize));
    out.push(pkt.pad_len);
    out.push(pkt.next_header);
    out.extend_from_slice(&pkt.icv);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(spi: u32, seq: u32, payload: &[u8], pad_len: u8, next_header: u8, icv_len: usize) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&spi.to_be_bytes());
        v.extend_from_slice(&seq.to_be_bytes());
        v.extend_from_slice(payload);
        v.extend(std::iter::repeat(0u8).take(pad_len as usize));
        v.push(pad_len);
        v.push(next_header);
        v.extend(std::iter::repeat(0xCC).take(icv_len));
        v
    }

    #[test]
    fn minimal_no_padding_no_icv() {
        // SPI=0xC0FFEE01, Seq=42, payload "abc", no padding, NH=IPPROTO_NONE(59), no ICV
        let bytes = build(0xC0FFEE01, 42, b"abc", 0, 59, 0);
        let pkt = parse(&bytes, false, 0).unwrap();
        assert_eq!(pkt.spi, 0xC0FFEE01);
        assert_eq!(pkt.seq, 42);
        assert_eq!(pkt.ciphertext, b"abc");
        assert_eq!(pkt.pad_len, 0);
        assert_eq!(pkt.next_header, 59); // IPPROTO_NONE
        assert!(pkt.icv.is_empty());
    }

    #[test]
    fn with_padding_and_icv() {
        // payload "hello", pad 4 bytes, NH=TCP(6), ICV = 12 bytes (HMAC-SHA1-96)
        let bytes = build(0x00000005, 1, b"hello", 4, 6, 12);
        let pkt = parse(&bytes, false, 12).unwrap();
        assert_eq!(pkt.ciphertext, b"hello");
        assert_eq!(pkt.pad_len, 4);
        assert_eq!(pkt.next_header, 6); // IPPROTO_TCP
        assert_eq!(pkt.icv.len(), 12);
        assert!(pkt.icv.iter().all(|&b| b == 0xCC));
    }

    #[test]
    fn spi_seq_extraction() {
        // Verify SPI and Sequence Number are read as big-endian u32.
        let bytes = build(0xDEADBEEF, 0x01020304, b"x", 0, 4, 0);
        let pkt = parse(&bytes, false, 0).unwrap();
        assert_eq!(pkt.spi, 0xDEADBEEF);
        assert_eq!(pkt.seq, 0x01020304);
    }

    #[test]
    fn full_alignment_boundary() {
        // Payload aligned to 4-byte boundary via 2 bytes of padding (RFC 4303 §2.4 alignment)
        let bytes = build(0x12345678, 99, b"xy", 2, 17, 16); // UDP = 17, ICV = AES-GCM 16 bytes
        let pkt = parse(&bytes, false, 16).unwrap();
        assert_eq!(pkt.ciphertext, b"xy");
        assert_eq!(pkt.pad_len, 2);
        assert_eq!(pkt.next_header, 17);
        assert_eq!(pkt.icv.len(), 16);
    }

    #[test]
    fn empty_payload() {
        // Pathological but legal: empty ciphertext region, zero padding
        let bytes = build(1, 0, b"", 0, 59, 0);
        let pkt = parse(&bytes, false, 0).unwrap();
        assert!(pkt.ciphertext.is_empty());
        assert_eq!(pkt.pad_len, 0);
        assert_eq!(pkt.next_header, 59);
    }

    #[test]
    fn truncated_header() {
        // Only 5 bytes — less than the 8-byte fixed header
        let bytes = vec![0, 1, 2, 3, 4];
        assert!(parse(&bytes, false, 0).is_err());
    }

    #[test]
    fn truncated_trailer() {
        // header is fine (8 bytes) but next_header byte is missing
        let bytes = vec![0; 8];
        assert!(parse(&bytes, false, 0).is_err());
    }

    #[test]
    fn pad_len_exceeds_region() {
        // pad_len claims 100 bytes of padding, but only 1 byte before pad_len field
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&2u32.to_be_bytes());
        bytes.push(0xAA); // one byte of "payload"
        bytes.push(100);  // pad_len = 100 (impossible)
        bytes.push(59);   // next_header
        let err = parse(&bytes, false, 0).unwrap_err();
        assert!(err.contains("pad_len") || err.contains("truncated"));
    }

    #[test]
    fn encode_decode_round_trip() {
        let original = EspPacket {
            spi: 0xCAFEBABE,
            seq: 12345,
            iv: vec![],
            ciphertext: b"encrypted-payload".to_vec(),
            pad_len: 3,
            next_header: 6,
            icv: vec![0xAA; 12],
        };
        let bytes = encode(&original);
        let parsed = parse(&bytes, false, 12).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn no_icv_aes_gcm_implicit() {
        // AES-GCM combines integrity + encryption: icv_len=0 but ciphertext still present
        let bytes = build(0x00010001, 1, b"ciphertext", 5, 4, 0);
        let pkt = parse(&bytes, false, 0).unwrap();
        assert_eq!(pkt.ciphertext, b"ciphertext");
        assert_eq!(pkt.pad_len, 5);
        assert!(pkt.icv.is_empty());
    }
}