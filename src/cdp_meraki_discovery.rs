// Minimal CDP (Cisco Discovery Protocol) TLV parser with a Meraki-specific
// accommodation.
//
// CDP runs directly on top of Ethernet (Ethertype 0x2000, IEEE 802.3 LLC
// DSAP/SSAP 0xAA/0xAA, OUI 0x00000C, PID 0x2000). The body after the LLC
// header is a single 32-bit "checksum" field — which, per RFC-style CDP
// semantics and real captures from Meraki APs, is NOT a TCP/UDP-style
// checksum but is included for forward compatibility. We accept the
// checksum and ignore it; CDP receivers are required to either validate it
// or treat the packet as authentic on the wire (Meraki MR access points
// emit it verbatim, with the checksum often set to zero).
//
// The CDP body (after the LLC header) consists of concatenated TLVs in the
// form:
//
//   +-----------+---------+-----------+--- ... ---+
//   |  Type (2) | Length(2)|  Value   (Length-4 bytes, may be 0)
//   +-----------+---------+-----------+--- ... ---+
//
// where Length counts the Type+Length fields themselves (i.e. Length >= 4
// and Value length = Length - 4). The list ends when fewer than 4 bytes
// remain.
//
// We preserve the TLV wire type in `CdpTlv::type_id` and the raw bytes in
// `CdpTlv::value`. Meraki-specific OUI-encapsulated payloads (such as the
// power-over-Ethernet negotiation TLV) are returned with their full value
// blob — callers can post-process them by TLV type.
//
// This module does NOT attempt to validate the CDP checksum; pass a slice
// that starts at the first TLV (i.e. after the LLC + 32-bit CDP checksum).

/// A single CDP TLV, in wire order.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CdpTlv {
    /// 16-bit TLV type. Common values (per Cisco CDP spec + Meraki docs):
    /// 0x0001 Device ID, 0x0002 Address, 0x0003 Port ID, 0x0004 Capabilities,
    /// 0x0005 Software Version, 0x0006 Platform, 0x0007 IP Prefix (unused),
    /// 0x0008 Protocol Hello, 0x0009 VTP Management Domain, 0x000A Native VLAN,
    /// 0x000B Duplex, 0x000C Appliance VLAN ID, 0x000D Power, 0x000E MTU,
    /// 0x000F Trust Bitmap, 0x0010 Untrusted Port CoS, 0x0011 System Name,
    /// 0x0012 System OID, 0x0013 Management Address, 0x0014 Physical Location,
    /// 0x0015 LLDP-MED capabilities, 0x001A EnergyWise, 0x001B Spare Pair,
    /// 0x001F Meraki-specific TLVs begin (Power negotiation block + others).
    pub type_id: u16,
    /// Raw value bytes, with length = (Length_field - 4).
    pub value: Vec<u8>,
}

/// Parse a CDP body (post-LLC, post-checksum) into a list of TLVs.
///
/// On success, returns the list. On a structural error (truncated TLV, length
/// field smaller than the TLV header, length field claiming more bytes than
/// are left in `input`), returns a `String` describing the first fault.
pub fn parse(input: &[u8]) -> Result<Vec<CdpTlv>, String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < input.len() {
        if input.len() - i < 4 {
            // Fewer than 4 bytes remain: per CDP spec the TLV header alone
            // is 4 bytes; anything shorter is end-of-list or a malformed
            // tail. We stop here.
            break;
        }
        let type_id = u16::from_be_bytes([input[i], input[i + 1]]);
        let length = u16::from_be_bytes([input[i + 2], input[i + 3]]) as usize;
        if length < 4 {
            return Err(format!(
                "CDP TLV at offset {} has length {} (< 4): cannot fit Type+Length header",
                i, length
            ));
        }
        let value_len = length - 4;
        if i + length > input.len() {
            return Err(format!(
                "CDP TLV at offset {} (type 0x{:04X}) claims {} bytes but only {} remain",
                i,
                type_id,
                length,
                input.len() - i
            ));
        }
        let value = input[i + 4..i + length].to_vec();
        out.push(CdpTlv { type_id, value });
        i += length;
    }
    Ok(out)
}

/// Convenience: format a TLV's value as an ASCII string if it looks like
/// printable ASCII + NUL-terminated (as is common for Device ID, Port ID,
/// Software Version, Platform). Returns `None` if the value is not printable.
pub fn maybe_ascii(value: &[u8]) -> Option<&str> {
    if value.is_empty() {
        return None;
    }
    // Trim trailing NUL if present (very common in CDP string TLVs).
    let trimmed = if value.last() == Some(&0) {
        &value[..value.len() - 1]
    } else {
        value
    };
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.iter().all(|b| (0x20..=0x7E).contains(b)) {
        return None;
    }
    std::str::from_utf8(trimmed).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a TLV with the given (type, value) pair. Length is value.len()+4.
    fn tlv(type_id: u16, value: &[u8]) -> Vec<u8> {
        let length = (value.len() + 4) as u16;
        let mut v = Vec::new();
        v.extend_from_slice(&type_id.to_be_bytes());
        v.extend_from_slice(&length.to_be_bytes());
        v.extend_from_slice(value);
        v
    }

    #[test]
    fn parses_two_simple_tlvs() {
        // TLV 0x0001 Device ID = "sw1.example.com\0" (CDP standard)
        let mut body = Vec::new();
        body.extend_from_slice(&tlv(0x0001, b"sw1.example.com\0"));
        body.extend_from_slice(&tlv(0x0003, b"GigabitEthernet0/1\0"));
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs.len(), 2);
        assert_eq!(tlvs[0].type_id, 0x0001);
        assert_eq!(tlvs[0].value, b"sw1.example.com\0");
        assert_eq!(tlvs[1].type_id, 0x0003);
        assert_eq!(tlvs[1].value, b"GigabitEthernet0/1\0");
    }

    #[test]
    fn cisco_cdp_device_id_format() {
        // Real-shape Device ID TLV (CDP, RFC-style example from Cisco docs):
        // type 0x0001, length 22 = 18-byte string + 4-byte header.
        // "router1.cisco.com\0" is 19 bytes, so length=23.
        let dev = b"router1.cisco.com\0";
        let body = tlv(0x0001, dev);
        // Verify our test math: length should be dev.len()+4
        assert_eq!(u16::from_be_bytes([body[2], body[3]]) as usize, dev.len() + 4);
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs.len(), 1);
        assert_eq!(tlvs[0].type_id, 0x0001);
        assert_eq!(maybe_ascii(&tlvs[0].value), Some("router1.cisco.com"));
    }

    #[test]
    fn meraki_power_negotiation_tlv() {
        // Meraki APs emit a Meraki-OUI Power negotiation TLV with type 0x001F.
        // The value blob is a sequence of 16-bit little-endian integers per
        // the Meraki private spec. We just verify we round-trip it.
        let meraki_power = vec![0x01, 0x00, 0x2C, 0x01, 0x00, 0x00];
        let body = tlv(0x001F, &meraki_power);
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs.len(), 1);
        assert_eq!(tlvs[0].type_id, 0x001F);
        assert_eq!(tlvs[0].value, meraki_power);
    }

    #[test]
    fn capabilities_tlv_preserves_bitmap() {
        // Capabilities TLV (type 0x0004) is a 32-bit capability bitmap.
        // Per CDP: bit 0=router, 1=bridge, 2=host, 3=igmp, etc.
        // Use 0x00000021 (router + host) as a real-shape value.
        let caps: [u8; 4] = [0x00, 0x00, 0x00, 0x21];
        let body = tlv(0x0004, &caps);
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs[0].type_id, 0x0004);
        assert_eq!(tlvs[0].value, &caps[..]);
    }

    #[test]
    fn empty_input_yields_empty_list() {
        assert_eq!(parse(&[]).unwrap(), Vec::new());
    }

    #[test]
    fn trailing_partial_header_stops_cleanly() {
        // End-of-buffer partial header (< 4 bytes) should not error.
        let body = vec![0x00, 0x01, 0x00]; // only 3 bytes
        let tlvs = parse(&body).expect("parse ok");
        assert!(tlvs.is_empty());
    }

    #[test]
    fn length_smaller_than_header_is_error() {
        // Length field = 3 (< 4): structurally invalid TLV header.
        let body = vec![0x00, 0x01, 0x00, 0x03];
        assert!(parse(&body).is_err());
    }

    #[test]
    fn length_overshoots_buffer_is_error() {
        // Length field claims 20 bytes but only 6 follow.
        let body = vec![0x00, 0x01, 0x00, 0x14, 0xAA, 0xBB];
        assert!(parse(&body).is_err());
    }

    #[test]
    fn concatenated_tlvs_with_varied_lengths() {
        // TLV 0x000A Native VLAN, 2-byte value 0x0064 (100).
        let mut body = Vec::new();
        body.extend_from_slice(&tlv(0x000A, &[0x00, 0x64]));
        // TLV 0x000B Duplex, 1-byte value 0x01 (full-duplex).
        body.extend_from_slice(&tlv(0x000B, &[0x01]));
        // TLV 0x000E MTU, 4-byte value 0x000005DC (1500).
        body.extend_from_slice(&tlv(0x000E, &[0x00, 0x00, 0x05, 0xDC]));
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs.len(), 3);
        assert_eq!(tlvs[0].type_id, 0x000A);
        assert_eq!(tlvs[0].value, vec![0x00, 0x64]);
        assert_eq!(tlvs[1].type_id, 0x000B);
        assert_eq!(tlvs[1].value, vec![0x01]);
        assert_eq!(tlvs[2].type_id, 0x000E);
        assert_eq!(tlvs[2].value, vec![0x00, 0x00, 0x05, 0xDC]);
    }

    #[test]
    fn zero_length_value_tlv() {
        // A TLV with value length 0 is structurally allowed (Length=4).
        let body = vec![0x00, 0x0D, 0x00, 0x04]; // type 0x000D (Power), no value
        let tlvs = parse(&body).expect("parse ok");
        assert_eq!(tlvs.len(), 1);
        assert_eq!(tlvs[0].type_id, 0x000D);
        assert!(tlvs[0].value.is_empty());
    }

    #[test]
    fn ascii_helper_handles_non_printable() {
        // A TLV whose value contains a control byte should not be coerced.
        assert_eq!(maybe_ascii(b"hello\0"), Some("hello"));
        assert_eq!(maybe_ascii(&[0x01, 0x02]), None);
        assert_eq!(maybe_ascii(&[]), None);
        // All-NUL value trims to empty -> None.
        assert_eq!(maybe_ascii(&[0x00]), None);
    }
}