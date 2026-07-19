// Minimal BIP-39 mnemonic codec.
//
// BIP-39 (Bitcoin Improvement Proposal 39) defines how to convert a
// high-entropy byte sequence into a human-friendly sequence of words from
// a fixed 2048-word list, plus how to recover the original bytes (and
// verify their integrity) from those words.
//
// At a glance:
//   * The wordlist has 2048 entries, so each word encodes 11 bits
//     (log2(2048) = 11).
//   * An `ENT`-bit entropy (where `ENT` is one of 128/160/192/224/256)
//     is concatenated with `CS = ENT / 32` checksum bits taken from the
//     leading bits of SHA-256(entropy).
//   * The combined `ENT + CS` bits are split into 11-bit groups and each
//     group indexes one wordlist entry.
//
// Allowed mnemonic lengths (in words) follow `3 * ENT / 32`:
//   12 words = 128-bit entropy  (+  4 checksum bits)
//   15 words = 160-bit entropy  (+  5 checksum bits)
//   18 words = 192-bit entropy  (+  6 checksum bits)
//   21 words = 224-bit entropy  (+  7 checksum bits)
//   24 words = 256-bit entropy  (+  8 checksum bits)
//
// This module supports all five lengths and exposes validate,
// entropy_to_mnemonic, and mnemonic_to_entropy per the task spec.

use sha2::{Digest, Sha256};

use crate::util::bip39_wordlist::WORDLIST;

const VALID_ENTROPY_BITS: &[usize] = &[128, 160, 192, 224, 256];

/// Normalize a mnemonic (lowercase + collapse internal whitespace).
fn normalize(mnemonic: &str) -> String {
    let mut out = String::with_capacity(mnemonic.len());
    let mut last_was_space = true;
    for ch in mnemonic.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.extend(ch.to_lowercase());
            last_was_space = false;
        }
    }
    let trimmed = out.trim();
    trimmed.to_string()
}

/// Append `n_bits` from `value` (low `n_bits`) to a bit buffer.
fn push_bits(buf: &mut Vec<u8>, value: u32, n_bits: u32) {
    for shift in (0..n_bits).rev() {
        buf.push(((value >> shift) & 1) as u8);
    }
}

/// Read `n_bits` from `buf` starting at `offset` and return them as a u32
/// (most-significant-bit-first within the field).
fn read_bits(buf: &[u8], offset: usize, n_bits: u32) -> u32 {
    let mut val: u32 = 0;
    for i in 0..n_bits {
        val = (val << 1) | (buf[offset + i as usize] as u32);
    }
    val
}

/// Validate a mnemonic: word count is allowed, every word is in the list,
/// and the appended checksum matches SHA-256(entropy)[..CS].
///
/// Returns `Ok(true)` on a fully valid mnemonic, `Ok(false)` if the words
/// decode but the checksum is wrong, and `Err` if the mnemonic is
/// structurally invalid (wrong length, unknown words, etc).
pub fn validate(mnemonic: &str) -> Result<bool, String> {
    let normalized = normalize(mnemonic);
    let words: Vec<&str> = normalized.split(' ').collect();
    let n = words.len();
    if n != 12 && n != 15 && n != 18 && n != 21 && n != 24 {
        return Err(format!("invalid mnemonic length: {} words", n));
    }
    // ENT = 11n * 32 / 33, CS = ENT / 32 = 11n / 33
    let entropy_bits = (n * 11 * 32) / 33;
    let cs_bits = entropy_bits / 32;

    // Build a flat bit buffer from the wordlist indices.
    let mut bits: Vec<u8> = Vec::with_capacity(n * 11);
    for w in &words {
        let idx =
            WORDLIST.iter().position(|x| x == w).ok_or_else(|| format!("unknown word: {}", w))?;
        push_bits(&mut bits, idx as u32, 11);
    }

    // The trailing `cs_bits` bits are the encoded checksum.
    let mut checksum_val: u32 = 0;
    for i in 0..cs_bits {
        checksum_val = (checksum_val << 1) | bits[entropy_bits + i as usize] as u32;
    }

    // The leading `entropy_bits` are the original entropy.
    let entropy_bytes = entropy_bits / 8;
    let mut entropy_vec: Vec<u8> = vec![0u8; entropy_bytes];
    for byte_idx in 0..entropy_bytes {
        let mut byte_val: u8 = 0;
        let start = byte_idx * 8;
        for bit_idx in 0..8 {
            byte_val = (byte_val << 1) | bits[start + bit_idx] as u8;
        }
        entropy_vec[byte_idx] = byte_val;
    }

    let mut hasher = Sha256::new();
    hasher.update(&entropy_vec);
    let hash = hasher.finalize();
    let expected_cs = (hash[0] as u16) >> (8 - cs_bits as u32);
    Ok(expected_cs as u32 == checksum_val)
}

/// Convert raw entropy bytes into a space-separated mnemonic.
///
/// `entropy.len()` must be one of 16, 20, 24, 28, or 32 (corresponding to
/// 128/160/192/224/256 bits). The checksum is computed automatically.
pub fn entropy_to_mnemonic(entropy: &[u8]) -> Result<String, String> {
    let ent_bits = entropy.len() * 8;
    if !VALID_ENTROPY_BITS.contains(&ent_bits) {
        return Err(format!("entropy must be 128/160/192/224/256 bits, got {} bits", ent_bits));
    }
    let cs_bits = ent_bits / 32;

    // Compute checksum over the entropy.
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let hash = hasher.finalize();
    // The first `cs_bits` bits of SHA-256(entropy) are the checksum.
    let checksum_val: u32 = (hash[0] as u32) >> (8 - cs_bits as u32);

    // Concatenate entropy bits + checksum bits into one buffer, then
    // split into 11-bit groups indexing the wordlist.
    let mut bits: Vec<u8> = Vec::with_capacity(ent_bits + cs_bits);
    for b in entropy {
        push_bits(&mut bits, *b as u32, 8);
    }
    push_bits(&mut bits, checksum_val, cs_bits as u32);

    let total_bits = bits.len();
    let n_words = total_bits / 11;
    let mut words: Vec<&str> = Vec::with_capacity(n_words);
    for i in 0..n_words {
        let idx = read_bits(&bits, i * 11, 11) as usize;
        words.push(WORDLIST[idx]);
    }
    Ok(words.join(" "))
}

/// Recover the original entropy bytes from a mnemonic.
///
/// On a structurally valid mnemonic whose checksum does not match, returns
/// an error (callers who need the looser check should call `validate`).
pub fn mnemonic_to_entropy(mnemonic: &str) -> Result<Vec<u8>, String> {
    let normalized = normalize(mnemonic);
    let words: Vec<&str> = normalized.split(' ').collect();
    let n = words.len();
    if n != 12 && n != 15 && n != 18 && n != 21 && n != 24 {
        return Err(format!("invalid mnemonic length: {} words", n));
    }
    let ent_bits = (n * 11 * 32) / 33;
    let cs_bits = ent_bits / 32;

    let mut bits: Vec<u8> = Vec::with_capacity(n * 11);
    for w in &words {
        let idx =
            WORDLIST.iter().position(|x| x == w).ok_or_else(|| format!("unknown word: {}", w))?;
        push_bits(&mut bits, idx as u32, 11);
    }

    let mut checksum_val: u32 = 0;
    for i in 0..cs_bits {
        checksum_val = (checksum_val << 1) | bits[ent_bits + i as usize] as u32;
    }

    let entropy_bytes = ent_bits / 8;
    let mut entropy: Vec<u8> = vec![0u8; entropy_bytes];
    for byte_idx in 0..entropy_bytes {
        let mut byte_val: u8 = 0;
        let start = byte_idx * 8;
        for bit_idx in 0..8 {
            byte_val = (byte_val << 1) | bits[start + bit_idx] as u8;
        }
        entropy[byte_idx] = byte_val;
    }

    let mut hasher = Sha256::new();
    hasher.update(&entropy);
    let hash = hasher.finalize();
    let expected_cs = (hash[0] as u16) >> (8 - cs_bits as u32);
    if expected_cs as u32 != checksum_val {
        return Err("checksum mismatch".to_string());
    }
    Ok(entropy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trezor_12word_vector_1() {
        // Trezor BIP-39 test vector #1:
        //   entropy  = 00000000000000000000000000000000
        //   mnemonic = abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon abandon abandon about
        let entropy = [0u8; 16];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon about"
        );
        let decoded = mnemonic_to_entropy(&mnemonic).expect("decode");
        assert_eq!(decoded, entropy);
    }

    #[test]
    fn trezor_12word_vector_2() {
        // Trezor BIP-39 test vector #2:
        //   entropy  = 7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f
        //   mnemonic = legal winner thank year wave sausage worth
        //              useful legal winner thank yellow
        let entropy = [0x7fu8; 16];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "legal winner thank year wave sausage worth useful legal winner thank yellow"
        );
        assert_eq!(mnemonic_to_entropy(&mnemonic).expect("decode"), entropy);
    }

    #[test]
    fn trezor_24word_vector() {
        // Trezor BIP-39 test vector #24 (24-word, all-zeros entropy):
        //   entropy  = 00000000000000000000000000000000 00000000000000000000000000000000
        //   mnemonic = abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon abandon abandon art
        let entropy = [0u8; 32];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon art"
        );
        let decoded = mnemonic_to_entropy(&mnemonic).expect("decode");
        assert_eq!(decoded, entropy);
    }

    #[test]
    fn trezor_15word_vector() {
        // Trezor BIP-39 test vector (15-word, 160-bit entropy):
        //   entropy  = 00000000000000000000000000000000 00000000
        //   mnemonic = abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon abandon abandon abandon
        //              abandon abandon abandon address
        let entropy = [0u8; 20];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon address"
        );
        assert_eq!(mnemonic_to_entropy(&mnemonic).expect("decode"), entropy);
    }

    #[test]
    fn validate_good_mnemonic_returns_true() {
        let m = "abandon abandon abandon abandon abandon abandon \
                 abandon abandon abandon abandon abandon about";
        assert_eq!(validate(m).expect("valid"), true);
    }

    #[test]
    fn validate_tampered_mnemonic_returns_false() {
        // Flip one word to a different valid list entry; checksum should fail.
        let m = "abandon abandon abandon abandon abandon abandon \
                 abandon abandon abandon abandon abandon abandon";
        assert_eq!(validate(m).expect("valid"), false);
    }

    #[test]
    fn validate_rejects_unknown_word() {
        let m = "abandon abandon abandon abandon abandon abandon \
                 abandon abandon abandon abandon abandon notrealword";
        let err = validate(m).expect_err("must reject");
        assert!(err.contains("unknown word"), "got: {}", err);
    }

    #[test]
    fn validate_rejects_wrong_word_count() {
        let m = "abandon abandon abandon abandon";
        let err = validate(m).expect_err("must reject");
        assert!(err.contains("invalid mnemonic length"), "got: {}", err);
    }

    #[test]
    fn entropy_to_mnemonic_rejects_wrong_size() {
        let err = entropy_to_mnemonic(&[0u8; 15]).expect_err("must reject");
        assert!(err.contains("128/160/192/224/256"), "got: {}", err);
    }

    #[test]
    fn uppercase_and_extra_whitespace_normalized() {
        let m = "ABANDON abandon abandon abandon abandon abandon \
                 abandon abandon abandon abandon abandon about  ";
        // Uppercase + trailing whitespace should still validate as the canonical mnemonic.
        assert_eq!(validate(m).expect("normalize"), true);
    }

    #[test]
    fn mnemonics_are_unique_for_known_vectors() {
        // Two different entropies must yield two different mnemonics.
        let m1 = entropy_to_mnemonic(&[0u8; 16]).expect("e1");
        let m2 = entropy_to_mnemonic(&[1u8; 16]).expect("e2");
        assert_ne!(m1, m2);
    }

    #[test]
    fn trezor_18word_vector() {
        // 18-word = 192-bit entropy. All-zeros canonical mnemonic.
        let entropy = [0u8; 24];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon agent"
        );
        assert_eq!(mnemonic_to_entropy(&mnemonic).expect("decode"), entropy);
    }

    #[test]
    fn trezor_21word_vector() {
        // 21-word = 224-bit entropy. All-zeros canonical mnemonic.
        // Last word encodes the 7-bit SHA-256 checksum.
        let entropy = [0u8; 28];
        let mnemonic = entropy_to_mnemonic(&entropy).expect("encode");
        assert_eq!(
            mnemonic,
            "abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon \
             abandon abandon admit"
        );
        assert_eq!(mnemonic_to_entropy(&mnemonic).expect("decode"), entropy);
    }
}
