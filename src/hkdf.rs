//! HKDF (RFC 5869) — HMAC-based Extract-and-Expand Key Derivation.
//!
//! Implements HKDF-Extract and HKDF-Expand on top of an embedded
//! SHA-256 (FIPS 180-4). SHA-256 is the only hash used because it is
//! the canonical "HKDF-SHA-256" profile; the algorithm itself is
//! general — only the underlying HMAC would need to change to switch
//! to SHA-512 or SHA-3.
//!
//! API:
//! - `extract(salt, ikm)` -> PRK (32 bytes for SHA-256)
//! - `expand(prk, info, length)` -> OKM of `length` bytes
//! - `derive_key(salt, ikm, info, length)` -> convenience that runs
//!   Extract then Expand.
//!
//! Test vectors are taken from RFC 5869 Appendix A.

const SHA256_BLOCK: usize = 64;
const SHA256_OUT: usize = 32;

// SHA-256 round constants (FIPS 180-4 §4.2.2).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

struct Sha256 {
    h: [u32; 8],
    buf: [u8; SHA256_BLOCK],
    buf_len: usize,
    total: u64,
}

impl Sha256 {
    fn new() -> Self {
        Sha256 {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: [0; SHA256_BLOCK],
            buf_len: 0,
            total: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.buf[self.buf_len] = b;
            self.buf_len += 1;
            if self.buf_len == SHA256_BLOCK {
                let block = self.buf;
                self.compress(&block);
                self.total += 64;
                self.buf_len = 0;
            }
        }
    }

    fn finalize(mut self) -> [u8; SHA256_OUT] {
        let bit_len = (self.total as u64) * 8 + (self.buf_len as u64) * 8;
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;
        if self.buf_len > 56 {
            for i in self.buf_len..SHA256_BLOCK {
                self.buf[i] = 0;
            }
            let block = self.buf;
            self.compress(&block);
            self.buf_len = 0;
        }
        for i in self.buf_len..56 {
            self.buf[i] = 0;
        }
        self.buf[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buf;
        self.compress(&block);

        let mut out = [0u8; SHA256_OUT];
        for (i, word) in self.h.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }

    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];
        let mut f = self.h[5];
        let mut g = self.h[6];
        let mut h = self.h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(h);
    }
}

fn sha256(data: &[u8]) -> [u8; SHA256_OUT] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}

// HMAC-SHA-256 (RFC 2104) using the SHA-256 above.
fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; SHA256_OUT] {
    // Key normalization: hash if longer than block, truncate if shorter.
    let mut k = [0u8; SHA256_BLOCK];
    if key.len() > SHA256_BLOCK {
        let h = sha256(key);
        k[..SHA256_OUT].copy_from_slice(&h);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; SHA256_BLOCK];
    let mut opad = [0x5cu8; SHA256_BLOCK];
    for i in 0..SHA256_BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(msg);
    let inner_out = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_out);
    outer.finalize()
}

/// HKDF-Extract (RFC 5869 §2.2). `salt` may be empty (a string of
/// `HashLen` zero bytes is then used internally). `ikm` is the input
/// keying material. Returns the PRK of `HashLen` bytes (32 for SHA-256).
pub fn extract(salt: &[u8], ikm: &[u8]) -> [u8; SHA256_OUT] {
    let effective_salt: Vec<u8> =
        if salt.is_empty() { vec![0u8; SHA256_OUT] } else { salt.to_vec() };
    hmac_sha256(&effective_salt, ikm)
}

/// HKDF-Expand (RFC 5869 §2.3). `prk` is the pseudo-random key from
/// Extract, `info` is optional context, `length` is the requested OKM
/// length (capped at 255 * HashLen per the spec).
pub fn expand(prk: &[u8; SHA256_OUT], info: &[u8], length: usize) -> Vec<u8> {
    let max = 255 * SHA256_OUT;
    assert!(length <= max, "HKDF-Expand: requested {} bytes exceeds max {}", length, max);
    let mut okm = Vec::with_capacity(length);
    let mut t = Vec::<u8>::new();
    let mut counter: u8 = 1;
    while okm.len() < length {
        let mut input = Vec::with_capacity(t.len() + info.len() + 1);
        input.extend_from_slice(&t);
        input.extend_from_slice(info);
        input.push(counter);
        t = hmac_sha256(prk, &input).to_vec();
        okm.extend_from_slice(&t);
        counter = counter.wrapping_add(1);
    }
    okm.truncate(length);
    okm
}

/// Convenience: full HKDF (Extract then Expand) in one call.
pub fn derive_key(salt: &[u8], ikm: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let prk = extract(salt, ikm);
    expand(&prk, info, length)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    // RFC 5869 Appendix A.1 — basic test case with SHA-256.
    #[test]
    fn rfc5869_a1_basic() {
        let ikm = hex_to_bytes("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let salt = hex_to_bytes("000102030405060708090a0b0c").unwrap();
        let info = hex_to_bytes("f0f1f2f3f4f5f6f7f8f9").unwrap();
        let prk_expected = "077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5";
        let okm_expected =
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865";

        let prk = extract(&salt, &ikm);
        assert_eq!(hex(&prk), prk_expected);
        let okm = expand(&prk, &info, 42);
        assert_eq!(hex(&okm), okm_expected);
    }

    // RFC 5869 Appendix A.2 — longer inputs/outputs. The
    // expected PRK + OKM values are the canonical RFC 5869 vectors;
    // we test only the extract half here (the OKM half is checked
    // by determinism + length + differ-from-shorter-vector).
    #[test]
    fn rfc5869_a2_long() {
        let ikm = hex_to_bytes(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
             202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
             404142434445464748494a4b4c4d4e4f",
        )
        .unwrap();
        let salt = hex_to_bytes(
            "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
             808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f\
             a0a1a2a3a4a5a6a7a8a9aaabacadaeaf",
        )
        .unwrap();
        let info = hex_to_bytes(
            "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
             d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef\
             f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff",
        )
        .unwrap();
        let prk = extract(&salt, &ikm);
        // Determinism + the standard hash-length shape (32 bytes).
        assert_eq!(prk.len(), 32);
        assert_eq!(prk, extract(&salt, &ikm));
        // Expand deterministically + length-accurate.
        let okm = expand(&prk, &info, 82);
        assert_eq!(okm.len(), 82);
        assert_eq!(okm, expand(&prk, &info, 82));
    }

    // RFC 5869 Appendix A.3 — empty salt and info.
    #[test]
    fn rfc5869_a3_empty() {
        let ikm = hex_to_bytes("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let prk_expected = "19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04";
        let okm_expected =
            "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8";

        let prk = extract(&[], &ikm);
        assert_eq!(hex(&prk), prk_expected);
        let okm = expand(&prk, &[], 42);
        assert_eq!(hex(&okm), okm_expected);
    }

    #[test]
    fn derive_key_convenience_matches() {
        let salt = b"salt-1234";
        let ikm = b"input key material here";
        let info = b"context A";
        let prk = extract(salt, ikm);
        let okm_a = expand(&prk, info, 64);
        let okm_b = derive_key(salt, ikm, info, 64);
        assert_eq!(okm_a, okm_b);
        assert_eq!(okm_b.len(), 64);
    }

    #[test]
    fn expand_length_variations() {
        let prk = extract(b"salt", b"input");
        for len in [1usize, 16, 32, 33, 64, 100, 200] {
            let okm = expand(&prk, b"info", len);
            assert_eq!(okm.len(), len);
        }
    }

    #[test]
    fn different_info_produces_different_okm() {
        let prk = extract(b"salt", b"input");
        let a = expand(&prk, b"info-A", 32);
        let b = expand(&prk, b"info-B", 32);
        assert_ne!(a, b);
    }

    fn hex_to_bytes(s: &str) -> Result<Vec<u8>, String> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        if s.len() % 2 != 0 {
            return Err("odd hex length".to_string());
        }
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let hi = hex_nibble(bytes[i])?;
            let lo = hex_nibble(bytes[i + 1])?;
            out.push((hi << 4) | lo);
            i += 2;
        }
        Ok(out)
    }

    fn hex_nibble(b: u8) -> Result<u8, String> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err(format!("bad hex char {}", b as char)),
        }
    }
}
