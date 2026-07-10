// Minimal DHCPv6 message parser (RFC 8415).
//
// DHCPv6 (Dynamic Host Configuration Protocol for IPv6) carries configuration
// state between clients and servers. A DHCPv6 message has the shape:
//
//     0                   1                   2                   3
//     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |    msg-type   |               transaction-id                  |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                                                               |
//    .                            options                            .
//    .                          (variable)                           .
//    |                                                               |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//
// Per RFC 8415 §8.1:
//   * msg-type is a single octet identifying the DHCP message type
//     (e.g. SOLICIT=1, ADVERTISE=2, REQUEST=3, REPLY=7).
//   * transaction-id is a 24-bit (3-byte) identifier chosen by the client.
//
// Each option has its own header (RFC 8415 §21.1):
//
//     0                   1                   2                   3
//     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |        option-code            |           option-len          |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                          option-data                          |
//    |                        (option-len octets)                    |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//
// We expose a tiny `parse` that captures the message header, walks every
// option, and surfaces any truncation or length mismatch as a `String` error.

/// DHCPv6 message types we recognize by name. From RFC 8415 §8.1, table 1
/// ("Message Types"). Values outside this set are returned as raw `u8`.
pub const SOLICIT: u8 = 1;
pub const ADVERTISE: u8 = 2;
pub const REQUEST: u8 = 3;
pub const CONFIRM: u8 = 4;
pub const RENEW: u8 = 5;
pub const REBIND: u8 = 6;
pub const REPLY: u8 = 7;
pub const RELEASE: u8 = 8;
pub const DECLINE: u8 = 9;
pub const RECONFIGURE: u8 = 10;
pub const INFORMATION_REQUEST: u8 = 11;
pub const RELAY_FORW: u8 = 12;
pub const RELAY_REPL: u8 = 13;

/// Common DHCPv6 option codes (RFC 8415 §21 and other RFCs assigning them).
pub const OPTION_CLIENTID: u16 = 1;
pub const OPTION_SERVERID: u16 = 2;
pub const OPTION_IA_NA: u16 = 3;
pub const OPTION_IA_TA: u16 = 4;
pub const OPTION_IAADDR: u16 = 5;
pub const OPTION_ORO: u16 = 6;
pub const OPTION_PREFERENCE: u16 = 7;
pub const OPTION_ELAPSED_TIME: u16 = 8;
pub const OPTION_RELAY_MSG: u16 = 9;
pub const OPTION_AUTH: u16 = 11;
pub const OPTION_UNICAST: u16 = 12;
pub const OPTION_STATUS_CODE: u16 = 13;
pub const OPTION_RAPID_COMMIT: u16 = 14;
pub const OPTION_USER_CLASS: u16 = 15;
pub const OPTION_VENDOR_CLASS: u16 = 16;
pub const OPTION_VENDOR_OPTS: u16 = 17;
pub const OPTION_INTERFACE_ID: u16 = 18;
pub const OPTION_RECONF_MSG: u16 = 19;
pub const OPTION_DNS_SERVERS: u16 = 23;
pub const OPTION_DOMAIN_LIST: u16 = 24;

/// A single DHCPv6 option, header-decoded only. The value is left as opaque
/// bytes — sub-decoding (IA_NA, IAADDR, status codes, ...) is the caller's job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6Option {
    /// 16-bit option code from RFC 8415 §21.
    pub code: u16,
    /// Raw option-value bytes; length equals the original `option-len`.
    pub value: Vec<u8>,
}

/// A parsed DHCPv6 message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6Msg {
    /// Message type octet (RFC 8415 §8.1).
    pub msg_type: u8,
    /// 24-bit transaction-id from RFC 8415 §8.1, in network byte order
    /// (i.e. first byte is the high byte).
    pub transaction_id: [u8; 3],
    /// Decoded options in wire order.
    pub options: Vec<Dhcp6Option>,
}

/// Return the symbolic name for a DHCPv6 message type, or `None` if the
/// numeric value is not in the IANA registry (RFC 8415 §8.1 table).
pub fn msg_type_name(t: u8) -> Option<&'static str> {
    match t {
        SOLICIT => Some("SOLICIT"),
        ADVERTISE => Some("ADVERTISE"),
        REQUEST => Some("REQUEST"),
        CONFIRM => Some("CONFIRM"),
        RENEW => Some("RENEW"),
        REBIND => Some("REBIND"),
        REPLY => Some("REPLY"),
        RELEASE => Some("RELEASE"),
        DECLINE => Some("DECLINE"),
        RECONFIGURE => Some("RECONFIGURE"),
        INFORMATION_REQUEST => Some("INFORMATION-REQUEST"),
        RELAY_FORW => Some("RELAY-FORW"),
        RELAY_REPL => Some("RELAY-REPL"),
        _ => None,
    }
}

/// Parse a DHCPv6 message. Returns the decoded message and any trailing bytes
/// (multiple DHCPv6 messages can theoretically be carried in a single UDP
/// datagram in some relay scenarios, so we surface a slice instead of forcing
/// the caller to size buffers exactly).
pub fn parse(input: &[u8]) -> Result<(Dhcp6Msg, &[u8]), String> {
    if input.len() < 4 {
        return Err(format!("dhcpv6: header too short (need 4 bytes, got {})", input.len()));
    }
    let msg_type = input[0];
    let transaction_id = [input[1], input[2], input[3]];
    let mut options = Vec::new();
    let mut rest = &input[4..];

    while !rest.is_empty() {
        if rest.len() < 4 {
            return Err(format!(
                "dhcpv6: truncated option header (need 4 bytes, got {})",
                rest.len()
            ));
        }
        let code = u16::from_be_bytes([rest[0], rest[1]]);
        let len = u16::from_be_bytes([rest[2], rest[3]]) as usize;
        let after_header = &rest[4..];
        if after_header.len() < len {
            return Err(format!(
                "dhcpv6: option {} truncated (need {} bytes, got {})",
                code,
                len,
                after_header.len()
            ));
        }
        let (value, after_option) = after_header.split_at(len);
        options.push(Dhcp6Option { code, value: value.to_vec() });
        rest = after_option;
    }

    Ok((Dhcp6Msg { msg_type, transaction_id, options }, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_rejected() {
        let err = parse(&[]).expect_err("must reject");
        assert!(err.contains("header too short"), "got: {}", err);
    }

    #[test]
    fn header_only_parses() {
        // msg-type = SOLICIT, xid = 0x000001, no options.
        let bytes = [0x01, 0x00, 0x00, 0x01];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.msg_type, SOLICIT);
        assert_eq!(msg.transaction_id, [0x00, 0x00, 0x01]);
        assert!(msg.options.is_empty());
    }

    #[test]
    fn parse_solicit_with_clientid_and_oro() {
        // msg-type = SOLICIT (1), xid = 0xABCDEF.
        // Option 1 (CLIENTID) length=4 value=0xDEADBEEF.
        // Option 6 (ORO)     length=4 value=[0x00,0x17,0x00,0x18] (DNS_SERVERS, DOMAIN_LIST).
        let bytes = [
            0x01, 0xAB, 0xCD, 0xEF, // CLIENTID
            0x00, 0x01, 0x00, 0x04, 0xDE, 0xAD, 0xBE, 0xEF, // ORO
            0x00, 0x06, 0x00, 0x04, 0x00, 0x17, 0x00, 0x18,
        ];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.msg_type, SOLICIT);
        assert_eq!(msg.transaction_id, [0xAB, 0xCD, 0xEF]);
        assert_eq!(msg.options.len(), 2);
        assert_eq!(msg.options[0].code, OPTION_CLIENTID);
        assert_eq!(msg.options[0].value, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(msg.options[1].code, OPTION_ORO);
        assert_eq!(msg.options[1].value, vec![0x00, 0x17, 0x00, 0x18]);
    }

    #[test]
    fn parse_reply_with_dns_servers() {
        // ADVERTISE / REPLY would carry DNS_SERVERS as 16-byte IPv6 addresses.
        // Reply (7), xid = 0x000002, DNS_SERVERS option 23 with two addresses
        // (32 bytes total payload).
        let bytes = [
            0x07, 0x00, 0x00, 0x02, // DNS_SERVERS option header
            0x00, 0x17, 0x00, 0x20, // IPv6 2001:db8::1
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // IPv6 2001:db8::2
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02,
        ];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.msg_type, REPLY);
        assert_eq!(msg.options.len(), 1);
        let opt = &msg.options[0];
        assert_eq!(opt.code, OPTION_DNS_SERVERS);
        assert_eq!(opt.value.len(), 32);
        // First 16 bytes are the IPv6 address 2001:db8::1.
        assert_eq!(
            &opt.value[0..16],
            &[0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01]
        );
        assert_eq!(
            &opt.value[16..32],
            &[0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02]
        );
    }

    #[test]
    fn truncated_option_length_rejected() {
        // Header ok, then an option that says length=8 with only 3 bytes of value.
        let bytes = [0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x08, 0xAA, 0xBB, 0xCC];
        let err = parse(&bytes).expect_err("must reject");
        assert!(err.contains("truncated"), "got: {}", err);
    }

    #[test]
    fn truncated_option_header_rejected() {
        // Header ok, but only 2 bytes left when we need 4 for the next option header.
        let bytes = [0x01, 0x00, 0x00, 0x01, 0x00, 0x01];
        let err = parse(&bytes).expect_err("must reject");
        assert!(err.contains("truncated"), "got: {}", err);
    }

    #[test]
    fn empty_option_value_allowed() {
        // An option with code=23 (DNS_SERVERS), length=0 is legal — it just
        // carries no server addresses. RFC 8415 §21.1 says the option-len
        // may legitimately be zero.
        let bytes = [0x01, 0x00, 0x00, 0x01, 0x00, 0x17, 0x00, 0x00];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.options.len(), 1);
        assert_eq!(msg.options[0].code, OPTION_DNS_SERVERS);
        assert!(msg.options[0].value.is_empty());
    }

    #[test]
    fn msg_type_names() {
        assert_eq!(msg_type_name(SOLICIT), Some("SOLICIT"));
        assert_eq!(msg_type_name(ADVERTISE), Some("ADVERTISE"));
        assert_eq!(msg_type_name(REQUEST), Some("REQUEST"));
        assert_eq!(msg_type_name(REPLY), Some("REPLY"));
        assert_eq!(msg_type_name(RECONFIGURE), Some("RECONFIGURE"));
        assert_eq!(msg_type_name(200), None);
    }

    #[test]
    fn multiple_options_in_order() {
        // Three options back-to-back to verify ordering and accumulation.
        let bytes = [
            0x03, 0x00, 0x00, 0x0A, // REQUEST
            0x00, 0x01, 0x00, 0x02, 0xCA, 0xFE, // CLIENTID
            0x00, 0x02, 0x00, 0x02, 0xBE, 0xEF, // SERVERID
            0x00, 0x08, 0x00, 0x02, 0x00, 0x64, // ELAPSED_TIME = 100
        ];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.msg_type, REQUEST);
        assert_eq!(msg.options.len(), 3);
        assert_eq!(msg.options[0].code, OPTION_CLIENTID);
        assert_eq!(msg.options[1].code, OPTION_SERVERID);
        assert_eq!(msg.options[2].code, OPTION_ELAPSED_TIME);
        assert_eq!(msg.options[2].value, vec![0x00, 0x64]);
    }

    #[test]
    fn rfc8415_section8_example_wire_format() {
        // RFC 8415 §8.6 ("Transmission of DHCPv6 Messages") describes the
        // 4-octet message header. Verify our header decodes the exact field
        // widths: msg-type (1 byte) | transaction-id (3 bytes).
        let bytes = [0x07, 0x12, 0x34, 0x56];
        let (msg, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(msg.msg_type, 0x07); // REPLY
        assert_eq!(msg.transaction_id, [0x12, 0x34, 0x56]);
    }
}
