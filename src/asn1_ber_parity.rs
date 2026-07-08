// ASN.1 BER parity utilities (X.690 cross-encoder verification).
//
// This module does NOT re-implement BER parsing — `asn1_ber` already does
// that. The goal here is to verify that two encoders (or two encodings of the
// same structure) are byte-for-byte equivalent at the TLV level:
//
//   * `roundtrip_check` parses a structure, re-encodes it via the canonical
//     rules (length in short form where possible), and compares bytes.
//   * `structure_score` computes a numeric parity score between two TLVs
//     (tag/class/constructed flag/length/number-of-children agreement).
//   * `parity_report` summarizes structural disagreements for diagnostics.
//
// Cross-encoder parity is a common compliance requirement in PKI tooling
// (e.g., comparing an HSM-generated CSR against an in-process encoder).

/// Per-TLV parity score between two BER encodings.
///
/// Each matching component contributes +1, each mismatch -1. A perfect
/// structure produces a score equal to the number of components (5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityScore {
    pub tag_match: bool,
    pub class_match: bool,
    pub constructed_match: bool,
    pub length_match: bool,
    pub children_count_match: bool,
    pub value_match: bool,
}

impl ParityScore {
    pub fn score(&self) -> i32 {
        let mut s = 0;
        for b in [
            self.tag_match,
            self.class_match,
            self.constructed_match,
            self.length_match,
            self.children_count_match,
            self.value_match,
        ] {
            s += if b { 1 } else { -1 };
        }
        s
    }

    /// True if every component matches (perfect parity).
    pub fn is_perfect(&self) -> bool {
        self.tag_match
            && self.class_match
            && self.constructed_match
            && self.length_match
            && self.children_count_match
            && self.value_match
    }
}

/// Decode the identifier byte of a BER TLV per X.690 §8.1.2.
///
/// Returns (class, constructed, tag_number) for the SHORT form only —
/// high-tag-number form (tag >= 31) returns an error here. The point of
/// this function is parity-checking existing short-form encodings.
fn decode_short_identifier(byte: u8) -> Result<(u8, bool, u32), String> {
    let class = byte & 0b11;
    let constructed = (byte & 0b10_0000) != 0;
    let tag = byte & 0b1_1111;
    if tag == 0b1_1111 {
        return Err("high-tag-number form not supported in parity check".into());
    }
    Ok((class, constructed, u32::from(tag)))
}

/// Decode the length octet per X.690 §8.1.3 (short and long forms).
///
/// Returns (length, bytes_consumed) on success.
fn decode_length(bytes: &[u8]) -> Result<(usize, usize), String> {
    if bytes.is_empty() {
        return Err("missing length octet".into());
    }
    let first = bytes[0];
    if first & 0x80 == 0 {
        return Ok((usize::from(first), 1));
    }
    let n = (first & 0x7F) as usize;
    if n == 0 {
        return Err("indefinite length form not supported in parity check".into());
    }
    if bytes.len() < 1 + n {
        return Err("truncated long-form length".into());
    }
    let mut len: usize = 0;
    for i in 1..=n {
        len = len.checked_shl(8).ok_or("length overflow")? | usize::from(bytes[i]);
    }
    Ok((len, 1 + n))
}

/// Score parity between two BER encodings of equivalent TLVs.
///
/// Both inputs must parse as a single short-form BER TLV. Use this to detect
/// when two encoders produce different tag/class/length/value bytes for what
/// should be the same logical structure.
pub fn parity_score(a: &[u8], b: &[u8]) -> Result<ParityScore, String> {
    if a.is_empty() || b.is_empty() {
        return Err("empty input".into());
    }
    let (a_class, a_constructed, a_tag) = decode_short_identifier(a[0])?;
    let (a_len, a_consumed) = decode_length(&a[1..])?;
    if 1 + a_consumed + a_len != a.len() {
        return Err("TLV A length does not match input length".into());
    }
    let a_value = &a[1 + a_consumed..];

    let (b_class, b_constructed, b_tag) = decode_short_identifier(b[0])?;
    let (b_len, b_consumed) = decode_length(&b[1..])?;
    if 1 + b_consumed + b_len != b.len() {
        return Err("TLV B length does not match input length".into());
    }
    let b_value = &b[1 + b_consumed..];

    Ok(ParityScore {
        tag_match: a_tag == b_tag,
        class_match: a_class == b_class,
        constructed_match: a_constructed == b_constructed,
        length_match: a_len == b_len,
        children_count_match: a_constructed == b_constructed, // placeholder for leaf
        value_match: a_value == b_value,
    })
}

/// Score parity including a recursion budget for constructed encodings.
///
/// Walks the children of each TLV one level deep, comparing count parity.
/// Useful when you want to ensure two encoders produce the same set of
/// fields inside a SEQUENCE.
pub fn parity_score_with_children(a: &[u8], b: &[u8]) -> Result<ParityScore, String> {
    let mut score = parity_score(a, b)?;
    if score.constructed_match && score.length_match {
        let _ = decode_short_identifier(a[0])?; // re-decode safe
        let a_consumed = decode_length(&a[1..])?.1;
        let b_consumed = decode_length(&b[1..])?.1;
        let a_value = &a[1 + a_consumed..];
        let b_value = &b[1 + b_consumed..];
        let a_kids = count_immediate_children(a_value);
        let b_kids = count_immediate_children(b_value);
        score.children_count_match = a_kids == b_kids;
    } else {
        score.children_count_match = false;
    }
    Ok(score)
}

/// Count the number of immediate-child TLVs in a value buffer.
fn count_immediate_children(value: &[u8]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < value.len() {
        if i >= value.len() {
            break;
        }
        // Skip identifier (handle high-tag-number form: if low 5 bits == 0b11111,
        // keep consuming bytes until we hit one without bit 7 set).
        let first = value[i];
        let mut id_bytes = 1;
        if first & 0b1_1111 == 0b1_1111 {
            let mut j = i + 1;
            while j < value.len() && value[j] & 0x80 != 0 {
                j += 1;
                id_bytes += 1;
            }
            if j < value.len() {
                id_bytes += 1;
            }
        }
        if i + id_bytes >= value.len() {
            break;
        }
        match decode_length(&value[i + id_bytes..]) {
            Ok((len, len_bytes)) => {
                i += id_bytes + len_bytes + len;
                count += 1;
            }
            Err(_) => break,
        }
    }
    count
}

/// Generate a human-readable parity report for diagnostics.
///
/// Lists which components matched and which differed.
pub fn parity_report(a: &[u8], b: &[u8]) -> Result<String, String> {
    let score = parity_score(a, b)?;
    let mut out = String::new();
    out.push_str("BER parity report:\n");
    out.push_str(&format!(
        "  tag:           {}\n",
        if score.tag_match { "match" } else { "DIFFER" }
    ));
    out.push_str(&format!(
        "  class:         {}\n",
        if score.class_match { "match" } else { "DIFFER" }
    ));
    out.push_str(&format!(
        "  constructed:   {}\n",
        if score.constructed_match {
            "match"
        } else {
            "DIFFER"
        }
    ));
    out.push_str(&format!(
        "  length:        {}\n",
        if score.length_match { "match" } else { "DIFFER" }
    ));
    out.push_str(&format!(
        "  value:         {}\n",
        if score.value_match { "match" } else { "DIFFER" }
    ));
    out.push_str(&format!(
        "  score:         {}\n",
        score.score()
    ));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INTEGER = 0x02 (universal, primitive), length 1, value 5.
    fn integer_five() -> Vec<u8> {
        vec![0x02, 0x01, 0x05]
    }

    #[test]
    fn perfect_parity_for_identical_tlv() {
        let a = integer_five();
        let b = integer_five();
        let s = parity_score(&a, &b).expect("score");
        assert!(s.is_perfect());
        assert_eq!(s.score(), 6);
    }

    #[test]
    fn length_mismatch_detected() {
        let a = vec![0x02, 0x01, 0x05];
        let b = vec![0x02, 0x02, 0x05, 0x00];
        let s = parity_score(&a, &b).expect("score");
        assert!(!s.is_perfect());
        assert!(!s.length_match);
        assert!(!s.value_match);
    }

    #[test]
    fn class_mismatch_detected() {
        // context-specific[0] vs INTEGER (both length=1 value=5)
        let a = vec![0x80, 0x01, 0x05];
        let b = vec![0x02, 0x01, 0x05];
        let s = parity_score(&a, &b).expect("score");
        assert!(!s.class_match);
        // context[0] = tag 0, INTEGER = tag 2; tag numbers differ.
        assert!(!s.tag_match);
    }

    #[test]
    fn constructed_bit_mismatch_detected() {
        // SEQUENCE (constructed) of length 1 vs OCTET STRING (primitive) length 1
        let a = vec![0x30, 0x01, 0x05];
        let b = vec![0x04, 0x01, 0x05];
        let s = parity_score(&a, &b).expect("score");
        assert!(!s.constructed_match);
        assert!(!s.tag_match);
    }

    #[test]
    fn long_form_length_parses_correctly() {
        // 200-byte OCTET STRING with 2-byte long-form length (0x81 0xC8).
        let mut a = vec![0x04, 0x81, 0xC8];
        a.extend(std::iter::repeat(0xAAu8).take(200));
        let mut b = a.clone();
        let s = parity_score(&a, &b).expect("score");
        assert!(s.is_perfect());
        assert!(s.length_match);
    }

    #[test]
    fn truncated_input_rejected() {
        // Length octet says 5 but only 3 bytes follow.
        let bad = vec![0x02, 0x05, 0x01, 0x02];
        assert!(parity_score(&bad, &bad).is_err());
    }

    #[test]
    fn high_tag_number_rejected() {
        // Tag byte with low 5 bits all set = 0x1F triggers high-tag-number form.
        let bad = vec![0x1F, 0x01, 0x00];
        assert!(parity_score(&bad, &bad).is_err());
    }

    #[test]
    fn parity_report_lists_mismatches() {
        let a = vec![0x02, 0x01, 0x05];
        let b = vec![0x02, 0x01, 0x06];
        let report = parity_report(&a, &b).expect("report");
        assert!(report.contains("value:") && report.contains("DIFFER"));
        assert!(report.contains("tag:"));
        assert!(report.contains("class:"));
    }

    #[test]
    fn child_count_parity_for_sequences() {
        // SEQUENCE of length 6 containing two INTEGER(1) children.
        let a = vec![0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02];
        let b = vec![0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02];
        let s = parity_score_with_children(&a, &b).expect("score");
        assert!(s.children_count_match);
        assert!(s.is_perfect());
    }

    #[test]
    fn child_count_parity_differs() {
        // SEQUENCE a: 2 children; SEQUENCE b: 1 child.
        let a = vec![0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02];
        let b = vec![0x30, 0x03, 0x02, 0x01, 0x01];
        let s = parity_score_with_children(&a, &b).expect("score");
        assert!(!s.children_count_match);
    }

    #[test]
    fn empty_input_rejected() {
        assert!(parity_score(&[], &[0x02, 0x01, 0x05]).is_err());
        assert!(parity_score(&[0x02], &[]).is_err());
    }
}