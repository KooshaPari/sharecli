//! Minimal OAuth 1.0a request signature builder + verifier (RFC 5849).
//!
//! Computes the `oauth_signature` value for a request by:
//!   1. normalizing the URL
//!   2. collecting parameters (URL query + body params + oauth_* params)
//!   3. percent-encoding keys and values
//!   4. sorting lexicographically
//!   5. building the signature base string
//!   6. signing with HMAC-SHA1 (RFC 5849 default) or PLAINTEXT
//!
//! PLAINTEXT mode concatenates `consumer_secret&token_secret` directly without
//! any hashing — useful for testing and for legacy servers that require it.
//!
//! Reference: <https://datatracker.ietf.org/doc/html/rfc5849#section-3.4>

use std::collections::BTreeMap;

/// HTTP request abstract sufficient to sign per RFC 5849.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureMethod {
    HmacSha1,
    Plaintext,
}

/// Build the `oauth_signature` value for a request.
///
/// `req` carries the HTTP method, URL, and any oauth_* / request params. URL
/// query parameters are merged into the signing set per §3.4.1.3.1.
///
/// `consumer_secret` is required. `token_secret` is required for token-bound
/// requests; pass `None` for two-legged OAuth (the secret field becomes empty).
pub fn build_signature(
    req: &Request,
    consumer_secret: &str,
    token_secret: Option<&str>,
    method: SignatureMethod,
) -> String {
    let token = token_secret.unwrap_or("");
    match method {
        SignatureMethod::Plaintext => {
            // RFC 5849 §3.4.4: PLAINTEXT simply concatenates the secrets.
            format!("{}&{}", percent_encode(consumer_secret), percent_encode(token))
        }
        SignatureMethod::HmacSha1 => {
            let base = build_signature_base_string(req);
            let key = build_signing_key(consumer_secret, token);
            let mac = hmac_sha1(key.as_bytes(), base.as_bytes());
            base64_encode(&mac)
        }
    }
}

/// Verify a previously-computed signature by re-signing the same request and
/// comparing in constant time.
pub fn verify_signature(
    req: &Request,
    consumer_secret: &str,
    token_secret: Option<&str>,
    method: SignatureMethod,
    expected_signature: &str,
) -> bool {
    let actual = build_signature(req, consumer_secret, token_secret, method);
    constant_time_eq(actual.as_bytes(), expected_signature.as_bytes())
}

// ---- HMAC-SHA1 (RFC 2104) ----
//
// Hand-written to avoid pulling in a crypto crate. SHA-1 is in scope for
// OAuth 1.0a (the spec explicitly requires it as the default signature
// method, RFC 5849 §3.4). It is not used for any other security purpose.

const SHA1_BLOCK: usize = 64;
const SHA1_DIGEST: usize = 20;

fn sha1_compress(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut w = [0u32; 80];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    for i in 0..80 {
        let (f, k) = match i {
            0..=19 => ((b & c) | ((!b) & d), 0x5a827999u32),
            20..=39 => (b ^ c ^ d, 0x6ed9eba1u32),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdcu32),
            _ => (b ^ c ^ d, 0xca62c1d6u32),
        };
        let temp =
            a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

fn sha1(data: &[u8]) -> [u8; SHA1_DIGEST] {
    let mut state: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut i = 0;
    while i + SHA1_BLOCK <= data.len() {
        let mut block = [0u8; SHA1_BLOCK];
        block.copy_from_slice(&data[i..i + SHA1_BLOCK]);
        sha1_compress(&mut state, &block);
        i += SHA1_BLOCK;
    }
    let rem = &data[i..];
    let mut block = [0u8; SHA1_BLOCK];
    block[..rem.len()].copy_from_slice(rem);
    block[rem.len()] = 0x80;
    if rem.len() >= SHA1_BLOCK - 8 {
        sha1_compress(&mut state, &block);
        block = [0u8; SHA1_BLOCK];
    }
    block[SHA1_BLOCK - 8..].copy_from_slice(&bit_len.to_be_bytes());
    sha1_compress(&mut state, &block);
    let mut out = [0u8; SHA1_DIGEST];
    for (idx, &s) in state.iter().enumerate() {
        out[idx * 4..idx * 4 + 4].copy_from_slice(&s.to_be_bytes());
    }
    out
}

pub(crate) fn hmac_sha1(key: &[u8], msg: &[u8]) -> [u8; SHA1_DIGEST] {
    let mut key_block = [0u8; SHA1_BLOCK];
    if key.len() > SHA1_BLOCK {
        let digest = sha1(key);
        key_block[..digest.len()].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; SHA1_BLOCK];
    let mut opad = [0x5cu8; SHA1_BLOCK];
    for i in 0..SHA1_BLOCK {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }
    let mut inner = Vec::with_capacity(SHA1_BLOCK + msg.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(msg);
    let inner_hash = sha1(&inner);
    let mut outer = Vec::with_capacity(SHA1_BLOCK + inner_hash.len());
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    sha1(&outer)
}

// ---- Base64 (standard, RFC 4648 §4) ----

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
        out.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(BASE64_CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        out.push(BASE64_CHARS[(b2 & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let b0 = input[i];
        out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
        out.push(BASE64_CHARS[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = input[i];
        let b1 = input[i + 1];
        out.push(BASE64_CHARS[(b0 >> 2) as usize] as char);
        out.push(BASE64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(BASE64_CHARS[((b1 & 0x0f) << 2) as usize] as char);
        out.push('=');
    }
    out
}

// ---- Percent-encoding (RFC 3986) ----
//
// Unreserved set: A-Z / a-z / 0-9 / "-" / "." / "_" / "~"
// All other bytes become %XX with uppercase hex.

pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        if matches!(
            b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
        ) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(hex_digit(b >> 4));
            out.push(hex_digit(b & 0x0f));
        }
    }
    out
}

fn hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + (n - 10)) as char,
        _ => '0',
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ---- URL normalization (RFC 5849 §3.4.1.2) ----

pub fn normalize_url(url: &str) -> String {
    let scheme_end = url.find("://").map(|i| i + 3);
    let (scheme, rest) = match scheme_end {
        Some(idx) => (url[..idx].to_ascii_lowercase(), &url[idx..]),
        None => (String::new(), url),
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let (host, port) = match authority.rfind(':') {
        Some(idx) => (&authority[..idx], &authority[idx + 1..]),
        None => (authority, ""),
    };
    let host_lc = host.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http://" => "80",
        "https://" => "443",
        _ => "",
    };
    let port_part =
        if !port.is_empty() && port != default_port { format!(":{}", port) } else { String::new() };
    format!("{}{}{}{}", scheme, host_lc, port_part, path)
}

fn split_url_query(url: &str) -> (String, String) {
    match url.find('?') {
        Some(i) => (url[..i].to_string(), url[i + 1..].to_string()),
        None => (url.to_string(), String::new()),
    }
}

fn parse_query(q: &str) -> Vec<(String, String)> {
    if q.is_empty() {
        return Vec::new();
    }
    q.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = match p.find('=') {
                Some(i) => (&p[..i], &p[i + 1..]),
                None => (p, ""),
            };
            (percent_decode(k), percent_decode(v))
        })
        .collect()
}

// ---- Signature base string (RFC 5849 §3.4.1) ----

fn build_signature_base_string(req: &Request) -> String {
    let (url_base, query) = split_url_query(&req.url);
    let normalized_url = normalize_url(&url_base);

    // Merge query params with explicit params.
    let mut combined: Vec<(String, String)> = Vec::with_capacity(req.params.len() + 8);
    for (k, v) in &req.params {
        combined.push((k.clone(), v.clone()));
    }
    for (k, v) in parse_query(&query) {
        if combined.iter().any(|(ek, ev)| ek == &k && ev == &v) {
            continue; // dedupe exact (k, v) pair
        }
        if let Some(existing) = combined.iter_mut().find(|(ek, _)| ek == &k) {
            existing.1.push(',');
            existing.1.push_str(&v);
        } else {
            combined.push((k, v));
        }
    }

    let mut encoded: Vec<(String, String)> =
        combined.iter().map(|(k, v)| (percent_encode(k), percent_encode(v))).collect();
    encoded.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let normalized_params =
        encoded.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&");
    format!(
        "{}&{}&{}",
        req.method.to_ascii_uppercase(),
        percent_encode(&normalized_url),
        percent_encode(&normalized_params),
    )
}

fn build_signing_key(consumer_secret: &str, token_secret: &str) -> String {
    format!("{}&{}", percent_encode(consumer_secret), percent_encode(token_secret),)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_request() -> Request {
        let mut params = BTreeMap::new();
        params.insert("status".into(), "Hello Ladies + Gentlemen, a signed OAuth request!".into());
        params.insert("include_entities".into(), "true".into());
        params.insert("oauth_consumer_key".into(), "xvz1evFS4wEEPTGEFPHBog".into());
        params.insert("oauth_nonce".into(), "kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg".into());
        params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
        params.insert("oauth_timestamp".into(), "1318622958".into());
        params.insert(
            "oauth_token".into(),
            "370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb".into(),
        );
        params.insert("oauth_version".into(), "1.0".into());
        Request {
            method: "POST".into(),
            url: "https://api.twitter.com/1/statuses/update.json?include_entities=true".into(),
            params,
        }
    }

    #[test]
    fn rfc_5849_3_4_1_1_base_string() {
        // Build the base string explicitly and assert it matches the RFC's
        // expected value. We don't need to compute the signature here — the
        // underlying base string construction is the same regardless of secret.
        let req = base_request();
        let s = build_signature_base_string(&req);
        assert_eq!(
            s,
            "POST&https%3A%2F%2Fapi.twitter.com%2F1%2Fstatuses%2Fupdate.json&include_entities%3Dtrue%26oauth_consumer_key%3Dxvz1evFS4wEEPTGEFPHBog%26oauth_nonce%3DkYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1318622958%26oauth_token%3D370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb%26oauth_version%3D1.0%26status%3DHello%2520Ladies%2520%252B%2520Gentlemen%252C%2520a%2520signed%2520OAuth%2520request%2521"
        );
    }

    #[test]
    fn rfc_5849_3_4_1_1_signature() {
        // Reference signature for the RFC 5849 example with the matching
        // consumer / token secrets. The percent-encoded form is
        //   tnnArxj06cWHq44gCs1OSKk%2FjLY%3D
        // which decodes to the base64 string below.
        let req = base_request();
        let sig = build_signature(
            &req,
            "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw",
            Some("LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE"),
            SignatureMethod::HmacSha1,
        );
        assert_eq!(sig, "tnnArxj06cWHq44gCs1OSKk/jLY=");
    }

    #[test]
    fn hmac_sha1_known_vector_rfc_2202() {
        // RFC 2202 test case 1: key = 0x0b*20, msg = "Hi There"
        let key = vec![0x0bu8; 20];
        let mac = hmac_sha1(&key, b"Hi There");
        let hex: String = mac.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "b617318655057264e28bc0b6fb378c8ef146be00");
    }

    #[test]
    fn plaintext_mode_no_hashing() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let sig = build_signature(&req, "consumer", Some("token"), SignatureMethod::Plaintext);
        assert_eq!(sig, "consumer&token");
        // Two-legged: token_secret None -> empty second segment.
        let sig2 = build_signature(&req, "consumer", None, SignatureMethod::Plaintext);
        assert_eq!(sig2, "consumer&");
    }

    #[test]
    fn percent_encoding_edge_cases() {
        assert_eq!(percent_encode(" "), "%20");
        assert_eq!(percent_encode("&"), "%26");
        assert_eq!(percent_encode("="), "%3D");
        assert_eq!(percent_encode("+"), "%2B");
        assert_eq!(percent_encode("/"), "%2F");
        assert_eq!(percent_encode("~"), "~");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn verify_accepts_matching_signature() {
        let req = base_request();
        let sig = build_signature(
            &req,
            "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw",
            Some("LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE"),
            SignatureMethod::HmacSha1,
        );
        assert!(verify_signature(
            &req,
            "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw",
            Some("LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE"),
            SignatureMethod::HmacSha1,
            &sig,
        ));
    }

    #[test]
    fn verify_rejects_tampered_signature() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let sig = build_signature(&req, "cs", Some("ts"), SignatureMethod::HmacSha1);
        let mut bad: Vec<u8> = sig.into_bytes();
        bad[0] = if bad[0] == b'A' { b'B' } else { b'A' };
        let bad_str = String::from_utf8(bad).unwrap();
        assert!(!verify_signature(&req, "cs", Some("ts"), SignatureMethod::HmacSha1, &bad_str,));
    }

    #[test]
    fn reject_invalid_base64_signature() {
        // A signature with non-base64 characters cannot match a real signature.
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let sig = build_signature(&req, "cs", Some("ts"), SignatureMethod::HmacSha1);
        // Tamper beyond just flipping a bit — corrupt a char that is normally
        // a valid base64 char but in this position it produces a different
        // string. Then length-mismatch style: a shorter or longer string.
        assert!(!verify_signature(
            &req,
            "cs",
            Some("ts"),
            SignatureMethod::HmacSha1,
            "not!base64$"
        ));
        // Wrong-length string is also rejected.
        assert!(!verify_signature(&req, "cs", Some("ts"), SignatureMethod::HmacSha1, "abc"));
        // Re-confirm original still verifies.
        assert!(verify_signature(&req, "cs", Some("ts"), SignatureMethod::HmacSha1, &sig));
    }

    #[test]
    fn parameter_order_independence() {
        // Two param maps with the same key set must produce identical signatures
        // regardless of insertion order — sorting happens after collection.
        let mut p1 = BTreeMap::new();
        p1.insert("z".into(), "1".into());
        p1.insert("a".into(), "2".into());
        p1.insert("m".into(), "3".into());
        let mut p2 = BTreeMap::new();
        p2.insert("a".into(), "2".into());
        p2.insert("m".into(), "3".into());
        p2.insert("z".into(), "1".into());
        let req1 = Request { method: "GET".into(), url: "https://example.com/".into(), params: p1 };
        let req2 = Request { method: "GET".into(), url: "https://example.com/".into(), params: p2 };
        let s1 = build_signature(&req1, "cs", Some("ts"), SignatureMethod::HmacSha1);
        let s2 = build_signature(&req2, "cs", Some("ts"), SignatureMethod::HmacSha1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn url_normalization_lowercases_host() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req1 = Request {
            method: "GET".into(),
            url: "https://Example.com/p".into(),
            params: params.clone(),
        };
        let req2 = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let s1 = build_signature(&req1, "cs", Some("ts"), SignatureMethod::HmacSha1);
        let s2 = build_signature(&req2, "cs", Some("ts"), SignatureMethod::HmacSha1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn url_normalization_strips_default_port() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req1 = Request {
            method: "GET".into(),
            url: "https://example.com:443/p".into(),
            params: params.clone(),
        };
        let req2 = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let s1 = build_signature(&req1, "cs", Some("ts"), SignatureMethod::HmacSha1);
        let s2 = build_signature(&req2, "cs", Some("ts"), SignatureMethod::HmacSha1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn method_is_uppercased() {
        let mut params = BTreeMap::new();
        params.insert("a".into(), "1".into());
        let req1 = Request {
            method: "get".into(),
            url: "https://example.com/p".into(),
            params: params.clone(),
        };
        let req2 = Request { method: "GET".into(), url: "https://example.com/p".into(), params };
        let s1 = build_signature(&req1, "cs", Some("ts"), SignatureMethod::HmacSha1);
        let s2 = build_signature(&req2, "cs", Some("ts"), SignatureMethod::HmacSha1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn signing_key_encodes_secrets() {
        // Unreserved pass-through.
        assert_eq!(build_signing_key("cs", "ts"), "cs&ts");
        // Special chars get percent-encoded.
        assert_eq!(build_signing_key("cs&", "ts="), "cs%26&ts%3D");
    }

    #[test]
    fn base64_encode_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
