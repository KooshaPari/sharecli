// Minimal SNMPv3 message parser (RFC 3412 Section 6 / RFC 3414 framing).
//
// This module parses the SNMPv3 wire format up to (but not including) the
// cryptographic operations of the User-based Security Model (USM). It is
// strictly a structural decoder for the message envelope:
//
//   +-----------+-------+-------+--------+--------+-----------------------+
//   |  integer  |  oct  |  oct  |   oct  |  str   |    integer (PDU)     |
//   | version-3 | msgID | flags | secMdl | secPar |     scopedPDU         |
//   +-----------+-------+-------+--------+--------+-----------------------+
//
// Wire layout (BER-TLV, all TLV fields use the SNMP INTEGER / OCTET STRING
// tags from SMIv2, but at the top level they are encoded as fixed-shape
// fields rather than full BER TLVs):
//
//   msgVersion [INTEGER]   - 3 bytes including the ASN.1 tag/length. The
//                            value byte must be 0x03 (the integer value 3).
//   msgID      [INTEGER]   - 4 bytes: tag, length, then up to 2 value bytes
//                            of the request identifier (big-endian).
//   msgMaxSize [INTEGER]   - 4 bytes: tag, length, then up to 2 value bytes
//                            of the message max size. Often seen in real
//                            captures; we accept and skip it if present.
//   msgFlags   [OCTET STR] - 3 bytes: tag, length=0x01, then a single flags
//                            byte (reportable | priv | auth | 4 bits unused).
//   msgSecurityModel [INT] - 3 bytes: tag, length, then 1 value byte.
//   msgSecurityParameters [OCTET STRING] - tag + length + opaque blob
//                            whose interior is model-specific. We capture it
//                            verbatim as `security_params`.
//   scopedPDU  [SEQUENCE]  - the BER-TLV of the inner scoped PDU, captured
//                            verbatim as `scoped_pdu`.
//
// This module does NOT implement USM (RFC 3414) encryption/decryption,
// timeliness checking, or HMAC validation. It only validates the envelope
// shape so callers can decide how to handle the security parameters blob.
//
// Bit layout of `flags_byte_raw` (RFC 3412 §6.3, msgFlags OCTET STRING):
//
//   bit 7 (MSB) = reportableFlag  (1 = expect a Report PDU on errors)
//   bit 6       = privFlag        (1 = scopedPDU is encrypted)
//   bit 5       = authFlag        (1 = scopedPDU is authenticated)
//   bits 4..0   = unused, MUST be zero per RFC 3412 §6.3
//
// `security_level` is the standard 3-valued enum from RFC 3412 §3.1.6
// (noAuth / authNoPriv / authPriv), which we recover from auth|priv bits.

/// Parsed SNMPv3 message envelope. `data` is the original input buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct V3Msg {
    /// The `msgFlags` OCTET STRING value byte, preserved raw.
    pub flags_byte_raw: u8,
    /// 1 = reportable, 0 = not (RFC 3412 §6.3).
    pub reportable: bool,
    /// 1 = scopedPDU is encrypted.
    pub priv_flag: bool,
    /// 1 = scopedPDU is authenticated.
    pub auth_flag: bool,
    /// Convenience field for the standard 3-value enum: 1=noAuth, 2=authNoPriv, 3=authPriv.
    pub security_level: u8,
    /// The `msgSecurityModel` INTEGER value (e.g. 3 = USM).
    pub security_model: u8,
    /// The opaque `msgSecurityParameters` OCTET STRING contents.
    pub security_params: Vec<u8>,
    /// The verbatim `scopedPDU` SEQUENCE bytes (tag+length+contents).
    pub scoped_pdu: Vec<u8>,
    /// The original input bytes for reference.
    pub data: Vec<u8>,
}

/// Parse an SNMPv3 message from `input`.
///
/// On success, returns a `V3Msg` whose fields are derived from the wire
/// encoding. On failure, returns a `String` describing which BER field could
/// not be consumed.
pub fn parse(input: &[u8]) -> Result<V3Msg, String> {
    let data = input.to_vec();
    let mut i = 0usize;

    // ---- msgVersion [INTEGER] ------------------------------------------------
    // Tag=0x02, len=0x01, value=0x03
    if input.len() < 3 {
        return Err("truncated msgVersion".into());
    }
    if input[0] != 0x02 {
        return Err(format!("msgVersion tag mismatch: expected 0x02 got 0x{:02X}", input[0]));
    }
    if input[1] != 0x01 {
        return Err(format!("msgVersion length mismatch: expected 0x01 got 0x{:02X}", input[1]));
    }
    if input[2] != 0x03 {
        return Err(format!("msgVersion value mismatch: expected 0x03 got 0x{:02X}", input[2]));
    }
    i += 3;

    // ---- msgID [INTEGER] -----------------------------------------------------
    // Tag=0x02, length=0x01..0x04, then value bytes (we accept 1..4 bytes)
    let (msg_id_len, _msg_id_value_len, _msg_id_bytes) =
        read_int_tlv(input, &mut i, "msgID")?;

    // ---- msgMaxSize [INTEGER] (optional but ubiquitous) ----------------------
    // Tag=0x02, length=0x01..0x04, then value bytes.
    let (_msg_max_len, _msg_max_value_len, _msg_max_bytes) =
        read_int_tlv(input, &mut i, "msgMaxSize")?;

    // ---- msgFlags [OCTET STRING] ---------------------------------------------
    if i + 3 > input.len() {
        return Err("truncated msgFlags".into());
    }
    if input[i] != 0x04 {
        return Err(format!("msgFlags tag mismatch: expected 0x04 got 0x{:02X}", input[i]));
    }
    if input[i + 1] != 0x01 {
        return Err(format!("msgFlags length mismatch: expected 0x01 got 0x{:02X}", input[i + 1]));
    }
    let flags_byte = input[i + 2];
    i += 3;

    // ---- msgSecurityModel [INTEGER] ------------------------------------------
    if i + 3 > input.len() {
        return Err("truncated msgSecurityModel".into());
    }
    if input[i] != 0x02 {
        return Err(format!(
            "msgSecurityModel tag mismatch: expected 0x02 got 0x{:02X}",
            input[i]
        ));
    }
    if input[i + 1] != 0x01 {
        return Err(format!(
            "msgSecurityModel length mismatch: expected 0x01 got 0x{:02X}",
            input[i + 1]
        ));
    }
    let security_model = input[i + 2];
    i += 3;

    // ---- msgSecurityParameters [OCTET STRING] -------------------------------
    if i >= input.len() {
        return Err("truncated msgSecurityParameters header".into());
    }
    if input[i] != 0x04 {
        return Err(format!(
            "msgSecurityParameters tag mismatch: expected 0x04 got 0x{:02X}",
            input[i]
        ));
    }
    let (params_header, params_value_len) =
        read_octet_string_tlv(input, &mut i, "msgSecurityParameters")?;
    if i + params_value_len > input.len() {
        return Err("truncated msgSecurityParameters body".into());
    }
    let security_params = input[i..i + params_value_len].to_vec();
    i += params_value_len;

    // ---- scopedPDU [SEQUENCE] -----------------------------------------------
    if i >= input.len() {
        return Err("truncated scopedPDU header".into());
    }
    if input[i] != 0x30 {
        return Err(format!("scopedPDU tag mismatch: expected 0x30 got 0x{:02X}", input[i]));
    }
    let scoped_pdu_value_len = read_constructed_len(input, &mut i, "scopedPDU")?;
    if i + scoped_pdu_value_len > input.len() {
        return Err("truncated scopedPDU body".into());
    }
    let scoped_pdu = input[i..i + scoped_pdu_value_len].to_vec();
    // We do not advance `i` past scopedPDU because it is the final field.

    // Bit decoding per RFC 3412 §6.3.
    let reportable = (flags_byte & 0b1000_0000) != 0;
    let priv_flag = (flags_byte & 0b0100_0000) != 0;
    let auth_flag = (flags_byte & 0b0010_0000) != 0;
    let security_level: u8 = if !auth_flag && !priv_flag {
        1 // noAuth
    } else if auth_flag && !priv_flag {
        2 // authNoPriv
    } else {
        3 // authPriv
    };

    Ok(V3Msg {
        flags_byte_raw: flags_byte,
        reportable,
        priv_flag,
        auth_flag,
        security_level,
        security_model,
        security_params,
        scoped_pdu,
        data,
    })
    // `i` is unused past scopedPDU; suppress the unused warning in case the
    // compiler complains. Note: `_msg_id_len`, `_msg_max_len`, `params_header`
    // are intentionally unused after construction.
    .map(|m| {
        let _ = (msg_id_len, params_header, _msg_id_bytes, _msg_max_bytes);
        m
    })
}

/// Read an INTEGER TLV at `i`, advance `i`, return (header_len, value_len, value_bytes).
fn read_int_tlv(input: &[u8], i: &mut usize, name: &str) -> Result<(usize, usize, Vec<u8>), String> {
    if *i + 2 > input.len() {
        return Err(format!("truncated {} header", name));
    }
    if input[*i] != 0x02 {
        return Err(format!("{} tag mismatch: expected 0x02 got 0x{:02X}", name, input[*i]));
    }
    let len = input[*i + 1] as usize;
    if len == 0 || len > 4 {
        return Err(format!("{} length out of range: {}", name, len));
    }
    if *i + 2 + len > input.len() {
        return Err(format!("truncated {} body", name));
    }
    let value = input[*i + 2..*i + 2 + len].to_vec();
    *i += 2 + len;
    Ok((2, len, value))
}

/// Read an OCTET STRING TLV at `i`, advance `i`, return (header_len, value_len).
fn read_octet_string_tlv(input: &[u8], i: &mut usize, name: &str) -> Result<(usize, usize), String> {
    if *i + 2 > input.len() {
        return Err(format!("truncated {} header", name));
    }
    if input[*i] != 0x04 {
        return Err(format!("{} tag mismatch: expected 0x04 got 0x{:02X}", name, input[*i]));
    }
    let len_byte = input[*i + 1];
    let (len, header_len) = if len_byte < 0x80 {
        (len_byte as usize, 2)
    } else if len_byte == 0x81 {
        if *i + 3 > input.len() {
            return Err(format!("truncated {} long-form length", name));
        }
        (input[*i + 2] as usize, 3)
    } else if len_byte == 0x82 {
        if *i + 4 > input.len() {
            return Err(format!("truncated {} long-form length", name));
        }
        (((input[*i + 2] as usize) << 8) | (input[*i + 3] as usize), 4)
    } else {
        return Err(format!("{} unsupported length form: 0x{:02X}", name, len_byte));
    };
    if *i + header_len + len > input.len() {
        return Err(format!("truncated {} body", name));
    }
    *i += header_len;
    Ok((header_len, len))
}

/// Read a CONSTRUCTED (SEQUENCE) length at `i`, advance past tag+length, return the
/// payload length. We do not consume the body — callers slice it themselves.
fn read_constructed_len(input: &[u8], i: &mut usize, name: &str) -> Result<usize, String> {
    if *i + 2 > input.len() {
        return Err(format!("truncated {} header", name));
    }
    let len_byte = input[*i + 1];
    let (len, header_len) = if len_byte < 0x80 {
        (len_byte as usize, 2)
    } else if len_byte == 0x81 {
        if *i + 3 > input.len() {
            return Err(format!("truncated {} long-form length", name));
        }
        (input[*i + 2] as usize, 3)
    } else if len_byte == 0x82 {
        if *i + 4 > input.len() {
            return Err(format!("truncated {} long-form length", name));
        }
        (((input[*i + 2] as usize) << 8) | (input[*i + 3] as usize), 4)
    } else {
        return Err(format!("{} unsupported length form: 0x{:02X}", name, len_byte));
    };
    *i += header_len;
    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal-but-valid SNMPv3 envelope with the given flags, sec model,
    /// sec params blob, and scopedPDU body. Useful for the table of cases below.
    fn build_msg(flags_byte: u8, sec_model: u8, params: &[u8], scoped_pdu: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        // msgVersion: INTEGER 3
        v.extend_from_slice(&[0x02, 0x01, 0x03]);
        // msgID: INTEGER 1
        v.extend_from_slice(&[0x02, 0x01, 0x01]);
        // msgMaxSize: INTEGER 1500 (long-form 1 byte would also work; use short-form)
        v.extend_from_slice(&[0x02, 0x02, 0x05, 0xDC]);
        // msgFlags: OCTET STRING 1 byte
        v.extend_from_slice(&[0x04, 0x01, flags_byte]);
        // msgSecurityModel: INTEGER sec_model
        v.extend_from_slice(&[0x02, 0x01, sec_model]);
        // msgSecurityParameters: OCTET STRING
        v.push(0x04);
        if params.len() < 0x80 {
            v.push(params.len() as u8);
        } else {
            v.push(0x81);
            v.push(params.len() as u8);
        }
        v.extend_from_slice(params);
        // scopedPDU: SEQUENCE
        v.push(0x30);
        if scoped_pdu.len() < 0x80 {
            v.push(scoped_pdu.len() as u8);
        } else {
            v.push(0x82);
            v.push((scoped_pdu.len() >> 8) as u8);
            v.push((scoped_pdu.len() & 0xFF) as u8);
        }
        v.extend_from_slice(scoped_pdu);
        v
    }

    #[test]
    fn rfc3412_minimal_noauth() {
        // RFC 3412 §6.4 example shape: reportable=1, no auth, no priv.
        // flags = 0b1000_0000 = 0x80 -> security_level=1 (noAuth).
        let pdu_body = vec![0xA0, 0x1C, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x14];
        let bytes = build_msg(0x80, 0x03, &[0xDE, 0xAD, 0xBE, 0xEF], &pdu_body);
        let m = parse(&bytes).expect("parse ok");
        assert_eq!(m.flags_byte_raw, 0x80);
        assert!(m.reportable);
        assert!(!m.auth_flag);
        assert!(!m.priv_flag);
        assert_eq!(m.security_level, 1);
        assert_eq!(m.security_model, 3);
        assert_eq!(m.security_params, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(m.scoped_pdu, pdu_body);
        assert_eq!(m.data, bytes);
    }

    #[test]
    fn rfc3412_auth_priv_flags() {
        // authPriv: auth_flag=1 (bit 5), priv_flag=1 (bit 6). security_level=3.
        // 0b0110_0000 = 0x60
        let pdu_body = vec![0xA0, 0x05, 0x02, 0x01, 0x01];
        let m = parse(&build_msg(0x60, 0x03, &[], &pdu_body)).expect("parse ok");
        assert_eq!(m.flags_byte_raw, 0x60);
        assert!(!m.reportable);
        assert!(m.auth_flag);
        assert!(m.priv_flag);
        assert_eq!(m.security_level, 3);
    }

    #[test]
    fn rfc3412_auth_no_priv() {
        // authNoPriv: only auth_flag=1 (bit 5). security_level=2.
        // 0b0010_0000 = 0x20
        let pdu_body = vec![0xA0, 0x05, 0x02, 0x01, 0x01];
        let m = parse(&build_msg(0x20, 0x03, &[], &pdu_body)).expect("parse ok");
        assert!(m.auth_flag);
        assert!(!m.priv_flag);
        assert_eq!(m.security_level, 2);
    }

    #[test]
    fn flags_zero_is_noauth_no_report() {
        // All-zero flags: no reportable, no auth, no priv, security_level=1.
        let m = parse(&build_msg(0x00, 0x03, &[], &[0xA0, 0x02, 0x05, 0x00]))
            .expect("parse ok");
        assert_eq!(m.security_level, 1);
        assert!(!m.reportable);
    }

    #[test]
    fn wrong_version_is_error() {
        // msgVersion value != 0x03 -> error
        let bytes = vec![0x02, 0x01, 0x02, 0x02, 0x01, 0x01, 0x02, 0x02, 0x05, 0xDC,
                         0x04, 0x01, 0x00, 0x02, 0x01, 0x03, 0x04, 0x00, 0x30, 0x00];
        assert!(parse(&bytes).is_err());
    }

    #[test]
    fn truncated_input_is_error() {
        let bytes = build_msg(0x00, 0x03, &[], &[0xA0, 0x00]);
        // Lop off the last 4 bytes (well into scopedPDU body).
        assert!(parse(&bytes[..bytes.len() - 4]).is_err());
    }

    #[test]
    fn bad_flags_tag_is_error() {
        // Replace OCTET STRING tag (0x04) for msgFlags with INTEGER (0x02).
        let mut bytes = build_msg(0x00, 0x03, &[], &[0xA0, 0x00]);
        // msgFlags starts after version(3) + msgID(3) + msgMaxSize(4) = offset 10
        bytes[10] = 0x02;
        assert!(parse(&bytes).is_err());
    }

    #[test]
    fn long_form_octet_string_params() {
        // A security params blob >=128 bytes forces 1-byte long-form length.
        let big_params = vec![0xAA; 200];
        let pdu = vec![0xA0, 0x02, 0x05, 0x00];
        let m = parse(&build_msg(0x00, 0x03, &big_params, &pdu)).expect("parse ok");
        assert_eq!(m.security_params.len(), 200);
        assert_eq!(m.security_params, big_params);
    }

    #[test]
    fn long_form_constructed_pdu() {
        // A scopedPDU body >=256 bytes forces 2-byte long-form length.
        let big_pdu = vec![0x00; 300];
        let m = parse(&build_msg(0x00, 0x03, &[], &big_pdu)).expect("parse ok");
        assert_eq!(m.scoped_pdu.len(), 300);
    }
}