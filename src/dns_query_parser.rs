// Minimal DNS message parser (RFC 1035 §4.1.1).
//
// Wire format overview (all multi-byte integers are big-endian):
//
//   Header (12 bytes):
//       uint16 ID
//       uint16 FLAGS    (QR | OPCODE | AA | TC | RD | RA | Z | RCODE)
//       uint16 QDCOUNT  questions
//       uint16 ANCOUNT  answers
//       uint16 NSCOUNT  authority records
//       uint16 ARCOUNT  additional records
//
//   Question section (QDCOUNT entries):
//       <name>          variable-length, label-encoded
//       uint16 QTYPE
//       uint16 QCLASS
//
//   Resource records (ANCOUNT + NSCOUNT + ARCOUNT entries):
//       <name>          variable-length, label-encoded
//       uint16 TYPE
//       uint16 CLASS
//       uint32 TTL
//       uint16 RDLENGTH
//       byte[RDLENGTH] RDATA (interpretation depends on TYPE)
//       — this module treats RDATA as opaque bytes; type-specific
//       decoding is the caller's responsibility.
//
// Domain name encoding (RFC 1035 §4.1.4):
//
//   A label is `<len><len bytes>`, where the high two bits of `len`
//   are 00 for a normal label, 11 for a compression pointer whose
//   low 14 bits are an offset from the start of the DNS message.
//   A label of length 0 terminates the name.
//
// This module:
//
//   * parses the header into `Header` (with the counts and a
//     `flags: u16` field that the caller can decode with the
//     helpers below),
//   * fully resolves any compression pointers while reading (no
//     pointer loops, max 64 levels of indirection),
//   * treats RDATA as `Vec<u8>` — it does not interpret A, AAAA,
//     CNAME, MX, NS, TXT, etc.,
//   * returns clear errors with byte offsets when the input is
//     truncated or malformed.

/// DNS message header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// 16-bit message ID. Echoed in responses so the client can
    /// match the answer to the right outstanding query.
    pub id: u16,
    /// Raw 16-bit flags word. Use [`flags_qr`], [`flags_opcode`],
    /// [`flags_rcode`], [`flags_is_authoritative`],
    /// [`flags_is_truncated`], [`flags_recursion_desired`],
    /// [`flags_recursion_available`] to decode individual bits.
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

/// A DNS question section entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Domain name (already resolved; compression has been expanded).
    pub name: String,
    /// QTYPE (1=A, 2=NS, 5=CNAME, 15=MX, 28=AAAA, 255=ANY, ...).
    pub qtype: u16,
    /// QCLASS (1=IN, 255=ANY, ...).
    pub qclass: u16,
}

/// A DNS resource record (answer, authority, or additional).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    /// Opaque RDATA bytes (type-specific decoding is the caller's job).
    pub rdata: Vec<u8>,
}

/// A fully parsed DNS message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authority: Vec<ResourceRecord>,
    pub additional: Vec<ResourceRecord>,
}

// ---------- Flag decoders (RFC 1035 §4.1.1) ----------

/// QR bit: 0 = query, 1 = response.
#[inline]
pub const fn flags_qr(flags: u16) -> bool {
    (flags & 0x8000) != 0
}

/// OPCODE: 0 = QUERY, 1 = IQUERY (obsolete), 2 = STATUS, 4 = NOTIFY,
/// 5 = UPDATE, ...
#[inline]
pub const fn flags_opcode(flags: u16) -> u8 {
    ((flags >> 11) & 0x0F) as u8
}

/// AA bit: server is an authority for the zone.
#[inline]
pub const fn flags_is_authoritative(flags: u16) -> bool {
    (flags & 0x0400) != 0
}

/// TC bit: message was truncated.
#[inline]
pub const fn flags_is_truncated(flags: u16) -> bool {
    (flags & 0x0200) != 0
}

/// RD bit: recursion desired.
#[inline]
pub const fn flags_recursion_desired(flags: u16) -> bool {
    (flags & 0x0100) != 0
}

/// RA bit: recursion available.
#[inline]
pub const fn flags_recursion_available(flags: u16) -> bool {
    (flags & 0x0080) != 0
}

/// RCODE: 0 = NOERROR, 1 = FORMERR, 2 = SERVFAIL, 3 = NXDOMAIN, ...
#[inline]
pub const fn flags_rcode(flags: u16) -> u8 {
    (flags & 0x000F) as u8
}

// ---------- Core parser ----------

const MAX_POINTER_DEPTH: usize = 64;
const MAX_NAME_LENGTH: usize = 255;
const MAX_LABEL_LENGTH: usize = 63;

/// Parse a DNS message from `input`. `input` is the full wire bytes,
/// including the 12-byte header.
///
/// Errors are formatted with byte offsets (when known) so callers
/// can correlate with a wire trace.
pub fn parse(input: &[u8]) -> Result<Packet, String> {
    if input.len() < 12 {
        return Err(format!(
            "DNS message too short: {} bytes (need at least 12 for header)",
            input.len()
        ));
    }

    let header = Header {
        id: u16::from_be_bytes([input[0], input[1]]),
        flags: u16::from_be_bytes([input[2], input[3]]),
        qdcount: u16::from_be_bytes([input[4], input[5]]),
        ancount: u16::from_be_bytes([input[6], input[7]]),
        nscount: u16::from_be_bytes([input[8], input[9]]),
        arcount: u16::from_be_bytes([input[10], input[11]]),
    };

    let mut cursor = 12usize;

    let mut questions = Vec::with_capacity(header.qdcount as usize);
    for _ in 0..header.qdcount {
        let (name, end) = read_name(input, cursor)?;
        cursor = end;
        if cursor + 4 > input.len() {
            return Err(format!(
                "truncated question section at byte {} (need QTYPE+QCLASS)",
                cursor
            ));
        }
        let qtype = u16::from_be_bytes([input[cursor], input[cursor + 1]]);
        let qclass = u16::from_be_bytes([input[cursor + 2], input[cursor + 3]]);
        cursor += 4;
        questions.push(Question { name, qtype, qclass });
    }

    let answers = read_rr_section(input, &mut cursor, header.ancount as usize, "answer")?;
    let authority = read_rr_section(input, &mut cursor, header.nscount as usize, "authority")?;
    let additional = read_rr_section(input, &mut cursor, header.arcount as usize, "additional")?;

    Ok(Packet { header, questions, answers, authority, additional })
}

/// Parse a label-encoded domain name starting at `start`.
///
/// Returns the decoded name (with no trailing dot, lowercase ASCII)
/// and the byte offset AFTER the name (which may include the QTYPE
/// + QCLASS for a Question, or the TYPE/CLASS/TTL/RDLENGTH fields
/// for a Resource Record).
///
/// Compression pointers are followed transparently — the returned
/// `end` offset is the position in the ORIGINAL buffer, not in any
/// pointed-to location.
fn read_name(input: &[u8], start: usize) -> Result<(String, usize), String> {
    let mut out = String::new();
    let mut cursor = start;
    let mut jumped = false;
    let mut final_end = start; // position after the name in the *original* buffer
    let mut depth = 0usize;

    loop {
        if cursor >= input.len() {
            return Err(format!("name extends past end of message at byte {}", cursor));
        }
        let len = input[cursor];
        match len & 0xC0 {
            0x00 => {
                // Plain label
                let l = (len & 0x3F) as usize;
                if l == 0 {
                    // End of name
                    cursor += 1;
                    if !jumped {
                        final_end = cursor;
                    }
                    break;
                }
                if l > MAX_LABEL_LENGTH {
                    return Err(format!("label length {} exceeds 63 at byte {}", l, cursor));
                }
                cursor += 1;
                if cursor + l > input.len() {
                    return Err(format!(
                        "label body truncated at byte {} (need {} bytes)",
                        cursor, l
                    ));
                }
                if !out.is_empty() {
                    out.push('.');
                }
                // RFC 1035 says DNS names are case-insensitive; emit
                // lowercase for deterministic comparisons.
                let label = std::str::from_utf8(&input[cursor..cursor + l])
                    .map_err(|e| format!("non-UTF8 label at byte {}: {}", cursor, e))?;
                out.push_str(&label.to_ascii_lowercase());
                cursor += l;
            }
            0xC0 => {
                // Compression pointer (RFC 1035 §4.1.4)
                if cursor + 1 >= input.len() {
                    return Err(format!("truncated compression pointer at byte {}", cursor));
                }
                let offset = u16::from_be_bytes([len & 0x3F, input[cursor + 1]]) as usize;
                if offset >= cursor {
                    return Err(format!(
                        "compression pointer at byte {} points forward to {}",
                        cursor, offset
                    ));
                }
                if !jumped {
                    final_end = cursor + 2;
                    jumped = true;
                }
                depth += 1;
                if depth > MAX_POINTER_DEPTH {
                    return Err(format!(
                        "compression pointer loop after {} jumps (last at byte {})",
                        depth, cursor
                    ));
                }
                cursor = offset;
            }
            _ => {
                // 0x80 (reserved) or 0x40 (extended label, RFC 2673 /
                // RFC 6891) — not implemented.
                return Err(format!("unsupported label type 0x{:02x} at byte {}", len, cursor));
            }
        }

        if out.len() > MAX_NAME_LENGTH {
            return Err(format!("name exceeds 255 bytes (truncated at byte {})", cursor));
        }
    }

    if out.is_empty() {
        return Err(format!("empty name at byte {}", start));
    }

    Ok((out, final_end))
}

fn read_rr_section(
    input: &[u8],
    cursor: &mut usize,
    count: usize,
    label: &str,
) -> Result<Vec<ResourceRecord>, String> {
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let (name, end) = read_name(input, *cursor)?;
        *cursor = end;
        // TYPE(2) + CLASS(2) + TTL(4) + RDLENGTH(2) = 10 bytes
        if *cursor + 10 > input.len() {
            return Err(format!("truncated {} record header at byte {}", label, cursor));
        }
        let rtype = u16::from_be_bytes([input[*cursor], input[*cursor + 1]]);
        let rclass = u16::from_be_bytes([input[*cursor + 2], input[*cursor + 3]]);
        let ttl = u32::from_be_bytes([
            input[*cursor + 4],
            input[*cursor + 5],
            input[*cursor + 6],
            input[*cursor + 7],
        ]);
        let rdlen = u16::from_be_bytes([input[*cursor + 8], input[*cursor + 9]]) as usize;
        *cursor += 10;
        if *cursor + rdlen > input.len() {
            return Err(format!(
                "{} record RDATA truncated at byte {} (need {} bytes)",
                label, cursor, rdlen
            ));
        }
        let rdata = input[*cursor..*cursor + rdlen].to_vec();
        *cursor += rdlen;
        out.push(ResourceRecord { name, rtype, rclass, ttl, rdata });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- fixtures (pre-computed, see /tmp/dns_check.py) ----

    /// Canonical example.com A query:
    ///   ID=0x1234, FLAGS=0x0100 (RD), QD=1
    ///   Question: example.com, A (1), IN (1)
    /// Hex: 123401000001000000000000076578616d706c6503636f6d0000010001
    fn example_com_query() -> Vec<u8> {
        vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65,
            0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00,
            0x01,
        ]
    }

    /// Canonical example.com A response with one answer and a
    /// compression pointer back to the question's name:
    ///   ID=0xABCD, FLAGS=0x8180 (QR=1, RD=1, RA=1), QD=1, AN=1
    ///   Question: example.com A IN
    ///   Answer: example.com (compressed) A IN TTL=3600 RDLENGTH=4 93.184.216.34
    fn example_com_response() -> Vec<u8> {
        vec![
            0xab, 0xcd, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65,
            0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00,
            0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x0e, 0x10, 0x00, 0x04, 0x5d,
            0xb8, 0xd8, 0x22,
        ]
    }

    // ---- parse header ----

    #[test]
    fn parses_canonical_query_header() {
        let pkt = parse(&example_com_query()).unwrap();
        assert_eq!(pkt.header.id, 0x1234);
        assert_eq!(pkt.header.flags, 0x0100);
        assert_eq!(pkt.header.qdcount, 1);
        assert_eq!(pkt.header.ancount, 0);
        assert_eq!(pkt.header.nscount, 0);
        assert_eq!(pkt.header.arcount, 0);

        assert!(!flags_qr(pkt.header.flags));
        assert_eq!(flags_opcode(pkt.header.flags), 0);
        assert!(flags_recursion_desired(pkt.header.flags));
        assert!(!flags_recursion_available(pkt.header.flags));
        assert_eq!(flags_rcode(pkt.header.flags), 0);
    }

    #[test]
    fn parses_canonical_response_header_and_answer() {
        let pkt = parse(&example_com_response()).unwrap();
        assert_eq!(pkt.header.id, 0xABCD);
        assert_eq!(pkt.header.flags, 0x8180);
        assert!(flags_qr(pkt.header.flags));
        // 0x8180 = QR=1 OPCODE=0 AA=0 TC=0 RD=1 RA=1 RCODE=0
        // (the AA bit is not set in this synthetic example)
        assert!(!flags_is_authoritative(pkt.header.flags));
        assert!(flags_recursion_desired(pkt.header.flags));
        assert!(flags_recursion_available(pkt.header.flags));
        assert_eq!(flags_rcode(pkt.header.flags), 0);

        assert_eq!(pkt.questions.len(), 1);
        assert_eq!(pkt.questions[0].name, "example.com");
        assert_eq!(pkt.questions[0].qtype, 1); // A
        assert_eq!(pkt.questions[0].qclass, 1); // IN

        assert_eq!(pkt.answers.len(), 1);
        let ans = &pkt.answers[0];
        // Compression pointer must resolve to the same name.
        assert_eq!(ans.name, "example.com");
        assert_eq!(ans.rtype, 1); // A
        assert_eq!(ans.rclass, 1); // IN
        assert_eq!(ans.ttl, 3600);
        assert_eq!(ans.rdata, vec![93, 184, 216, 34]);
    }

    #[test]
    fn rejects_short_input() {
        let err = parse(&[0x12, 0x34, 0x01]).unwrap_err();
        assert!(err.contains("too short"), "got: {err}");
    }

    #[test]
    fn rejects_truncated_question_section() {
        // Header says QD=1 but the buffer ends before QTYPE/QCLASS.
        let bytes = vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65,
            0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];
        let err = parse(&bytes).unwrap_err();
        assert!(err.contains("truncated question"), "got: {err}");
    }

    #[test]
    fn rejects_truncated_rdata() {
        // Response with RDLENGTH=4 but only 2 bytes of RDATA.
        let bytes = vec![
            0xab, 0xcd, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65,
            0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00,
            0x01, 0x07, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x0e, 0x10, 0x00, 0x04, 0x5d,
            0xb8, // only 2 of 4 RDATA bytes
        ];
        let err = parse(&bytes).unwrap_err();
        assert!(err.contains("RDATA truncated"), "got: {err}");
    }

    // ---- name compression ----

    #[test]
    fn compression_pointer_resolves_to_question_name() {
        // Build a response where the additional record's name is
        // a compression pointer pointing back to the question's
        // name (the canonical case from the parser-canonical-
        // response test, but as an additional record instead of
        // an answer). This catches off-by-one bugs in pointer
        // resolution — the pointer must point to the LENGTH byte,
        // not the first label char.
        //
        // Layout:
        //   [0..12]   header
        //   [12..15]  question name: "\x01a\x00"  (length at 12)
        //   [15..19]  QTYPE=A, QCLASS=IN (4 bytes)
        //   [19..21]  compression pointer to byte 12 (the question name)
        //   [21..31]  TYPE=A, CLASS=IN, TTL=60, RDLENGTH=4 (10 bytes)
        //   [31..35]  RDATA = 1.2.3.4 (4 bytes)
        let bytes = vec![
            0xab, 0xcd, 0x81, 0x80, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x61,
            0x00, // question: "a." (length at byte 12)
            0x00, 0x01, 0x00, 0x01, // QTYPE=A, QCLASS=IN  ends at byte 18
            // additional section starts at byte 19
            0xc0, 0x0c, // pointer to byte 0x0c = 12 = question name
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00,
            0x04, // TYPE/CLASS/TTL/RDLENGTH
            0x01, 0x02, 0x03, 0x04, // RDATA
        ];
        let pkt = parse(&bytes).unwrap();
        assert_eq!(pkt.additional.len(), 1);
        // 0xc0 0x0c → pointer target byte 12 → "a."
        assert_eq!(pkt.additional[0].name, "a");
        assert_eq!(pkt.additional[0].rdata, vec![1, 2, 3, 4]);
        assert_eq!(pkt.additional[0].ttl, 60);
    }

    #[test]
    fn forward_compression_pointer_is_rejected() {
        // A pointer at byte 20 pointing forward to byte 30 must be
        // rejected (RFC 1035 §4.1.4 requires backward pointers only).
        let bytes = vec![
            0xab, 0xcd, 0x81, 0x80, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x61,
            0x00, // question: "a." at byte 12
            0x00, 0x01, 0x00, 0x01, // QTYPE=A, QCLASS=IN  ends at byte 20
            // additional section: forward pointer at byte 20 -> 30
            0xc0, 0x1e, // points to byte 30 (forward!)
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x04, 0x01, 0x02, 0x03,
            0x04,
            // byte 30 is in the middle of the RDATA — pointer is invalid
        ];
        let err = parse(&bytes).unwrap_err();
        assert!(err.contains("forward") || err.contains("non-UTF8"), "got: {err}");
    }

    #[test]
    fn name_with_uppercase_is_normalised_to_lowercase() {
        // DNS names are case-insensitive (RFC 1035 §2.3.3); we
        // always emit lowercase for deterministic comparisons.
        let bytes = vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x46,
            0x4f, 0x4f, 0x02, 0x42, 0x41, 0x00, // "FOO.BA."
            0x00, 0x01, 0x00, 0x01,
        ];
        let pkt = parse(&bytes).unwrap();
        assert_eq!(pkt.questions[0].name, "foo.ba");
    }

    #[test]
    fn empty_message_with_zero_counts_parses() {
        // A header-only DNS message with all counts at zero.
        let bytes = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let pkt = parse(&bytes).unwrap();
        assert_eq!(pkt.questions.len(), 0);
        assert_eq!(pkt.answers.len(), 0);
        assert_eq!(pkt.authority.len(), 0);
        assert_eq!(pkt.additional.len(), 0);
    }

    #[test]
    fn label_length_63_is_accepted_max_legal_label() {
        // A label of exactly 63 chars is legal (the upper bound of
        // the 6-bit length field per RFC 1035 §2.3.4). Build a
        // question name = "aaa...aaa.example.com" where the first
        // label is 63 'a's and the second is "example", "com", root.
        let mut bytes =
            vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        bytes.push(63); // 6-bit label length = 63 (legal max)
        for _ in 0..63 {
            bytes.push(b'a');
        }
        bytes.extend_from_slice(&[7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 0]);
        bytes.extend_from_slice(&[0, 1, 0, 1]); // QTYPE=A, QCLASS=IN
        let pkt = parse(&bytes).unwrap();
        assert_eq!(pkt.questions[0].name.len(), 63 + 8); // 63 'a' + ".example" + 1 char of leading dot? actually it's "aaa...aaa.example"
        let first_label = pkt.questions[0].name.split('.').next().unwrap();
        assert_eq!(first_label.len(), 63);
    }

    #[test]
    fn unsupported_extended_label_type_is_rejected() {
        // A label whose high two bits are 01 (extended label,
        // RFC 2673 / RFC 6891) is declared unsupported by this
        // module. Build a label byte of 0x40 at byte 12.
        let bytes = vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00,
            0x00, 0x01, 0x00, 0x01, // extended-label byte + body
        ];
        let err = parse(&bytes).unwrap_err();
        assert!(err.contains("unsupported label type"), "got: {err}");
    }

    #[test]
    fn nxdomain_response_with_rcode_3() {
        // ID=0xBEEF, FLAGS=0x8183 (response, RD, RA, RCODE=3=NXDOMAIN),
        // QD=1, AN=0, NS=0, AR=0
        // Question: doesnotexist.invalid A IN
        let bytes = vec![
            0xbe, 0xef, 0x81, 0x83, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x64,
            0x6f, 0x65, 0x73, 0x6e, 0x6f, 0x74, 0x65, 0x78, 0x69, 0x73, 0x74, 0x07, 0x69, 0x6e,
            0x76, 0x61, 0x6c, 0x69, 0x64, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];
        let pkt = parse(&bytes).unwrap();
        assert_eq!(flags_rcode(pkt.header.flags), 3);
        assert!(flags_qr(pkt.header.flags));
        assert_eq!(pkt.questions[0].name, "doesnotexist.invalid");
        assert_eq!(pkt.questions[0].qtype, 1);
        assert!(pkt.answers.is_empty());
    }

    #[test]
    fn multiple_questions_and_answers_parsed_in_order() {
        // Two questions, two answers. Verifies counts are honoured
        // and that ANCOUNT/NSCOUNT/ARCOUNT section boundaries are
        // not crossed.
        //
        // Question 1: alpha.test A IN
        // Question 2: beta.test A IN
        // Answer 1:   alpha.test A IN TTL=60 RDATA=1.2.3.4
        // Answer 2:   beta.test A IN TTL=120 RDATA=5.6.7.8
        let bytes = vec![
            // header
            0x00, 0x01, 0x01, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, // Q1
            0x05, b'a', b'l', b'p', b'h', b'a', 0x04, b't', b'e', b's', b't', 0x00, 0x00, 0x01,
            0x00, 0x01, // Q2
            0x04, b'b', b'e', b't', b'a', 0x04, b't', b'e', b's', b't', 0x00, 0x00, 0x01, 0x00,
            0x01, // A1
            0x05, b'a', b'l', b'p', b'h', b'a', 0x04, b't', b'e', b's', b't', 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x04, 1, 2, 3, 4, // A2
            0x04, b'b', b'e', b't', b'a', 0x04, b't', b'e', b's', b't', 0x00, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x78, 0x00, 0x04, 5, 6, 7, 8,
        ];
        let pkt = parse(&bytes).unwrap();
        assert_eq!(pkt.questions.len(), 2);
        assert_eq!(pkt.answers.len(), 2);
        assert_eq!(pkt.questions[0].name, "alpha.test");
        assert_eq!(pkt.questions[1].name, "beta.test");
        assert_eq!(pkt.answers[0].name, "alpha.test");
        assert_eq!(pkt.answers[0].ttl, 60);
        assert_eq!(pkt.answers[0].rdata, vec![1, 2, 3, 4]);
        assert_eq!(pkt.answers[1].name, "beta.test");
        assert_eq!(pkt.answers[1].ttl, 120);
        assert_eq!(pkt.answers[1].rdata, vec![5, 6, 7, 8]);
    }
}
