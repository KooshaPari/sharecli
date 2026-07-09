// Minimal DHCP option codec (RFC 2132 / RFC 1497). Parses and renders DHCP
// option TLVs (type, length, value). Length is a single byte (max 255 bytes
// per option). Handles the common subset of options: padding (0), end (255),
// subnet mask (1), router (3), DNS (6), hostname (12), MTU (26), requested IP
// (50), lease time (51), message type (53), server id (54), parameter
// request list (55), and a generic fallback for any unknown option (stored
// as raw bytes).
//
// This does NOT parse DHCP message structure — only the options block inside
// a DHCP packet (or BOOTP vendor area).

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DhcpOption {
    Pad,
    End,
    SubnetMask([u8; 4]),
    Router(Vec<u8>),
    Dns(Vec<u8>),
    Hostname(String),
    Mtu(u16),
    RequestedIp([u8; 4]),
    LeaseTime(u32),
    MessageType(u8),
    ServerId([u8; 4]),
    ParamRequest(Vec<u8>),
    Unknown { code: u8, value: Vec<u8> },
}

pub fn parse_options(data: &[u8]) -> Result<Vec<DhcpOption>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let code = data[i];
        if code == 0 {
            out.push(DhcpOption::Pad);
            i += 1;
            continue;
        }
        if code == 255 {
            out.push(DhcpOption::End);
            return Ok(out);
        }
        if i + 1 >= data.len() {
            return Err("truncated option header".into());
        }
        let len = data[i + 1] as usize;
        if i + 2 + len > data.len() {
            return Err("truncated option value".into());
        }
        let value = &data[i + 2..i + 2 + len];
        out.push(decode_option(code, value));
        i += 2 + len;
    }
    Ok(out)
}

fn decode_option(code: u8, value: &[u8]) -> DhcpOption {
    match code {
        1 if value.len() == 4 => DhcpOption::SubnetMask([value[0], value[1], value[2], value[3]]),
        3 => DhcpOption::Router(value.to_vec()),
        6 => DhcpOption::Dns(value.to_vec()),
        12 => DhcpOption::Hostname(std::str::from_utf8(value).unwrap_or("").to_string()),
        26 if value.len() == 2 => DhcpOption::Mtu(u16::from_be_bytes([value[0], value[1]])),
        50 if value.len() == 4 => DhcpOption::RequestedIp([value[0], value[1], value[2], value[3]]),
        51 if value.len() == 4 => {
            DhcpOption::LeaseTime(u32::from_be_bytes([value[0], value[1], value[2], value[3]]))
        }
        53 if value.len() == 1 => DhcpOption::MessageType(value[0]),
        54 if value.len() == 4 => DhcpOption::ServerId([value[0], value[1], value[2], value[3]]),
        55 => DhcpOption::ParamRequest(value.to_vec()),
        _ => DhcpOption::Unknown { code, value: value.to_vec() },
    }
}

pub fn encode_options(opts: &[DhcpOption]) -> Vec<u8> {
    let mut out = Vec::new();
    for opt in opts {
        match opt {
            DhcpOption::Pad => out.push(0),
            DhcpOption::End => {
                out.push(255);
                return out;
            }
            DhcpOption::SubnetMask(ip) => {
                out.push(1);
                out.push(4);
                out.extend_from_slice(ip);
            }
            DhcpOption::Router(v) => {
                out.push(3);
                out.push(v.len() as u8);
                out.extend_from_slice(v);
            }
            DhcpOption::Dns(v) => {
                out.push(6);
                out.push(v.len() as u8);
                out.extend_from_slice(v);
            }
            DhcpOption::Hostname(s) => {
                let b = s.as_bytes();
                out.push(12);
                out.push(b.len() as u8);
                out.extend_from_slice(b);
            }
            DhcpOption::Mtu(m) => {
                out.push(26);
                out.push(2);
                out.extend_from_slice(&m.to_be_bytes());
            }
            DhcpOption::RequestedIp(ip) => {
                out.push(50);
                out.push(4);
                out.extend_from_slice(ip);
            }
            DhcpOption::LeaseTime(t) => {
                out.push(51);
                out.push(4);
                out.extend_from_slice(&t.to_be_bytes());
            }
            DhcpOption::MessageType(t) => {
                out.push(53);
                out.push(1);
                out.push(*t);
            }
            DhcpOption::ServerId(ip) => {
                out.push(54);
                out.push(4);
                out.extend_from_slice(ip);
            }
            DhcpOption::ParamRequest(v) => {
                out.push(55);
                out.push(v.len() as u8);
                out.extend_from_slice(v);
            }
            DhcpOption::Unknown { code, value } => {
                out.push(*code);
                out.push(value.len() as u8);
                out.extend_from_slice(value);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_pad_end() {
        let data = vec![0, 0, 255];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::Pad, DhcpOption::Pad, DhcpOption::End]);
    }
    #[test]
    fn parse_subnet_mask() {
        let data = vec![1, 4, 255, 255, 255, 0];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::SubnetMask([255, 255, 255, 0])]);
    }
    #[test]
    fn parse_message_type_offer() {
        let data = vec![53, 1, 2];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::MessageType(2)]);
    }
    #[test]
    fn parse_mtu() {
        let data = vec![26, 2, 0x05, 0xdc];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::Mtu(1500)]);
    }
    #[test]
    fn parse_lease_time() {
        let data = vec![51, 4, 0, 1, 0x51, 0x80];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::LeaseTime(86400)]);
    }
    #[test]
    fn parse_hostname() {
        let data = vec![12, 5, b'h', b'o', b's', b't', b'1'];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::Hostname("host1".into())]);
    }
    #[test]
    fn parse_unknown_option() {
        let data = vec![99, 3, 1, 2, 3];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::Unknown { code: 99, value: vec![1, 2, 3] }]);
    }
    #[test]
    fn parse_truncated_value() {
        let data = vec![1, 4, 255, 255];
        assert!(parse_options(&data).is_err());
    }
    #[test]
    fn parse_truncated_header() {
        let data = vec![12];
        assert!(parse_options(&data).is_err());
    }
    #[test]
    fn parse_param_request() {
        let data = vec![55, 4, 1, 3, 6, 15];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts, vec![DhcpOption::ParamRequest(vec![1, 3, 6, 15])]);
    }
    #[test]
    fn round_trip() {
        let opts = vec![
            DhcpOption::SubnetMask([255, 255, 255, 0]),
            DhcpOption::Router(vec![192, 168, 1, 1]),
            DhcpOption::LeaseTime(3600),
            DhcpOption::End,
        ];
        let bytes = encode_options(&opts);
        let parsed = parse_options(&bytes).unwrap();
        assert_eq!(parsed, opts);
    }
    #[test]
    fn end_returns_remaining_ignored() {
        // End marker should return immediately, ignoring any trailing bytes.
        let data = vec![1, 4, 255, 255, 255, 0, 99, 1, 1, 255];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts[0], DhcpOption::SubnetMask([255, 255, 255, 0]));
        assert_eq!(opts.last(), Some(&DhcpOption::End));
    }
    #[test]
    fn multiple_options_with_pad() {
        let data = vec![0, 53, 1, 1, 0, 51, 4, 0, 0, 0, 60, 255];
        let opts = parse_options(&data).unwrap();
        assert_eq!(opts.len(), 5);
        assert_eq!(opts[1], DhcpOption::MessageType(1));
        assert_eq!(opts[3], DhcpOption::LeaseTime(60));
    }
}
