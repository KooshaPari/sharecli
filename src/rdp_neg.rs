// Minimal Remote Desktop Protocol negotiation parser (MS-RDPBCGR §2.2).
//
// RDP embeds a multi-layer framing on the wire. The negotiation request that
// starts an RDP session has the following wire shape (MS-RDPBCGR §2.2.1):
//
//     TPKT Header (ITU-T X.224 / RFC 1006):
//       1 byte  version      (0x03)
//       1 byte  reserved     (0x00)
//       2 bytes length       (big-endian, includes TPKT header)
//
//     X.224 Connection Request TPDU (ITU-T X.224 §13.3):
//       1 byte  length       (0x26 = 38, the size of the TPDU payload
//                             that follows the length byte)
//       1 byte  CR TPDU code (0xE0)
//       1 byte  DST-REF      (0x00, padding)
//       1 byte  SRC-REF      (0x00, padding)
//       1 byte  Class Option (0x00)
//       ... (Cookie / rdpCookie) ...
//       1 byte  Protocol /   (0x0E)
//             negotiation  ("Cookie: mstshash=" + terminator
//                           + RDPNEG_REQ signature "Cookie: mstshash=\r\n"
//                           + appended RDPNEG_REQ fields)
//
//     Cookie  (MS-RDPBCGR §2.2.1.1.1):
//       ASCII "Cookie: mstshash=" followed by the user name and a 0x0D 0x0A
//       terminator. The literal ASCII bytes ("Cookie: mstshash=") on the
//       wire spell out the hex sequence:
//       43 6F 6F 6B 69 65 3A 20 6D 73 74 73 68 61 73 68 3D  -- 18 bytes
//
//     RDP Negotiation Request (MS-RDPBCGR §2.2.1.1):
//       1 byte  type         (0x01 = TYPE_RDP_NEG_REQ)
//       1 byte  flags        (0x00 in standard requests)
//       2 bytes length       (0x0008 little-endian, the size of the
//                             request payload that follows)
//       4 bytes requestedProtocols  (little-endian bitmask; see flags
//                                    below)
//
//     RDP Negotiation Request flags (PROTOCOL_* values are MS-RDPBCGR
//     §2.2.1.1.1, "requestedProtocols" — bit-OR combinations are allowed):
//
//       PROTOCOL_RDS_TLS    0x00000004  (server-bound TLS, legacy)
//       PROTOCOL_RDS_LEGACY 0x00000000  (no security)
//       PROTOCOL_HYBRID     0x00000002  (NLA / CredSSP)
//       PROTOCOL_SSL        0x00000001  (legacy SSL)
//
// We expose a focused parser that:
//   1. Strips the TPKT header and validates the magic.
//   2. Strips the X.224 CR TPDU length header.
//   3. Reads the cookie (if any) up to the 0x0D 0x0A terminator.
//   4. Reads the RDPNEG_REQ body.
//   5. Returns the protocol bitmask and any bytes left over from the TPKT
//      payload (so callers can fold the result into a connection state
//      machine without us speculating about framing).
//
// This parser does *not* validate the full X.224 TPDU — it only consumes the
// portions of the wire format that are needed to extract the negotiation
// request, which is the section most useful for protocol detection.

/// `TYPE_RDP_NEG_REQ` from MS-RDPBCGR §2.2.1.1.
pub const TYPE_RDP_NEG_REQ: u8 = 0x01;

/// Cookie prefix as it appears on the wire before the user name and the
/// `\r\n` terminator (MS-RDPBCGR §2.2.1.1.1).
/// ASCII = "Cookie: mstshash=".
pub const COOKIE_PREFIX: &[u8] = b"Cookie: mstshash=";

/// Cookie terminator per MS-RDPBCGR §2.2.1.1.1: `\r\n`.
pub const COOKIE_TERMINATOR: &[u8] = b"\r\n";

/// RDP negotiation request flags (MS-RDPBCGR §2.2.1.1, "requestedProtocols").
pub const PROTOCOL_SSL: u32 = 0x0000_0001;
pub const PROTOCOL_HYBRID: u32 = 0x0000_0002;
pub const PROTOCOL_RDS_TLS: u32 = 0x0000_0004;

/// Decoded RDP negotiation request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdpNegReq {
    /// Negotiated security-protocol bitmask (PROTOCOL_* flag values).
    pub rdp_protocols: u32,
    /// The cookie value when one was present (between `Cookie: mstshash=`
    /// and the `\r\n` terminator), `None` if the request had no cookie or
    /// the cookie was malformed.
    pub cookie: Option<Vec<u8>>,
}

/// Parse the body of a CR TPDU (the part that follows the X.224 length-byte
/// header). Returns the negotiation request plus any bytes that remain in
/// the same TPKT payload (so the caller can inspect whatever sits after the
/// `requestedProtocols` field).
///
/// `input` should typically be the bytes *after* the TPKT header (and the
/// initial X.224 length byte and CR byte); if you have a raw frame, peel
/// the framing off yourself with `parse_request`'s sibling helper below.
pub fn parse_request(input: &[u8]) -> Result<(RdpNegReq, &[u8]), String> {
    // Cookie: optional, but if the prefix is present, the terminator and
    // the RDPNEG_REQ body must follow. We make the cookie optional so that
    // older / minimal RDP clients (which omit a cookie) still parse.
    let mut rest = input;
    let cookie = if rest.starts_with(COOKIE_PREFIX) {
        rest = &rest[COOKIE_PREFIX.len()..];
        if let Some(idx) = find_subslice(rest, COOKIE_TERMINATOR) {
            let value = rest[..idx].to_vec();
            rest = &rest[idx + COOKIE_TERMINATOR.len()..];
            Some(value)
        } else {
            // Cookie prefix declared but no terminator — broken frame.
            return Err("rdp_neg: cookie prefix present but missing CRLF terminator".into());
        }
    } else {
        None
    };

    // RDPNEG_REQ body — 8 bytes total per §2.2.1.1.
    if rest.len() < 8 {
        return Err(format!(
            "rdp_neg: truncated RDPNEG_REQ body (need 8 bytes, got {})",
            rest.len()
        ));
    }
    let req_type = rest[0];
    if req_type != TYPE_RDP_NEG_REQ {
        return Err(format!(
            "rdp_neg: expected TYPE_RDP_NEG_REQ (0x01), got 0x{:02X}",
            req_type
        ));
    }
    // rest[1] is `flags` per spec; we do not enforce a particular value
    // here because there are known clients that set it non-zero.
    let _flags = rest[1];
    let req_len = u16::from_le_bytes([rest[2], rest[3]]) as usize;
    if req_len != 8 {
        return Err(format!(
            "rdp_neg: RDPNEG_REQ length must be 8, got {}",
            req_len
        ));
    }
    let rdp_protocols = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]);
    let tail = &rest[8..];

    Ok((
        RdpNegReq {
            rdp_protocols,
            cookie,
        },
        tail,
    ))
}

/// Parse a raw frame starting with the TPKT header. Returns the same
/// `RdpNegReq` plus the bytes after the RDPNEG_REQ body within the TPKT
/// payload (still inside the TPDU).
pub fn parse_frame(input: &[u8]) -> Result<(RdpNegReq, &[u8]), String> {
    if input.len() < 4 {
        return Err(format!(
            "rdp_neg: TPKT header too short (need 4 bytes, got {})",
            input.len()
        ));
    }
    let tpkt_version = input[0];
    if tpkt_version != 0x03 {
        return Err(format!(
            "rdp_neg: bad TPKT version (want 0x03, got 0x{:02X})",
            tpkt_version
        ));
    }
    let tpkt_length = u16::from_be_bytes([input[2], input[3]]) as usize;
    if tpkt_length < 4 {
        return Err(format!(
            "rdp_neg: bad TPKT length ({} < 4)",
            tpkt_length
        ));
    }
    let tpkt_end = std::cmp::min(tpkt_length, input.len());
    let payload = &input[4..tpkt_end];

    // X.224 CR TPDU header — first byte is the length (excluding the length
    // byte itself, but including the 0xE0 code byte that follows).
    if payload.is_empty() {
        return Err("rdp_neg: empty TPDU after TPKT".into());
    }
    let tpdu_len = payload[0] as usize;
    // Validate: TPDU length must include the 0xE0 byte — minimum is 6 bytes
    // (length + 0xE0 + dst-ref + src-ref + class option + cookie-rdp).
    if tpdu_len < 6 || payload.len() < 1 + tpdu_len {
        return Err(format!(
            "rdp_neg: TPDU length {} invalid for buffer of {} bytes",
            tpdu_len,
            payload.len()
        ));
    }
    let tpdu_payload = &payload[1..1 + tpdu_len];
    if tpdu_payload.is_empty() || tpdu_payload[0] != 0xE0 {
        return Err(format!(
            "rdp_neg: bad X.224 CR TPDU code (want 0xE0, got 0x{:02X})",
            tpdu_payload.first().copied().unwrap_or(0)
        ));
    }
    let cr_body = &tpdu_payload[1..]; // skip 0xE0
    parse_request(cr_body)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_rejected() {
        let err = parse_request(&[]).expect_err("must reject");
        assert!(err.contains("truncated"), "got: {}", err);
    }

    #[test]
    fn parse_minimal_req_no_cookie() {
        // Hand-built minimal RDPNEG_REQ body — no cookie, just the 8-byte body.
        let bytes = [
            0x01, // type
            0x00, // flags
            0x08, 0x00, // length=8 (LE)
            0x03, 0x00, 0x00, 0x00, // PROTOCOL_SSL | PROTOCOL_HYBRID
        ];
        let (req, rest) = parse_request(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert!(req.cookie.is_none());
        assert_eq!(req.rdp_protocols, PROTOCOL_SSL | PROTOCOL_HYBRID);
    }

    #[test]
    fn parse_with_cookie() {
        // Cookie "alice" then CRLF, then RDPNEG_REQ.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(COOKIE_PREFIX);
        bytes.extend_from_slice(b"alice");
        bytes.extend_from_slice(COOKIE_TERMINATOR);
        bytes.extend_from_slice(&[
            0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, // PROTOCOL_SSL
        ]);
        let (req, rest) = parse_request(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(req.cookie.as_deref().unwrap(), b"alice");
        assert_eq!(req.rdp_protocols, PROTOCOL_SSL);
    }

    #[test]
    fn wrong_request_type_rejected() {
        let bytes = [0xFF, 0x00, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00];
        let err = parse_request(&bytes).expect_err("must reject");
        assert!(err.contains("TYPE_RDP_NEG_REQ"), "got: {}", err);
    }

    #[test]
    fn wrong_request_length_rejected() {
        // Spec says length field MUST be 8.
        let bytes = [0x01, 0x00, 0x09, 0x00, 0x01, 0x00, 0x00, 0x00];
        let err = parse_request(&bytes).expect_err("must reject");
        assert!(err.contains("length must be 8"), "got: {}", err);
    }

    #[test]
    fn cookie_missing_terminator_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(COOKIE_PREFIX);
        bytes.extend_from_slice(b"bob_no_crlf");
        // No CRLF, no RDPNEG_REQ.
        let err = parse_request(&bytes).expect_err("must reject");
        assert!(err.contains("CRLF"), "got: {}", err);
    }

    #[test]
    fn hybrid_protocol_constant() {
        // Ensure the bitmask constants from §2.2.1.1 match the spec.
        assert_eq!(PROTOCOL_SSL, 0x0000_0001);
        assert_eq!(PROTOCOL_HYBRID, 0x0000_0002);
        assert_eq!(PROTOCOL_RDS_TLS, 0x0000_0004);
    }

    #[test]
    fn parse_frame_tpkt_x224() {
        // Build a minimal valid full frame:
        //   TPKT: 03 00 <len> <len>
        //   X.224 CR: <len> E0 00 00 00 <cookie...> <RDPNEG_REQ body>
        let mut body = Vec::new();
        body.extend_from_slice(COOKIE_PREFIX);
        body.extend_from_slice(b"user");
        body.extend_from_slice(COOKIE_TERMINATOR);
        body.extend_from_slice(&[
            0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x00, 0x00, // PROTOCOL_HYBRID
        ]);
        // X.224 CR TPDU: length byte (1 + 1 + body.len()), then 0xE0 + body.
        // length=N counts 0xE0 byte itself + everything that follows it.
        let tpdu_payload_len = 1 + body.len(); // 0xE0 + body
        assert!(tpdu_payload_len <= 255);
        let mut tpdu = Vec::new();
        tpdu.push(tpdu_payload_len as u8);
        tpdu.push(0xE0);
        tpdu.extend_from_slice(&body);
        // TPKT wraps the entire TPDU (header + TPDU).
        let tpkt_payload_len = 4 + tpdu.len();
        let mut frame = Vec::new();
        frame.push(0x03);
        frame.push(0x00);
        frame.push(((tpkt_payload_len >> 8) & 0xFF) as u8);
        frame.push((tpkt_payload_len & 0xFF) as u8);
        frame.extend_from_slice(&tpdu);
        let (req, rest) = parse_frame(&frame).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(req.cookie.as_deref().unwrap(), b"user");
        assert_eq!(req.rdp_protocols, PROTOCOL_HYBRID);
    }

    #[test]
    fn parse_frame_bad_tpkt_version_rejected() {
        let bytes = [0x04, 0x00, 0x00, 0x10];
        let err = parse_frame(&bytes).expect_err("must reject");
        assert!(err.contains("TPKT version"), "got: {}", err);
    }
}
