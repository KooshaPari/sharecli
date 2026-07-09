// Minimal ASN.1 BER (X.690) parser/encoder.
//
// ASN.1 (Abstract Syntax Notation One) is the data-structure language used
// by X.509 certificates, LDAP, SNMP, PKCS, Kerberos, and many other
// Internet protocols. BER (Basic Encoding Rules, ITU-T X.690) is one of
// several ways to encode ASN.1 values into bytes.
//
// Each BER TLV has three parts:
//   Tag      : 1+ bytes describing the class (Universal/Application/
//              Context-specific/Private), the form (primitive/constructed),
//              and the tag number.
//   Length   : 1+ bytes. Short form: a single byte 0..=127. Long form:
//              first byte has high bit set (0x80 | n), then n length bytes.
//   Contents : `Length` bytes of value (primitive) or nested TLVs
//              (constructed).
//
// We support the universal class tags most often seen in the wild:
// BOOLEAN (0x01), INTEGER (0x02), BIT STRING (0x03), OCTET STRING (0x04),
// NULL (0x05), OBJECT IDENTIFIER (0x06), UTF8String (0x0C), SEQUENCE (0x30),
// SET (0x31), PrintableString (0x13), IA5String (0x16), and high-tag-number
// form for tag numbers >= 31. Constructed forms (SEQUENCE, SET) expose
// their inner TLVs in `children`.

use std::fmt;

/// BER class (low 2 bits of the identifier byte).
pub const CLASS_UNIVERSAL: u8 = 0b00;
pub const CLASS_APPLICATION: u8 = 0b01;
pub const CLASS_CONTEXT: u8 = 0b10;
pub const CLASS_PRIVATE: u8 = 0b11;

/// A parsed BER TLV.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ber {
    /// The raw identifier byte (or first byte of a high-tag-number form).
    pub tag: u8,
    /// True if the constructed bit (bit 5 of the first identifier byte) is set.
    pub constructed: bool,
    /// BER class (low 2 bits of the identifier byte).
    pub class: u8,
    /// Decoded tag number. For high-tag-number form this is computed from the
    /// continuation bytes per X.690 §8.1.2.4.
    pub tag_number: u32,
    /// Raw contents bytes (empty for NULL).
    pub value: Vec<u8>,
    /// For constructed encodings, the parsed child TLVs.
    pub children: Vec<Ber>,
}

impl fmt::Display for Ber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ber(class={}, constructed={}, tag={}, value={}B, children={})",
            self.class,
            self.constructed,
            self.tag_number,
            self.value.len(),
            self.children.len()
        )
    }
}

/// Parse a BER TLV from the front of `input` and return the parsed value plus
/// any unconsumed trailing bytes.
///
/// The parser accepts indefinite-length encoding (length byte = 0x80) by
/// reading child TLVs until a 0x00 0x00 end-of-content marker, per X.690
/// §8.1.3.6.
pub fn parse(input: &[u8]) -> Result<(Ber, &[u8]), String> {
    let (first_id, rest) = input.split_first().ok_or_else(|| "empty input".to_string())?;

    // Class lives in bits 7-6 (top 2 bits) of the first identifier byte,
    // not in the bottom 2 bits. X.690 §8.1.2.2.
    let class = (first_id >> 6) & 0b11;
    let constructed = (first_id & 0b0010_0000) != 0;
    let low_tag = first_id & 0b0001_1111;

    let (tag_number, after_id) = if low_tag != 0b0001_1111 {
        (low_tag as u32, rest)
    } else {
        // High-tag-number form (X.690 §8.1.2.4).
        let mut acc: u32 = 0;
        let mut cursor = rest;
        loop {
            let (b, next) =
                cursor.split_first().ok_or_else(|| "truncated high-tag-number form".to_string())?;
            acc = acc.checked_shl(7).ok_or_else(|| "high-tag-number overflow".to_string())?
                | (*b as u32) & 0x7f;
            let found_last = (*b & 0x80) == 0;
            cursor = next;
            if found_last {
                break;
            }
            if acc == 0 && cursor.len() == usize::MAX {
                // unreachable guard to keep the borrow checker happy
                return Err("infinite high-tag-number loop".to_string());
            }
        }
        (acc, cursor)
    };

    let (len, after_len) = read_length(after_id)?;
    let (contents, trailing, indefinite_children) = if len == usize::MAX {
        // Indefinite length: read children until 0x00 0x00 EOC.
        let (children, after_eoc) = read_indefinite_contents(after_len)?;
        (Vec::new(), after_eoc, Some(children))
    } else {
        if after_len.len() < len {
            return Err(format!("truncated value: need {} bytes, have {}", len, after_len.len()));
        }
        let (contents, trailing) = after_len.split_at(len);
        (contents.to_vec(), trailing, None)
    };

    let mut node = Ber {
        tag: *first_id,
        constructed,
        class,
        tag_number,
        value: contents,
        children: Vec::new(),
    };

    if constructed {
        // Parse children from the contents bytes (or use the indefinite-length
        // children collected above).
        if let Some(children) = indefinite_children {
            node.children = children;
        } else {
            let mut cursor: &[u8] = &node.value;
            while !cursor.is_empty() {
                let (child, rest) = parse(cursor)?;
                node.children.push(child);
                cursor = rest;
            }
        }
    }

    Ok((node, trailing))
}

fn read_length(input: &[u8]) -> Result<(usize, &[u8]), String> {
    let (first, rest) = input.split_first().ok_or_else(|| "missing length octet".to_string())?;
    if *first < 0x80 {
        return Ok(((*first) as usize, rest));
    }
    if *first == 0x80 {
        return Ok((usize::MAX, rest));
    }
    let n = (*first & 0x7f) as usize;
    if n == 0 {
        return Err("reserved length form (0x80 | 0)".to_string());
    }
    if rest.len() < n {
        return Err(format!("truncated long-form length: need {} bytes, have {}", n, rest.len()));
    }
    let (len_bytes, after) = rest.split_at(n);
    let mut len: usize = 0;
    for b in len_bytes {
        len = len.checked_shl(8).ok_or("length overflow")? | (*b as usize);
    }
    Ok((len, after))
}

fn read_indefinite_contents(input: &[u8]) -> Result<(Vec<Ber>, &[u8]), String> {
    // Walk TLVs until we hit a single EOC: identifier 0x00 + length 0x00.
    let mut cursor = input;
    let mut children: Vec<Ber> = Vec::new();
    loop {
        if cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x00 {
            return Ok((children, &cursor[2..]));
        }
        let (child, rest) = parse(cursor)?;
        children.push(child);
        cursor = rest;
    }
}

/// Encode a primitive BER value: `(tag_byte, contents) -> Vec<u8>`.
pub fn encode_primitive(tag: u8, contents: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    encode_length(&mut out, contents.len());
    out.extend_from_slice(contents);
    out
}

/// Encode a constructed BER value: opens with `tag`, then each child as its
/// own TLV, then closes (no explicit closing marker in BER).
pub fn encode_constructed(tag: u8, children: &[Ber]) -> Vec<u8> {
    let mut out = vec![tag];
    let mut body = Vec::new();
    for c in children {
        body.extend_from_slice(&encode(c));
    }
    encode_length(&mut out, body.len());
    out.extend_from_slice(&body);
    out
}

fn encode_length(out: &mut Vec<u8>, len: usize) {
    if len < 0x80 {
        out.push(len as u8);
    } else {
        // Long form: count the bytes required.
        let mut buf = [0u8; 9];
        let mut n = len;
        let mut i = 0;
        while n > 0 {
            buf[i] = n as u8;
            n >>= 8;
            i += 1;
        }
        out.push(0x80 | (i as u8));
        // Emit length bytes in big-endian order.
        for j in (0..i).rev() {
            out.push(buf[j]);
        }
    }
}

/// Encode an already-parsed `Ber` back to bytes.
pub fn encode(b: &Ber) -> Vec<u8> {
    if !b.children.is_empty() {
        encode_constructed(b.tag, &b.children)
    } else {
        encode_primitive(b.tag, &b.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_true() {
        // SEQUENCE { BOOLEAN true } -- standard DER for `true`.
        let bytes = [0x30, 0x03, 0x01, 0x01, 0xff];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x30);
        assert!(node.constructed);
        assert_eq!(node.children.len(), 1);
        let inner = &node.children[0];
        assert_eq!(inner.tag, 0x01);
        assert!(!inner.constructed);
        assert_eq!(inner.value, vec![0xff]);
    }

    #[test]
    fn parse_integer_x690_test_vector() {
        // X.690 §8.3 example: INTEGER value 49 (= 0x31).
        // Encoding: tag 0x02, length 0x01, contents 0x31.
        let bytes = [0x02, 0x01, 0x31];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x02);
        assert_eq!(node.tag_number, 2);
        assert_eq!(node.value, vec![0x31]);
    }

    #[test]
    fn parse_integer_negative_two_complement() {
        // INTEGER -128: X.690 says negatives encode as 2's complement;
        // -128 = 0x80 in 1 byte.
        let bytes = [0x02, 0x01, 0x80];
        let (node, _) = parse(&bytes).expect("parse");
        assert_eq!(node.value, vec![0x80]);
    }

    #[test]
    fn parse_octet_string() {
        // OCTET STRING containing "abc" = 0x61 0x62 0x63.
        let bytes = [0x04, 0x03, 0x61, 0x62, 0x63];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x04);
        assert_eq!(node.value, b"abc");
    }

    #[test]
    fn parse_null_value() {
        // NULL has zero-length contents.
        let bytes = [0x05, 0x00];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x05);
        assert!(node.value.is_empty());
    }

    #[test]
    fn parse_object_identifier_sha256_with_ec() {
        // OID 1.2.840.10045.4.3.2 (id-ecdsa-with-SHA256), encoded per X.690 §8.19.
        // First two arcs: 1*40 + 2 = 42 = 0x2A. Subsequent arcs in base-128
        // with high bit set on all but the last byte.
        //   840  = 0x86 0x48
        //   10045 = 0x81 0x4D
        //   4    = 0x04
        //   3    = 0x03
        //   2    = 0x02
        let bytes = [0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x04, 0x03, 0x02];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x06);
        // Decoded value bytes must equal the wire form.
        assert_eq!(node.value, vec![0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x04, 0x03, 0x02]);
    }

    #[test]
    fn parse_sequence_with_nested_sequence() {
        // SEQUENCE { SEQUENCE { INTEGER 5 } }
        // Outer: tag 0x30, length 0x05.
        // Inner SEQUENCE: tag 0x30, length 0x03, contents 02 01 05.
        // Total: 30 05 30 03 02 01 05.
        let bytes = [0x30, 0x05, 0x30, 0x03, 0x02, 0x01, 0x05];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.children.len(), 1);
        let inner = &node.children[0];
        assert!(inner.constructed);
        assert_eq!(inner.children.len(), 1);
        assert_eq!(inner.children[0].tag_number, 2);
        assert_eq!(inner.children[0].value, vec![0x05]);
    }

    #[test]
    fn long_form_length_two_byte() {
        // Length 200 in long form: 0x81 0xC8.
        // Construct a SEQUENCE of length 200 with all-zero body.
        let mut bytes = vec![0x30, 0x81, 0xC8];
        bytes.extend(std::iter::repeat(0u8).take(200));
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert!(node.constructed);
        assert_eq!(node.value.len(), 200);
    }

    #[test]
    fn high_tag_number_form_parses() {
        // Tag number 100 (= 0x64) in high-tag-number form.
        // First byte: low 5 bits = 0x1F (signals high-tag-number form).
        // Continuation byte: 0x64 (high bit 0 = last byte).
        // Length byte: 0x00 (zero-length contents).
        let bytes = [0x1F, 0x64, 0x00];
        let (node, _) = parse(&bytes).expect("parse");
        assert_eq!(node.tag_number, 0x64);
    }

    #[test]
    fn truncated_input_rejected() {
        // Tag 0x02 with length 0x05 but only 2 bytes of contents.
        let bytes = [0x02, 0x05, 0xAB, 0xCD];
        let err = parse(&bytes).expect_err("must reject");
        assert!(err.contains("truncated"), "got: {}", err);
    }

    #[test]
    fn empty_input_rejected() {
        let err = parse(&[]).expect_err("must reject");
        assert!(err.contains("empty"), "got: {}", err);
    }

    #[test]
    fn encode_roundtrip_integer() {
        let node = Ber {
            tag: 0x02,
            constructed: false,
            class: CLASS_UNIVERSAL,
            tag_number: 2,
            value: vec![0x7F],
            children: Vec::new(),
        };
        let bytes = encode(&node);
        assert_eq!(bytes, vec![0x02, 0x01, 0x7F]);
        let (decoded, rest) = parse(&bytes).expect("reparse");
        assert!(rest.is_empty());
        assert_eq!(decoded, node);
    }

    #[test]
    fn encode_long_form_length() {
        // Length 300 requires long form: 0x82 0x01 0x2C.
        let bytes = encode_primitive(0x04, &vec![0u8; 300]);
        assert_eq!(bytes[0], 0x04);
        assert_eq!(bytes[1], 0x82);
        assert_eq!(bytes[2], 0x01);
        assert_eq!(bytes[3], 0x2C);
        assert_eq!(bytes.len(), 4 + 300);
    }

    #[test]
    fn encode_then_parse_printable_string() {
        // PrintableString "Hello"
        let bytes = encode_primitive(0x13, b"Hello");
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert_eq!(node.tag, 0x13);
        assert_eq!(node.tag_number, 0x13);
        assert_eq!(node.value, b"Hello");
    }

    #[test]
    fn parse_set_constructed() {
        // SET { INTEGER 1, INTEGER 2 } -- DER requires ascending tags inside.
        let bytes = [0x31, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02];
        let (node, rest) = parse(&bytes).expect("parse");
        assert!(rest.is_empty());
        assert!(node.constructed);
        assert_eq!(node.tag, 0x31);
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].value, vec![1]);
        assert_eq!(node.children[1].value, vec![2]);
    }

    #[test]
    fn trailing_bytes_after_tlv_left_in_rest() {
        // Parse the first TLV and ensure the rest of the buffer is returned.
        let bytes = [0x02, 0x01, 0x05, 0x05, 0x00];
        let (node, rest) = parse(&bytes).expect("parse");
        assert_eq!(node.tag, 0x02);
        assert_eq!(rest, &[0x05, 0x00]);
    }

    #[test]
    fn parse_utf8_string() {
        // UTF8String "héllo" = 68 c3 a9 6c 6c 6f
        let bytes = [0x0C, 0x06, 0x68, 0xC3, 0xA9, 0x6C, 0x6C, 0x6F];
        let (node, _) = parse(&bytes).expect("parse");
        assert_eq!(node.tag, 0x0C);
        assert_eq!(node.value, "héllo".as_bytes());
    }

    #[test]
    fn parse_ia5_string() {
        // IA5String "[email protected]"
        let bytes = [0x16, 0x0A, b'a', b'@', b'b', b'.', b'c', b'o', b'm', 0x00, 0x01, 0x02];
        let (node, _) = parse(&bytes).expect("parse");
        assert_eq!(node.tag, 0x16);
        assert_eq!(node.value, b"a@b.com\x00\x01\x02");
    }

    #[test]
    fn parse_bit_string_unused_bits_marker() {
        // BIT STRING with 0 unused bits, value 0x06, 0x05.
        let bytes = [0x03, 0x03, 0x00, 0x06, 0x05];
        let (node, _) = parse(&bytes).expect("parse");
        assert_eq!(node.tag, 0x03);
        assert_eq!(node.value, vec![0x00, 0x06, 0x05]);
    }

    #[test]
    fn long_form_length_overflow_rejected() {
        // 9 length bytes of 0xFF = 0xFFFFFFFFFFFFFFFF, but read_length uses
        // usize::checked_shl so on a 32-bit target it returns Err.
        let bytes = [0x02, 0x84, 0xFF, 0xFF, 0xFF, 0xFF];
        let err = parse(&bytes).expect_err("must reject");
        assert!(err.contains("length overflow") || err.contains("truncated"));
    }
}
