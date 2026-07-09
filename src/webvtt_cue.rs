// Minimal WebVTT cue parser.
//
// Spec: WebVTT (W3C) https://www.w3.org/TR/webvtt1/
//
// A WebVTT file looks like:
//
//   WEBVTT
//
//   NOTE this is a comment block spanning lines
//
//   cue-identifier (optional)
//   00:00:00.000 --> 00:00:05.000 line:class.foo
//   payload line one
//   payload line two
//
//   00:00:05.500 --> 00:00:10.000
//   single line payload
//
// Timestamp format is HH:MM:SS.mmm or MM:SS.mmm.
// `line`, `position`, `size`, `align`, `vertical` may follow the timing
// arrow, separated by whitespace.
//
// `NOTE` blocks are comments and are ignored.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueTiming {
    pub start_ms: u32,
    pub end_ms: u32,
    /// Raw settings string after the `-->`. Empty if no settings.
    pub settings: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cue {
    pub identifier: String,
    pub timing: CueTiming,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebVtt {
    pub cues: Vec<Cue>,
}

/// Parse `HH:MM:SS.mmm` or `MM:SS.mmm` into milliseconds.
/// Returns `None` on malformed input.
pub fn parse_timestamp(s: &str) -> Option<u32> {
    let (hms, ms) = match s.rsplit_once('.') {
        Some((hms, ms)) => (hms, ms),
        None => (s, "0"),
    };
    let ms: u32 = ms.parse().ok()?;
    let parts: Vec<&str> = hms.split(':').collect();
    let (h, m, sec) = match parts.as_slice() {
        [h, m, sec] => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?, sec.parse::<u32>().ok()?),
        [m, sec] => (0u32, m.parse::<u32>().ok()?, sec.parse::<u32>().ok()?),
        _ => return None,
    };
    if m >= 60 || sec >= 60 || ms >= 1000 {
        return None;
    }
    Some(((h * 3600 + m * 60 + sec) * 1000) + ms)
}

/// Split a WebVTT file into the comment/NOTE preamble and the cue blocks.
/// Blank lines are block separators.
fn collect_blocks(text: &str) -> Vec<Vec<String>> {
    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() {
            if !cur.is_empty() {
                blocks.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(line.to_string());
        }
    }
    if !cur.is_empty() {
        blocks.push(cur);
    }
    blocks
}

/// Parse a single cue block. The first line is either an identifier
/// (no `-->`, doesn't look like a timing line) or the timing line itself.
fn parse_block(block: &[String]) -> Result<Cue, String> {
    if block.is_empty() {
        return Err("empty block".into());
    }
    let (identifier, timing_idx) = if block[0].contains("-->") {
        (String::new(), 0)
    } else {
        if block.len() < 2 {
            return Err("block missing timing".into());
        }
        (block[0].clone(), 1)
    };
    let timing_line = &block[timing_idx];
    parse_cue(identifier, timing_line, &block[timing_idx + 1..])
}

fn parse_cue(
    identifier: String,
    timing_line: &str,
    payload_lines: &[String],
) -> Result<Cue, String> {
    let arrow = timing_line.find("-->").ok_or("missing --> arrow")?;
    let start = timing_line[..arrow].trim();
    let rest = timing_line[arrow + 3..].trim_start();
    let end_split = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let end = rest[..end_split].trim();
    let settings = rest[end_split..].trim().to_string();
    let start_ms =
        parse_timestamp(start).ok_or_else(|| format!("bad start timestamp: {start:?}"))?;
    let end_ms = parse_timestamp(end).ok_or_else(|| format!("bad end timestamp: {end:?}"))?;
    if end_ms < start_ms {
        return Err(format!("end before start: {start_ms}..{end_ms}"));
    }
    let payload = payload_lines.join("\n");
    Ok(Cue { identifier, timing: CueTiming { start_ms, end_ms, settings }, payload })
}

/// Parse a complete WebVTT document.
pub fn parse(input: &str) -> Result<WebVtt, String> {
    let text = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let mut lines = text.split('\n').peekable();
    // First non-empty line must begin with "WEBVTT".
    let header = loop {
        let line = match lines.next() {
            None => return Err("missing WEBVTT header".into()),
            Some(l) => l.strip_suffix('\r').unwrap_or(l).to_string(),
        };
        if line.is_empty() {
            continue;
        }
        break line;
    };
    let first_token = header.split_whitespace().next().unwrap_or("");
    if first_token != "WEBVTT" {
        return Err(format!("not a WebVTT document, first line: {header:?}"));
    }
    // Skip any header-text lines until the first blank line or cue timing.
    while let Some(l) = lines.peek() {
        let trimmed = l.strip_suffix('\r').unwrap_or(l).trim();
        if trimmed.is_empty() {
            lines.next();
            break;
        }
        if trimmed.contains("-->") {
            break;
        }
        // Header text line — skip it.
        lines.next();
    }
    let body: String = lines.collect::<Vec<_>>().join("\n");
    let blocks = collect_blocks(&body);
    let mut cues = Vec::new();
    for block in &blocks {
        if block[0].starts_with("NOTE") {
            continue;
        }
        if block[0].starts_with("STYLE") || block[0].starts_with("REGION") {
            continue;
        }
        let cue = parse_block(block)?;
        cues.push(cue);
    }
    Ok(WebVtt { cues })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timestamp_basic() {
        assert_eq!(parse_timestamp("00:00:00.000"), Some(0));
        assert_eq!(parse_timestamp("00:00:01.500"), Some(1500));
        assert_eq!(parse_timestamp("01:02:03.004"), Some(3723_004));
    }

    #[test]
    fn parse_timestamp_minutes_only() {
        assert_eq!(parse_timestamp("00:05.000"), Some(5000));
        assert_eq!(parse_timestamp("05:00.000"), Some(300_000));
    }

    #[test]
    fn parse_timestamp_rejects_overflow() {
        assert_eq!(parse_timestamp("00:60:00.000"), None);
        assert_eq!(parse_timestamp("00:00:60.000"), None);
        assert_eq!(parse_timestamp("00:00:00.1000"), None);
        assert_eq!(parse_timestamp("garbage"), None);
    }

    #[test]
    fn parse_minimal_cue() {
        let input = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nhello world\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues.len(), 1);
        assert_eq!(v.cues[0].identifier, "");
        assert_eq!(v.cues[0].timing.start_ms, 1000);
        assert_eq!(v.cues[0].timing.end_ms, 2000);
        assert_eq!(v.cues[0].payload, "hello world");
        assert_eq!(v.cues[0].timing.settings, "");
    }

    #[test]
    fn parse_cue_with_identifier_and_settings() {
        let input = "WEBVTT\n\nintro\n00:00:00.000 --> 00:00:05.000 line:0 position:50%\nfirst line\nsecond line\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues.len(), 1);
        assert_eq!(v.cues[0].identifier, "intro");
        assert_eq!(v.cues[0].timing.start_ms, 0);
        assert_eq!(v.cues[0].timing.end_ms, 5000);
        assert_eq!(v.cues[0].timing.settings, "line:0 position:50%");
        assert_eq!(v.cues[0].payload, "first line\nsecond line");
    }

    #[test]
    fn parse_multiple_cues() {
        let input =
            "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\none\n\n00:00:02.500 --> 00:00:03.000\ntwo\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues.len(), 2);
        assert_eq!(v.cues[0].payload, "one");
        assert_eq!(v.cues[1].timing.start_ms, 2500);
        assert_eq!(v.cues[1].timing.end_ms, 3000);
        assert_eq!(v.cues[1].payload, "two");
    }

    #[test]
    fn parse_skips_note_block() {
        let input = "WEBVTT\n\nNOTE this is a\nmulti-line note\n\n00:00:00.000 --> 00:00:01.000\nafter-note\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues.len(), 1);
        assert_eq!(v.cues[0].payload, "after-note");
    }

    #[test]
    fn parse_rejects_bad_header() {
        assert!(parse("NOPE\n\n00:00:00.000 --> 00:00:01.000\nx\n").is_err());
    }

    #[test]
    fn parse_rejects_end_before_start() {
        let input = "WEBVTT\n\n00:00:05.000 --> 00:00:01.000\nbad\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_handles_crlf() {
        let input = "WEBVTT\r\n\r\n00:00:00.000 --> 00:00:01.000\r\nhi\r\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues[0].payload, "hi");
    }

    #[test]
    fn parse_minutes_only_timestamp_in_cue() {
        // MM:SS.mmm form is valid.
        let input = "WEBVTT\n\n05:30.000 --> 06:00.000\nhalfway\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues[0].timing.start_ms, 330_000);
        assert_eq!(v.cues[0].timing.end_ms, 360_000);
    }

    #[test]
    fn parse_header_text_then_cue() {
        // "WEBVTT - Some title" header form, blank line, then cue.
        let input = "WEBVTT - subtitle track\n\n00:00:00.000 --> 00:00:01.000\nfirst\n";
        let v = parse(input).unwrap();
        assert_eq!(v.cues.len(), 1);
        assert_eq!(v.cues[0].payload, "first");
    }
}
