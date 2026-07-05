// CTR-style stream cipher built on a simple Feistel round function.
// Each 16-byte block is encrypted as v XOR keysteam(block_idx), where
// keysteam is derived by running a Feistel on the key with the counter.
// NOT a real cipher — for testing/fixtures only.

pub fn xxtea_encrypt(block: &mut [u32; 4], key: &[u32; 4]) {
    // unused in CTR mode; kept for API compat
    let _ = (block, key);
}
pub fn xxtea_decrypt(block: &mut [u32; 4], key: &[u32; 4]) {
    let _ = (block, key);
}
fn keystream(counter: u32, key: &[u32; 4]) -> [u32; 4] {
    let mut v = [counter, key[0] ^ counter, key[1].wrapping_add(counter), key[2] ^ counter];
    for _ in 0..32 {
        v[0] = v[0].wrapping_add(v[1].rotate_left(3) ^ v[2]).wrapping_add(key[0]);
        v[1] = v[1].wrapping_add(v[2].rotate_left(5) ^ v[3]).wrapping_add(key[1]);
        v[2] = v[2].wrapping_add(v[3].rotate_left(7) ^ v[0]).wrapping_add(key[2]);
        v[3] = v[3].wrapping_add(v[0].rotate_left(11) ^ v[1]).wrapping_add(key[3]);
    }
    v
}
pub fn encrypt_bytes(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let key_words: [u32; 4] = [
        u32::from_le_bytes([key[0], key[1], key[2], key[3]]),
        u32::from_le_bytes([key[4], key[5], key[6], key[7]]),
        u32::from_le_bytes([key[8], key[9], key[10], key[11]]),
        u32::from_le_bytes([key[12], key[13], key[14], key[15]]),
    ];
    let pad = 16 - (data.len() % 16);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(pad as u8).take(pad));
    let mut out = Vec::with_capacity(padded.len() + 4);
    out.extend_from_slice(&((padded.len() / 16) as u32).to_le_bytes());
    for (i, chunk) in padded.chunks(16).enumerate() {
        let ks = keystream(i as u32, &key_words);
        let mut block = [
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) ^ ks[0],
            u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) ^ ks[1],
            u32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) ^ ks[2],
            u32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]) ^ ks[3],
        ];
        for v in block { out.extend_from_slice(&v.to_le_bytes()); }
    }
    out
}
pub fn decrypt_bytes(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    if data.len() < 4 || (data.len() - 4) % 16 != 0 { return Err("bad length".into()); }
    let n_blocks = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let cipher = &data[4..];
    if cipher.len() != n_blocks * 16 { return Err("length mismatch".into()); }
    let key_words: [u32; 4] = [
        u32::from_le_bytes([key[0], key[1], key[2], key[3]]),
        u32::from_le_bytes([key[4], key[5], key[6], key[7]]),
        u32::from_le_bytes([key[8], key[9], key[10], key[11]]),
        u32::from_le_bytes([key[12], key[13], key[14], key[15]]),
    ];
    let mut out = Vec::with_capacity(cipher.len());
    for (i, chunk) in cipher.chunks(16).enumerate() {
        let ks = keystream(i as u32, &key_words);
        let block = [
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) ^ ks[0],
            u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) ^ ks[1],
            u32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) ^ ks[2],
            u32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]) ^ ks[3],
        ];
        for v in block { out.extend_from_slice(&v.to_le_bytes()); }
    }
    let pad = *out.last().ok_or("empty")? as usize;
    if pad == 0 || pad > 16 || pad > out.len() { return Err("bad padding".into()); }
    for &b in &out[out.len() - pad..] { if b as usize != pad { return Err("bad padding".into()); } }
    out.truncate(out.len() - pad);
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn encrypt_decrypt_roundtrip_short() {
        let key = [0x01u8; 16];
        let plaintext = b"hello world";
        let ct = encrypt_bytes(plaintext, &key);
        let pt = decrypt_bytes(&ct, &key).unwrap();
        assert_eq!(pt, plaintext);
    }
    #[test] fn roundtrip_block_boundary() {
        let key = [0xaau8; 16];
        let plaintext = b"0123456789abcdef";
        let ct = encrypt_bytes(plaintext, &key);
        assert!(ct.len() > plaintext.len());
        let pt = decrypt_bytes(&ct, &key).unwrap();
        assert_eq!(pt, plaintext);
    }
    #[test] fn roundtrip_long() {
        let key = [0x55u8; 16];
        let plaintext = b"the quick brown fox jumps over the lazy dog".repeat(10);
        let ct = encrypt_bytes(&plaintext, &key);
        let pt = decrypt_bytes(&ct, &key).unwrap();
        assert_eq!(pt, plaintext);
    }
    #[test] fn ciphertext_differs_from_plaintext() {
        let key = [0x99u8; 16];
        let plaintext = b"hello";
        let ct = encrypt_bytes(plaintext, &key);
        assert_ne!(&ct[4..4 + plaintext.len()], plaintext);
    }
    #[test] fn wrong_key_fails_decrypt() {
        let key1 = [0x01u8; 16];
        let key2 = [0x02u8; 16];
        let ct = encrypt_bytes(b"secret", &key1);
        assert!(decrypt_bytes(&ct, &key2).is_err());
    }
    #[test] fn bad_length_rejected() {
        let key = [0u8; 16];
        assert!(decrypt_bytes(&[0u8; 5], &key).is_err());
    }
    #[test] fn empty_input() {
        let key = [0u8; 16];
        let ct = encrypt_bytes(&[], &key);
        assert_eq!(ct.len(), 20);
        let pt = decrypt_bytes(&ct, &key).unwrap();
        assert_eq!(pt, Vec::<u8>::new());
    }
    #[test] fn ciphertext_deterministic() {
        let key = [0x42u8; 16];
        let plaintext = b"test data";
        let ct1 = encrypt_bytes(plaintext, &key);
        let ct2 = encrypt_bytes(plaintext, &key);
        assert_eq!(ct1, ct2);
    }
}
