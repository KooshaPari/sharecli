//! Base64url (RFC 4648 §5) encoder + decoder.
//!
//! URL-safe variant of base64 with `-` and `_` instead of `+` and `/`,
//! and padding stripped by convention. Common for JWT (RFC 7519),
//! URL-safe binary blobs, and opaque token strings.
//!
//! Padding is handled leniently: `encode` always emits unpadded output,
//! `decode` accepts both padded and unpadded input.

const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// RFC 4648 §5 base64url encode. Output is unpadded.
pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 4 + 2) / 3);
    let mut i = 0;
    while i + 3 <= data.len() {
        let b = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
        out.push(T[((b >> 6) & 0x3f) as usize] as char);
        out.push(T[(b & 0x3f) as usize] as char);
        i += 3;
    }
    let remaining = data.len() - i;
    if remaining == 1 {
        let b = (data[i] as u32) << 16;
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
    } else if remaining == 2 {
        let b = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(T[((b >> 18) & 0x3f) as usize] as char);
        out.push(T[((b >> 12) & 0x3f) as usize] as char);
        out.push(T[((b >> 6) & 0x3f) as usize] as char);
    }
    out
}

/// RFC 4648 §5 base64url decode. Accepts both padded and unpadded input.
/// Returns `Err` on any character outside the URL-safe alphabet (after
/// `=` is stripped).
pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    let stripped: String = input.chars().filter(|&c| c != '=').collect();
    if stripped.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = stripped.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in bytes {
        let idx = T.iter().position(|&c| c == b).ok_or_else(|| format!("bad char 0x{b:02x}"))?;
        buf = (buf << 6) | idx as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4648_test_vectors() {
        // RFC 4648 §10 base64url test vectors
        assert_eq!(encode(b""), "");
        assert_eq!(encode(b"f"), "Zg");
        assert_eq!(encode(b"fo"), "Zm8");
        assert_eq!(encode(b"foo"), "Zm9v");
        assert_eq!(encode(b"foob"), "Zm9vYg");
        assert_eq!(encode(b"fooba"), "Zm9vYmE");
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn rfc4648_decode_padded() {
        assert_eq!(decode("Zg==").unwrap(), b"f");
        assert_eq!(decode("Zm8=").unwrap(), b"fo");
        assert_eq!(decode("Zm9v").unwrap(), b"foo");
    }

    #[test]
    fn rfc4648_decode_unpadded() {
        assert_eq!(decode("Zg").unwrap(), b"f");
        assert_eq!(decode("Zm8").unwrap(), b"fo");
        assert_eq!(decode("Zm9v").unwrap(), b"foo");
        assert_eq!(decode("Zm9vYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn url_safe_alphabet_no_plus_slash() {
        // Standard base64 would emit '+' and '/' for these inputs;
        // base64url uses '-' and '_' instead. Sanity check that no
        // '+' or '/' appears in the output across many byte values.
        for byte in 0u8..=255 {
            let encoded = encode(&[byte]);
            assert!(!encoded.contains('+'), "byte {} produced {}", byte, encoded);
            assert!(!encoded.contains('/'), "byte {} produced {}", byte, encoded);
        }
    }

    #[test]
    fn round_trip_random() {
        for len in 0..40 {
            let data: Vec<u8> = (0..len).map(|i| (i * 37) as u8).collect();
            let encoded = encode(&data);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, data, "round-trip failed at len={}", len);
        }
    }

    #[test]
    fn decode_bad_char_errors() {
        assert!(decode("@@@@").is_err());
    }
}