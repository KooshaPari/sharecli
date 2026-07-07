// Minimal CDR / CD-DA cue sheet parser.
//
// Spec: see "cuesheet format" (e.g. CDRWIN / EAC conventions).
// This module handles the common shapes:
//
//   TITLE "Album Title"
//   PERFORMER "Artist Name"
//   FILE "audio.wav" WAVE
//     TRACK 01 AUDIO
//       TITLE "Track One"
//       PERFORMER "Track Artist"
//       INDEX 01 00:00:00
//       INDEX 00 00:30:00
//     TRACK 02 MODE1/2352
//       PREGAP 00:02:00
//       INDEX 01 05:00:00
//     TRACK 03 AUDIO
//       POSTGAP 00:00:10
//       INDEX 01 08:42:12
//
// `FILE` may declare a mode (`WAVE`, `MP3`, `BINARY`) — we only
// surface the filename string here. `TRACK` may declare a mode of
// `AUDIO` or `MODE1/2048`, `MODE1/2352`, `MODE2/2336`, `MODE2/2352`.
// `INDEX` lines reference frames (1/75 second). `PREGAP` and
// `POSTGAP` use the same MM:SS:FF format.
//
// This module is intentionally minimal: line-based parser, no
// continuation support, single FILE per sheet (additional FILE
// blocks reset `file` but are not commonly seen).

/// One track from a cue sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueTrack {
    /// Track number as it appears in the sheet (1..=99 typically).
    pub number: u8,
    /// Track mode (`"AUDIO"`, `"MODE1/2048"`, `"MODE1/2352"`,
    /// `"MODE2/2336"`, `"MODE2/2352"`).
    pub mode: String,
    /// All `INDEX` lines as `(index_id, mm, ss, ff)`. Order is
    /// preserved as encountered.
    pub indices: Vec<(u8, u8, u8, u8)>,
    /// `PREGAP` if present, parsed to `(mm, ss, ff)`.
    pub pregap: Option<(u8, u8, u8)>,
    /// `POSTGAP` if present, parsed to `(mm, ss, ff)`.
    pub postgap: Option<(u8, u8, u8)>,
}

/// A parsed cue sheet (single FILE block).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueSheet {
    /// Top-level `TITLE` if present.
    pub title: String,
    /// Top-level `PERFORMER` if present.
    pub performer: String,
    /// Filename from the `FILE` directive (without quotes / mode).
    pub file: String,
    /// All tracks in source order.
    pub tracks: Vec<CueTrack>,
}

const VALID_MODES: &[&str] = &[
    "AUDIO",
    "MODE1/2048",
    "MODE1/2352",
    "MODE2/2336",
    "MODE2/2352",
];

/// Parse a cue sheet from `input`. Returns an error for:
///   * malformed `MM:SS:FF` timestamps,
///   * `TRACK` blocks with an unknown mode,
///   * missing FILE directive,
///   * TRACK with no INDEX 01.
pub fn parse(input: &str) -> Result<CueSheet, String> {
    let mut sheet = CueSheet {
        title: String::new(),
        performer: String::new(),
        file: String::new(),
        tracks: Vec::new(),
    };

    let mut current: Option<CueTrack> = None;

    for (lineno, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let lineno = lineno + 1;

        if let Some(rest) = line.strip_prefix("TITLE ") {
            let val = parse_quoted(rest, lineno)?;
            if let Some(ref mut t) = current {
                // track-level TITLE: ignore for the minimal struct.
                let _ = t;
                let _ = val;
            } else {
                sheet.title = val;
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("PERFORMER ") {
            let val = parse_quoted(rest, lineno)?;
            if current.is_some() {
                let _ = val;
            } else {
                sheet.performer = val;
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("FILE ") {
            // Close any open track when a new FILE block starts.
            if let Some(t) = current.take() {
                sheet.tracks.push(t);
            }
            let val = parse_quoted(rest, lineno)?;
            // Strip trailing mode token (WAVE / MP3 / BINARY) if present.
            sheet.file = val
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
            continue;
        }

        if let Some(rest) = line.strip_prefix("TRACK ") {
            // Flush previous track.
            if let Some(t) = current.take() {
                sheet.tracks.push(t);
            }
            let mut parts = rest.split_whitespace();
            let num_str = parts
                .next()
                .ok_or_else(|| format!("line {}: TRACK missing number", lineno))?;
            let num: u8 = num_str
                .parse()
                .map_err(|e| format!("line {}: invalid TRACK number '{}': {}", lineno, num_str, e))?;
            let mode = parts
                .collect::<Vec<_>>()
                .join(" ");
            let mode = mode.trim().to_string();
            if !VALID_MODES.contains(&mode.as_str()) {
                return Err(format!(
                    "line {}: unknown TRACK mode '{}'",
                    lineno, mode
                ));
            }
            current = Some(CueTrack {
                number: num,
                mode,
                indices: Vec::new(),
                pregap: None,
                postgap: None,
            });
            continue;
        }

        if let Some(rest) = line.strip_prefix("INDEX ") {
            let t = current
                .as_mut()
                .ok_or_else(|| format!("line {}: INDEX outside of TRACK", lineno))?;
            let mut parts = rest.split_whitespace();
            let id_str = parts
                .next()
                .ok_or_else(|| format!("line {}: INDEX missing id", lineno))?;
            let id: u8 = id_str
                .parse()
                .map_err(|e| format!("line {}: invalid INDEX id '{}': {}", lineno, id_str, e))?;
            let stamp = parts
                .next()
                .ok_or_else(|| format!("line {}: INDEX missing timestamp", lineno))?;
            let (mm, ss, ff) = parse_msf(stamp, lineno)?;
            t.indices.push((id, mm, ss, ff));
            continue;
        }

        if let Some(rest) = line.strip_prefix("PREGAP ") {
            let t = current
                .as_mut()
                .ok_or_else(|| format!("line {}: PREGAP outside of TRACK", lineno))?;
            t.pregap = Some(parse_msf(rest.trim(), lineno)?);
            continue;
        }

        if let Some(rest) = line.strip_prefix("POSTGAP ") {
            let t = current
                .as_mut()
                .ok_or_else(|| format!("line {}: POSTGAP outside of TRACK", lineno))?;
            t.postgap = Some(parse_msf(rest.trim(), lineno)?);
            continue;
        }

        // Unknown directive — silently ignore for forward-compat.
    }

    if let Some(t) = current.take() {
        sheet.tracks.push(t);
    }

    if sheet.file.is_empty() {
        return Err("missing FILE directive".to_string());
    }
    if sheet.tracks.is_empty() {
        return Err("cue sheet has no TRACK blocks".to_string());
    }
    for t in &sheet.tracks {
        if !t.indices.iter().any(|(id, _, _, _)| *id == 1) {
            return Err(format!(
                "track {} is missing INDEX 01",
                t.number
            ));
        }
    }

    Ok(sheet)
}

fn parse_quoted(rest: &str, lineno: usize) -> Result<String, String> {
    let rest = rest.trim();
    let inner = rest
        .strip_prefix('"')
        .ok_or_else(|| format!("line {}: expected quoted string, got '{}'", lineno, rest))?;
    let inner = inner
        .rsplit_once('"')
        .map(|(s, _)| s)
        .ok_or_else(|| format!("line {}: unterminated quoted string", lineno))?;
    Ok(inner.to_string())
}

fn parse_msf(s: &str, lineno: usize) -> Result<(u8, u8, u8), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err(format!(
            "line {}: expected MM:SS:FF, got '{}'",
            lineno, s
        ));
    }
    let mm: u8 = parts[0]
        .parse()
        .map_err(|e| format!("line {}: invalid minutes '{}': {}", lineno, parts[0], e))?;
    let ss: u8 = parts[1]
        .parse()
        .map_err(|e| format!("line {}: invalid seconds '{}': {}", lineno, parts[1], e))?;
    let ff: u8 = parts[2]
        .parse()
        .map_err(|e| format!("line {}: invalid frames '{}': {}", lineno, parts[2], e))?;
    if ss >= 60 || ff >= 75 {
        return Err(format!(
            "line {}: out-of-range time '{}' (seconds<60, frames<75)",
            lineno, s
        ));
    }
    Ok((mm, ss, ff))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_track_audio() {
        let input = r#"FILE "track.wav" WAVE
  TRACK 01 AUDIO
    INDEX 01 00:00:00
"#;
        let sheet = parse(input).unwrap();
        assert_eq!(sheet.file, "track.wav");
        assert_eq!(sheet.tracks.len(), 1);
        let t = &sheet.tracks[0];
        assert_eq!(t.number, 1);
        assert_eq!(t.mode, "AUDIO");
        assert_eq!(t.indices, vec![(1u8, 0u8, 0u8, 0u8)]);
        assert!(t.pregap.is_none());
        assert!(t.postgap.is_none());
    }

    #[test]
    fn parse_multi_track_with_indices_and_gaps() {
        let input = r#"TITLE "My Album"
PERFORMER "My Artist"
FILE "disc.wav" WAVE
  TRACK 01 AUDIO
    TITLE "First"
    INDEX 00 00:00:00
    INDEX 01 00:00:02
  TRACK 02 MODE1/2352
    PREGAP 00:02:00
    INDEX 01 05:00:00
  TRACK 03 AUDIO
    POSTGAP 00:00:10
    INDEX 01 08:42:12
"#;
        let sheet = parse(input).unwrap();
        assert_eq!(sheet.title, "My Album");
        assert_eq!(sheet.performer, "My Artist");
        assert_eq!(sheet.file, "disc.wav");
        assert_eq!(sheet.tracks.len(), 3);

        assert_eq!(sheet.tracks[0].indices.len(), 2);
        assert_eq!(sheet.tracks[0].indices[0], (0, 0, 0, 0));
        assert_eq!(sheet.tracks[0].indices[1], (1, 0, 0, 2));

        assert_eq!(sheet.tracks[1].mode, "MODE1/2352");
        assert_eq!(sheet.tracks[1].pregap, Some((0, 2, 0)));
        assert_eq!(sheet.tracks[1].indices[0], (1, 5, 0, 0));

        assert_eq!(sheet.tracks[2].postgap, Some((0, 0, 10)));
        assert_eq!(sheet.tracks[2].indices[0], (1, 8, 42, 12));
    }

    #[test]
    fn parse_malformed_msf_rejects() {
        let input = r#"FILE "track.wav" WAVE
  TRACK 01 AUDIO
    INDEX 01 99:99:99
"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_out_of_range_frames_rejects() {
        let input = r#"FILE "track.wav" WAVE
  TRACK 01 AUDIO
    INDEX 01 00:00:80
"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_unknown_mode_rejects() {
        let input = r#"FILE "track.bin" BINARY
  TRACK 01 MODE5/9999
    INDEX 01 00:00:00
"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_missing_index_01_rejects() {
        let input = r#"FILE "track.wav" WAVE
  TRACK 01 AUDIO
    INDEX 00 00:00:00
"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_missing_file_rejects() {
        let input = "TRACK 01 AUDIO\n  INDEX 01 00:00:00\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_blank_lines_and_whitespace_handled() {
        let input = "

FILE \"a.wav\" WAVE

  TRACK 01 AUDIO
    INDEX 01 00:00:00

";
        let sheet = parse(input).unwrap();
        assert_eq!(sheet.tracks.len(), 1);
        assert_eq!(sheet.file, "a.wav");
    }

    #[test]
    fn parse_all_supported_modes_accepted() {
        for mode in &[
            "AUDIO",
            "MODE1/2048",
            "MODE1/2352",
            "MODE2/2336",
            "MODE2/2352",
        ] {
            let input = format!(
                "FILE \"x.wav\" WAVE\n  TRACK 01 {}\n    INDEX 01 00:00:00\n",
                mode
            );
            let sheet = parse(&input)
                .unwrap_or_else(|e| panic!("mode {} failed: {}", mode, e));
            assert_eq!(sheet.tracks[0].mode, *mode);
        }
    }
}