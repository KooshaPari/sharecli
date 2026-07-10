// Minimal ASC X12 EDI segment parser.
//
// X12 ( Accredited Standards Committee X12 ) is the dominant EDI standard
// in US healthcare supply-chain traffic. The wire format is plain ASCII
// with three reserved separator bytes that the ISA interchange header
// itself defines:
//
//   * element separator: usually `*`
//   * sub-element (component) separator: usually `:` (often `>`)
//   * segment terminator: usually `~` (sometimes `\n`)
//
// The ISA envelope is exactly 106 characters wide and the very first
// three bytes ARE the segment terminator + two data-element separators
// (in that order). Every other segment uses those declared separators.
//
// We expose a deliberately small surface:
//
//   * `Segment { id, elements }`  — a parsed segment with one
//     string per composite element. Composite elements (those whose
//     content may contain a sub-element separator) are stored as a
//     `Vec<String>` of sub-components; the outer `Vec<Vec<String>>`
//     is the segment's elements.
//   * `parse_segment(line, sep, comp_sep, term)` — parse a single
//     segment line given the resolved separators.
//   * `parse_interchange(input)` — discover separators from the ISA
//     header, parse the whole interchange into a `Vec<Segment>`.
//   * `separators_from_isa(isa)` — pull `(comp_sep, sep, term)` from
//     a 106-char ISA header string.
//
// Notes:
//
//   * We DO NOT validate against a transaction-set schema. This is a
//     structural parser — it enforces only that segments are well-
//     formed (non-empty id, separator count is consistent).
//   * We DO NOT validate element counts per transaction set.
//   * The ISA segment itself is length 16 (16 elements) and starts
//     with `ISA` (positions 0-2) followed by the segment terminator
//     (positions 3) and the two declared separators (positions
//     4-5); element 1 starts at position 6.

/// A single X12 segment: identifier plus the composite elements
/// that follow it. `elements[0]` is always the first real element
/// of the segment (after the segment ID); for `ISA` the ID is
/// `elements[0][0]`, and `elements[0]` would be a composite field
/// only if the standard calls for one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The segment identifier (e.g. `"ISA"`, `"GS"`, `"ST"`, `"BHT"`,
    /// `"NM1"`, `"CLM"`, `"SV1"`, `"SE"`). For ISA it is the
    /// literal string `"ISA"`.
    pub id: String,
    /// All composite elements that follow the segment ID. Each
    /// composite element is a `Vec<String>` of sub-components
    /// (one component if no sub-element separator is present in
    /// the wire form).
    pub elements: Vec<Vec<String>>,
}

impl Segment {
    /// The first composite element (e.g. for ISA, this is the
    /// authorization info qualifier `ISA01`). Returns `None`
    /// when the segment carries no elements (only an ID, like
    /// `"SE"` followed by nothing else).
    pub fn first_element(&self) -> Option<&Vec<String>> {
        self.elements.first()
    }

    /// Number of composite elements (excludes the segment ID).
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

/// Parsed envelope: the three separators inferred from an ISA
/// header. Defaults assume the common HIPAA variant when an
/// interchange has no ISA header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Separators {
    /// Sub-element (component) separator, typically `:`.
    pub component: char,
    /// Data-element separator, typically `*`.
    pub element: char,
    /// Segment terminator, typically `~`.
    pub segment: char,
}

impl Separators {
    /// Default HIPAA / 5010 separators.
    pub const HIPAA_DEFAULT: Self = Self { component: ':', element: '*', segment: '~' };
}

impl Default for Separators {
    fn default() -> Self {
        Self::HIPAA_DEFAULT
    }
}

/// Pull `(component, element, segment)` separators from a 106-character
/// ISA header. The ISA header declares them in fixed positions:
///
///   * byte   3:    data-element separator (between the literal
///                  "ISA" tag and ISA01)
///   * byte 104:    sub-element (component) separator — this is
///                  the value of field ISA16
///   * byte 105:    segment terminator (the very last character)
///
/// Returns `Err` if the input is shorter than 106 bytes or if the
/// ISA id at byte 0..3 is not `"ISA"`.
pub fn separators_from_isa(isa: &str) -> Result<Separators, String> {
    if isa.len() < 106 {
        return Err(format!("ISA header too short: {} bytes (need 106)", isa.len()));
    }
    if !isa.starts_with("ISA") {
        return Err(format!(
            "ISA header does not begin with 'ISA': got {:?}",
            &isa[..isa.len().min(3)]
        ));
    }
    let elem = isa.as_bytes()[3] as char;
    let comp = isa.as_bytes()[104] as char;
    let term = isa.as_bytes()[105] as char;
    Ok(Separators { component: comp, element: elem, segment: term })
}

/// Parse a single segment line given resolved separators.
///
/// The input MUST NOT include the trailing newline; the segment
/// terminator is consumed if present but not required.
///
/// A segment is split into elements by `element_sep`. Each element
/// is then split into sub-components by `component_sep`. The very
/// first token (before the first `element_sep`) is the segment ID.
/// An empty input or an input that begins with the segment
/// terminator yields an `Err`.
pub fn parse_segment(
    line: &str,
    element_sep: char,
    component_sep: char,
) -> Result<Segment, String> {
    if line.is_empty() {
        return Err("empty segment line".to_string());
    }

    // Strip trailing terminator if present (some producers emit it
    // on every segment; some omit it on the last one).
    let trimmed = line.trim_end_matches(['~', '\n', '\r']);
    if trimmed.is_empty() {
        return Err("segment line is only a terminator".to_string());
    }

    let tokens: Vec<&str> = trimmed.split(element_sep).collect();
    if tokens.is_empty() {
        return Err("segment has no tokens".to_string());
    }

    let id = tokens[0];
    if id.is_empty() {
        return Err("segment ID is empty".to_string());
    }
    if !id.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
        return Err(format!("segment ID {:?} must be uppercase ASCII letters/digits", id));
    }

    let mut elements: Vec<Vec<String>> = Vec::with_capacity(tokens.len() - 1);
    for raw in &tokens[1..] {
        // ISA segment values are exactly two characters and never
        // contain the component separator, so the split still works.
        let parts: Vec<String> = raw.split(component_sep).map(|s| s.to_string()).collect();
        elements.push(parts);
    }

    Ok(Segment { id: id.to_string(), elements })
}

/// Discover separators from the leading ISA header and parse the
/// entire interchange into a `Vec<Segment>`. The interchange may
/// include any number of segments after the ISA; a trailing `IEA`
/// is required by the X12 standard but is not enforced here.
///
/// Lines may be separated by `\n`, `\r\n`, or just the segment
/// terminator. Empty lines are skipped.
pub fn parse_interchange(input: &str) -> Result<Vec<Segment>, String> {
    if input.is_empty() {
        return Err("interchange is empty".to_string());
    }

    // Locate the ISA header end — by the standard, ISA is 106 bytes
    // long when measured in raw characters. We trust the producer:
    // if the first three bytes are "ISA", the 106th byte onward is
    // the first segment after ISA.
    if !input.starts_with("ISA") {
        return Err("interchange does not begin with ISA".to_string());
    }
    if input.len() < 106 {
        return Err(format!("interchange shorter than one ISA header: {} bytes", input.len()));
    }

    let seps = separators_from_isa(&input[..106])?;
    let rest = &input[106..];

    let mut segments = Vec::new();
    // Reserve the ISA segment itself.
    segments.push(Segment {
        id: "ISA".to_string(),
        elements: parse_isa_elements(&input[..106], seps.element, seps.component, seps.segment)?,
    });

    for raw_line in rest.split(seps.segment) {
        // Strip CR/LF padding around each segment.
        let line = raw_line.trim_matches(['\n', '\r', ' ']);
        if line.is_empty() {
            continue;
        }
        segments.push(parse_segment(line, seps.element, seps.component)?);
    }

    Ok(segments)
}

/// Split the 106-character ISA header into composite elements
/// using the declared separators. The 16 ISA fields are
/// element-separated, and ISA16 (the component separator) is the
/// last field — it sits directly before the segment terminator
/// with no element separator between them. We split the byte
/// 3..105 window on the element separator; the result starts
/// with a leading empty string (because byte 3 IS the element
/// separator) and the final token is `ISA16 + segment_term` glued
/// together. We drop both ends. The result is exactly 16 elements
/// per the standard.
fn parse_isa_elements(
    isa: &str,
    element_sep: char,
    component_sep: char,
    segment_term: char,
) -> Result<Vec<Vec<String>>, String> {
    // Body window: bytes 3..105 of the ISA header. Byte 3 is the
    // leading element separator; byte 105 is the segment terminator
    // (which we strip from the final token, not from the window).
    let bytes = isa.as_bytes();
    if bytes.len() < 106 {
        return Err("ISA header too short".to_string());
    }
    let body_bytes = &bytes[3..106]; // include the segment terminator
    let body = std::str::from_utf8(body_bytes).map_err(|e| format!("ISA body not UTF-8: {}", e))?;

    let mut out = Vec::new();
    for raw in body.split(element_sep) {
        // Strip a trailing segment terminator from the very last
        // token only (since the last token is `ISA16 + segment_term`).
        let cleaned = if raw.ends_with(segment_term) { &raw[..raw.len() - 1] } else { raw };
        let parts: Vec<String> = cleaned.split(component_sep).map(|s| s.to_string()).collect();
        out.push(parts);
    }
    // Drop the leading empty token (the gap before ISA01, caused
    // by the byte 3 element separator).
    if !out.is_empty() && out[0].len() == 1 && out[0][0].is_empty() {
        out.remove(0);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- separators_from_isa ----

    #[test]
    fn separators_from_canonical_isa_header() {
        // Build a 106-char ISA header byte-by-byte. The layout is:
        //   "ISA" + '*' + ISA01 + '*' + ISA02 + '*' + ... + ISA15 + '*' + ISA16 + '~'
        // ISA16 is the component separator and the final byte is
        // the segment terminator. There is no trailing element
        // separator after ISA16.
        let fields = [
            "00",              // ISA01
            "          ",      // ISA02 (10 spaces)
            "00",              // ISA03
            "          ",      // ISA04
            "ZZ",              // ISA05
            "SUBMITTERID    ", // ISA06 (15 chars)
            "ZZ",              // ISA07
            "RECEIVERID     ", // ISA08
            "250101",          // ISA09
            "1200",            // ISA10
            "^",               // ISA11 repetition separator
            "00501",           // ISA12
            "000000001",       // ISA13
            "0",               // ISA14
            "P",               // ISA15
            ":",               // ISA16 = component separator
        ];
        assert_eq!(fields.len(), 16);
        let mut s = String::with_capacity(106);
        s.push_str("ISA");
        for f in &fields {
            s.push('*');
            s.push_str(f);
        }
        s.push('~');
        assert_eq!(s.len(), 106, "ISA fixture is wrong: {}", s);

        let seps = separators_from_isa(&s).unwrap();
        assert_eq!(seps.element, '*', "byte 3 must be element separator");
        assert_eq!(seps.component, ':', "byte 104 must be component separator");
        assert_eq!(seps.segment, '~', "byte 105 must be segment terminator");
    }

    #[test]
    fn separators_from_isa_rejects_short_input() {
        let err = separators_from_isa("ISA~00*").unwrap_err();
        assert!(err.contains("too short"), "got: {err}");
    }

    #[test]
    fn separators_from_isa_rejects_non_isa_id() {
        // 106 bytes but starts with "GSB"
        let mut s = String::with_capacity(106);
        s.push_str("GSB");
        while s.len() < 106 {
            s.push(' ');
        }
        let err = separators_from_isa(&s).unwrap_err();
        assert!(err.contains("does not begin with 'ISA'"), "got: {err}");
    }

    // ---- parse_segment ----

    #[test]
    fn parse_simple_segment_no_components() {
        let seg = parse_segment("ST*270*0001", '*', ':').unwrap();
        assert_eq!(seg.id, "ST");
        assert_eq!(seg.elements, vec![vec!["270"], vec!["0001"]]);
    }

    #[test]
    fn parse_segment_with_component_separator() {
        // NM1*IL*1*DOE*JOHN***MI*12345678A
        // Real-world NM1*IL has these fields (per HIPAA 270/271):
        //   NM101=IL, NM102=1, NM103=DOE, NM104=JOHN,
        //   NM105/NM106/NM107 (middle name / prefix / suffix) = empty/empty/empty,
        //   NM108=MI (id code qualifier), NM109=12345678A.
        let seg = parse_segment("NM1*IL*1*DOE*JOHN***MI*12345678A", '*', ':').unwrap();
        assert_eq!(seg.id, "NM1");
        // 8 elements expected after the ID:
        assert_eq!(seg.elements.len(), 8);
        assert_eq!(seg.elements[0], vec!["IL"]);
        assert_eq!(seg.elements[1], vec!["1"]);
        assert_eq!(seg.elements[2], vec!["DOE"]);
        assert_eq!(seg.elements[3], vec!["JOHN"]);
        assert_eq!(seg.elements[4], vec![""]);
        assert_eq!(seg.elements[5], vec![""]);
        assert_eq!(seg.elements[6], vec!["MI"]);
        assert_eq!(seg.elements[7], vec!["12345678A"]);
    }

    #[test]
    fn parse_segment_with_explicit_component_in_one_field() {
        // Composite element on the SV1 service line, where the
        // composite procedure code is "HC:99213".
        let seg = parse_segment("SV1*HC:99213*150*UN*1***1", '*', ':').unwrap();
        assert_eq!(seg.id, "SV1");
        assert_eq!(seg.elements[0], vec!["HC", "99213"]);
        assert_eq!(seg.elements[1], vec!["150"]);
        assert_eq!(seg.elements[2], vec!["UN"]);
        assert_eq!(seg.elements[3], vec!["1"]);
    }

    #[test]
    fn parse_segment_strips_trailing_terminator() {
        let seg = parse_segment("SE*8*0001~", '*', ':').unwrap();
        assert_eq!(seg.id, "SE");
        assert_eq!(seg.elements, vec![vec!["8"], vec!["0001"]]);
    }

    #[test]
    fn parse_segment_rejects_empty_id() {
        let err = parse_segment("*270*0001", '*', ':').unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_segment_rejects_lowercase_id() {
        let err = parse_segment("st*270*0001", '*', ':').unwrap_err();
        assert!(err.contains("uppercase"), "got: {err}");
    }

    // ---- parse_interchange ----

    #[test]
    fn parse_interchange_single_segment_after_isa() {
        // Build a minimal interchange: just an ISA header and a single
        // dummy segment after it.
        let isa = build_test_isa();
        let body = format!("{isa}ST*270*0001~");
        let segs = parse_interchange(&body).unwrap();
        // ISA + ST
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].id, "ISA");
        assert_eq!(segs[1].id, "ST");
        assert_eq!(segs[1].elements, vec![vec!["270"], vec!["0001"]]);
    }

    #[test]
    fn parse_interchange_full_270_loop() {
        // ISA + GS + ST + BHT + NM1*IL + HL + TRN + SE + GE + IEA
        let isa = build_test_isa();
        let body = format!(
            "{isa}\
             GS*HC*SUB*REC*20250101*1200*1*X*005010X279A1~\
             ST*270*0001*005010X279A1~\
             BHT*0022*13*REF123*20250101*1200~\
             NM1*IL*1*DOE*JOHN***MI*12345678A~\
             SE*4*0001~\
             GE*1*1~\
             IEA*1*000000001~"
        );
        let segs = parse_interchange(&body).unwrap();
        let ids: Vec<&str> = segs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["ISA", "GS", "ST", "BHT", "NM1", "SE", "GE", "IEA"]);

        let nm1 = &segs[4];
        assert_eq!(nm1.id, "NM1");
        // NM1*IL layout: IL / 1 / DOE / JOHN / "" / "" / MI / 12345678A
        // (2 empty fields between JOHN and MI correspond to
        // NM105/NM106 — middle name and prefix).
        assert_eq!(nm1.elements[0], vec!["IL"]);
        assert_eq!(nm1.elements[6], vec!["MI"]);
        assert_eq!(nm1.elements[7], vec!["12345678A"]);

        // ISA has 16 elements per the standard.
        let isa_seg = &segs[0];
        assert_eq!(isa_seg.id, "ISA");
        assert_eq!(isa_seg.elements.len(), 16);
    }

    #[test]
    fn parse_interchange_rejects_empty_input() {
        let err = parse_interchange("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_interchange_rejects_short_input() {
        // Starts with ISA but shorter than 106 bytes.
        let err = parse_interchange("ISA~").unwrap_err();
        assert!(err.contains("shorter than one ISA"), "got: {err}");
    }

    #[test]
    fn parse_interchange_rejects_non_isa_prefix() {
        let err = parse_interchange("NOT_AN_INTERCHANGE").unwrap_err();
        assert!(err.contains("does not begin with ISA"), "got: {err}");
    }

    // ---- helpers ----

    /// Build a 106-character canonical ISA header with the standard
    /// HIPAA separators (`*`, `:`, `~`). All content fields are
    /// padded with spaces so that the parser sees the same number of
    /// element separators it expects.
    ///
    /// Layout: `"ISA" + '*' + <16 fields> + '~'`. The 16th field
    /// (ISA16) is the component separator itself, and the final
    /// byte is the segment terminator. There is no element
    /// separator between ISA16 and the segment terminator.
    fn build_test_isa() -> String {
        let fields = [
            "00",              // ISA01
            "          ",      // ISA02 (10 spaces)
            "00",              // ISA03
            "          ",      // ISA04
            "ZZ",              // ISA05
            "SUBMITTERID    ", // ISA06 (15 chars)
            "ZZ",              // ISA07
            "RECEIVERID     ", // ISA08
            "250101",          // ISA09
            "1200",            // ISA10
            "^",               // ISA11
            "00501",           // ISA12
            "000000001",       // ISA13
            "0",               // ISA14
            "P",               // ISA15
            ":",               // ISA16 = component separator
        ];
        let mut s = String::with_capacity(106);
        s.push_str("ISA");
        for f in &fields {
            s.push('*');
            s.push_str(f);
        }
        s.push('~');
        assert_eq!(s.len(), 106, "test ISA header must be exactly 106 bytes");
        s
    }
}
