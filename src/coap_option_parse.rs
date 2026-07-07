// Minimal CoAP Option codec (RFC 7252, Section 3.1).
//
// CoAP options appear between the CoAP header and the optional payload, in
// order of ascending Option Number. Each option is encoded as:
//
//   +---------+--------+---+---+---+---+--------+--------+---------+---------+
//   |  Delta  | Length |   Option Value (Length bytes) ...
//   +---------+--------+---+---+---+---+--------+--------+---------+---------+
//
// Both Delta and Length occupy the low/high nibbles of the first byte and use
// the following special values:
//
//   0..12  -> the literal value (0..12)
//   13     -> the value 13 plus the following unsigned byte  (0..=13+255=268)
//   14     -> the value 269 plus the following two unsigned bytes  (0..=269+65535)
//   15     -> reserved for the Payload Marker (not a real option)
//
// The actual Option Number is computed incrementally: each option's delta is
// added to the previous option's number. Delta=0 is valid for repeated options.
//
// This module only handles the options block, not the full CoAP message header.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CoapOption {
    pub number: u16,
    pub value: Vec<u8>,
}

const DELTA_EXT_1_BYTE: u8 = 13;
const DELTA_EXT_2_BYTE: u8 = 14;
const DELTA_PAYLOAD_MARKER: u8 = 15;
const LEN_EXT_1_BYTE: u8 = 13;
const LEN_EXT_2_BYTE: u8 = 14;

pub fn parse_options(input: &[u8]) -> Result<Vec<CoapOption>, String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut prev_number: u16 = 0;

    while i < input.len() {
        let first = input[i];
        if first == 0xFF {
            // Payload marker (0xFF is the byte, but in CoAP the first byte
            // equal to 0xFF means payload marker; delta/length would never
            // legitimately be 15/15 since 15 is reserved).
            return Err("payload marker (0xFF) encountered; remaining bytes belong to payload".into());
        }
        let delta_nibble = (first >> 4) & 0x0F;
        let length_nibble = first & 0x0F;
        i += 1;

        let delta = match delta_nibble {
            0..=12 => delta_nibble as u16,
            DELTA_EXT_1_BYTE => {
                if i >= input.len() { return Err("truncated extended delta".into()); }
                let v = input[i] as u16;
                i += 1;
                v + 13
            }
            DELTA_EXT_2_BYTE => {
                if i + 1 >= input.len() { return Err("truncated extended delta (2-byte)".into()); }
                let v = u16::from_be_bytes([input[i], input[i + 1]]);
                i += 2;
                v + 269
            }
            DELTA_PAYLOAD_MARKER => return Err("delta nibble 15 is reserved (payload marker)".into()),
            _ => unreachable!(),
        };

        let length = match length_nibble {
            0..=12 => length_nibble as usize,
            LEN_EXT_1_BYTE => {
                if i >= input.len() { return Err("truncated extended length".into()); }
                let v = input[i] as usize;
                i += 1;
                v + 13
            }
            LEN_EXT_2_BYTE => {
                if i + 1 >= input.len() { return Err("truncated extended length (2-byte)".into()); }
                let v = u16::from_be_bytes([input[i], input[i + 1]]) as usize;
                i += 2;
                v + 269
            }
            _ => unreachable!(),
        };

        if i + length > input.len() {
            return Err(format!("option value truncated (need {} bytes, have {})", length, input.len() - i));
        }
        let value = input[i..i + length].to_vec();
        i += length;

        // Option number = previous + delta.  Sum may overflow u16.
        let number = prev_number.checked_add(delta)
            .ok_or_else(|| "option number overflow".to_string())?;
        prev_number = number;
        out.push(CoapOption { number, value });
    }

    Ok(out)
}

pub fn encode_options(opts: &[CoapOption]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut prev_number: u16 = 0;
    for opt in opts {
        let delta = opt.number - prev_number;
        let mut delta_bytes: Vec<u8> = Vec::new();
        let delta_nibble = if delta <= 12 {
            delta as u8
        } else if delta <= 12 + 255 {
            delta_bytes.push((delta - 13) as u8);
            DELTA_EXT_1_BYTE
        } else if delta >= 269 {
            // Max 2-byte delta = 269 + 65535; if `delta >= 269` we know it
            // fits in u16 (CoAP option numbers are u16).
            let v = delta - 269;
            delta_bytes.extend_from_slice(&v.to_be_bytes());
            DELTA_EXT_2_BYTE
        } else {
            0
        };

        let length = opt.value.len();
        let mut length_bytes: Vec<u8> = Vec::new();
        let length_nibble = if length <= 12 {
            length as u8
        } else if length <= 12 + 255 {
            length_bytes.push((length - 13) as u8);
            LEN_EXT_1_BYTE
        } else if length >= 269 {
            let v = length - 269;
            length_bytes.extend_from_slice(&v.to_be_bytes());
            LEN_EXT_2_BYTE
        } else {
            0
        };

        out.push((delta_nibble << 4) | length_nibble);
        out.extend_from_slice(&delta_bytes);
        out.extend_from_slice(&length_bytes);
        out.extend_from_slice(&opt.value);
        prev_number = opt.number;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_single_option() {
        // Option 11 (Uri-Path) = "a", delta=11, length=1
        // byte 0 = (11<<4) | 1 = 0xB1
        let bytes = vec![0xB1, b'a'];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![CoapOption { number: 11, value: vec![b'a'] }]);
    }

    #[test]
    fn multiple_options_ascending_numbers() {
        // Opt 1 (delta=1, len=0) followed by Opt 5 Uri-Path = "b" (delta=4, len=1)
        // byte0 = (1<<4)|0 = 0x10
        // byte1 = (4<<4)|1 = 0x41, value 'b'
        let bytes = vec![0x10, 0x41, b'b'];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![
            CoapOption { number: 1, value: vec![] },
            CoapOption { number: 5, value: vec![b'b'] },
        ]);
    }

    #[test]
    fn empty_options() {
        let opts = parse_options(&[]).unwrap();
        assert!(opts.is_empty());
    }

    #[test]
    fn extended_delta_one_byte() {
        // First option: number 200 (delta=200, len=0). Delta fits in 1-byte extended (delta - 13 <= 255)
        // byte0 = (13<<4)|0 = 0xD0, then byte 1 = (200 - 13) = 187 = 0xBB
        let bytes = vec![0xD0, 0xBB];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![CoapOption { number: 200, value: vec![] }]);
    }

    #[test]
    fn extended_delta_two_byte() {
        // First option: number 1000 (delta=1000). 1000 > 268, so 2-byte extended.
        // byte0 = (14<<4)|0 = 0xE0, then bytes = 1000-269 = 731 = 0x02DB
        let bytes = vec![0xE0, 0x02, 0xDB];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![CoapOption { number: 1000, value: vec![] }]);
    }

    #[test]
    fn extended_length_one_byte() {
        // Opt 1, value of 100 bytes. Length = 100 > 12, fits 1-byte extended.
        // byte0 = (1<<4)|13 = 0x1D, then byte = 100-13 = 87, then 100 value bytes
        let mut bytes = vec![0x1D, 87];
        let value: Vec<u8> = (0..100u8).collect();
        bytes.extend_from_slice(&value);
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![CoapOption { number: 1, value }]);
    }

    #[test]
    fn extended_length_two_byte() {
        // Opt 1, value of 500 bytes. Length = 500 > 268, so 2-byte extended.
        // byte0 = (1<<4)|14 = 0x1E, then bytes = 500-269 = 231 = 0x00E7
        let mut bytes = vec![0x1E, 0x00, 0xE7];
        let value: Vec<u8> = (0..500u16).map(|i| (i & 0xFF) as u8).collect();
        bytes.extend_from_slice(&value);
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![CoapOption { number: 1, value }]);
    }

    #[test]
    fn uri_path_three_segments() {
        // RFC 7252 Section 5.10.1 example: Uri-Path "coap" + Uri-Path "example"
        //   Segment 1: number 11, delta=11, len=4. byte0 = (11<<4)|4 = 0xB4, value "coap"
        //   Segment 2: number 11, delta=0, len=7. byte0 = (0<<4)|7 = 0x07, value "example"
        let bytes = vec![0xB4, b'c', b'o', b'a', b'p', 0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e'];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![
            CoapOption { number: 11, value: b"coap".to_vec() },
            CoapOption { number: 11, value: b"example".to_vec() },
        ]);
    }

    #[test]
    fn encode_round_trip() {
        let original = vec![
            CoapOption { number: 1, value: vec![] },
            CoapOption { number: 11, value: b"a".to_vec() },
            CoapOption { number: 200, value: vec![0xAA, 0xBB] },
            CoapOption { number: 1000, value: vec![0x01] },
        ];
        let bytes = encode_options(&original);
        let parsed = parse_options(&bytes).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn payload_marker_is_error() {
        // 0xFF marks the start of payload in CoAP; should not be parsed as an option
        let bytes = vec![0x10, 0xFF];
        assert!(parse_options(&bytes).is_err());
    }

    #[test]
    fn truncated_value_is_error() {
        // Opt 1, length 5, but only 3 bytes follow
        let bytes = vec![0x15, 1, 2, 3];
        assert!(parse_options(&bytes).is_err());
    }

    #[test]
    fn repeated_option_zero_delta() {
        // Two Opt 11 entries (Uri-Path = "a", Uri-Path = "b"), second has delta=0
        let bytes = vec![0xB1, b'a', 0x01, b'b'];
        let opts = parse_options(&bytes).unwrap();
        assert_eq!(opts, vec![
            CoapOption { number: 11, value: vec![b'a'] },
            CoapOption { number: 11, value: vec![b'b'] },
        ]);
    }
}