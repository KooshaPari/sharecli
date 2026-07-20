//! Agent process pattern detection (FR-006).
//!
//! Discovers known coding agents by process name / cmdline tokens.
//! Detection is observation-only — sharecli MUST NOT wrap or replace vendor
//! agent binaries as the primary integration path.

/// Known agent family ids returned by [`match_known_agent`].
pub const KNOWN_AGENT_FAMILIES: &[&str] =
    &["claude", "codex", "gemini", "cursor-agent", "aider", "goose", "forge"];

/// Families whose short `comm` names collide with non-agent tooling — require cmdline fingerprints.
const AMBIGUOUS_FAMILIES: &[&str] = &["forge", "goose", "gemini"];

/// Cmdline substrings that fingerprint each family (AC-006.11).
const CMDLINE_FINGERPRINTS: &[(&str, &[&str])] = &[
    ("claude", &["claude", "claude-code", ".claude"]),
    ("codex", &["codex", "openai-codex"]),
    ("gemini", &["gemini", "gemini-cli", "google-gemini"]),
    ("cursor-agent", &["cursor-agent", ".cursor"]),
    ("aider", &["aider"]),
    ("goose", &["goose", "block-goose", "goose-agent"]),
    ("forge", &["forge", ".forge", "forge conversation"]),
];

/// Match a process `comm` (short name) and optional cmdline tokens against the
/// known-agent pattern registry.
///
/// Returns the canonical family id when a pattern matches; `None` otherwise.
/// Matching never implies wrapping the agent binary.
pub fn match_known_agent(comm: &str, cmdline: &[impl AsRef<str>]) -> Option<&'static str> {
    let comm_l = comm.to_ascii_lowercase();
    if let Some(family) = match_token(&comm_l) {
        if family_allowed(family, cmdline) {
            return Some(family);
        }
    }
    for arg in cmdline {
        let t = arg.as_ref().to_ascii_lowercase();
        let base = t.rsplit('/').next().unwrap_or(&t);
        let base = base.rsplit('\\').next().unwrap_or(base);
        if let Some(family) = match_token(base) {
            if family_allowed(family, cmdline) {
                return Some(family);
            }
        }
        if let Some(family) = match_token(&t) {
            if family_allowed(family, cmdline) {
                return Some(family);
            }
        }
    }
    None
}

fn family_allowed(family: &'static str, cmdline: &[impl AsRef<str>]) -> bool {
    if AMBIGUOUS_FAMILIES.contains(&family) {
        return cmdline_has_fingerprint(family, cmdline);
    }
    true
}

fn cmdline_has_fingerprint(family: &str, cmdline: &[impl AsRef<str>]) -> bool {
    let Some(markers) = CMDLINE_FINGERPRINTS
        .iter()
        .find(|(f, _)| *f == family)
        .map(|(_, m)| *m)
    else {
        return false;
    };
    for arg in cmdline {
        let t = arg.as_ref().to_ascii_lowercase();
        if markers.iter().any(|marker| t.contains(marker)) {
            return true;
        }
    }
    false
}

fn match_token(token: &str) -> Option<&'static str> {
    if token == "claude" || token.starts_with("claude-") || token.contains("claude-code") {
        return Some("claude");
    }
    if token == "codex" || token.starts_with("codex-") {
        return Some("codex");
    }
    if token == "gemini" || token.starts_with("gemini-") {
        return Some("gemini");
    }
    if token == "cursor-agent" || token == "cursor-agent.exe" {
        return Some("cursor-agent");
    }
    if token == "aider" || token.starts_with("aider-") {
        return Some("aider");
    }
    if token == "goose" || token.starts_with("goose-") {
        return Some("goose");
    }
    if token == "forge" || token.starts_with("forge-") {
        return Some("forge");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_claude_comm() {
        assert_eq!(match_known_agent("claude", &[] as &[&str]), Some("claude"));
    }

    #[test]
    fn detects_path_cmdline() {
        assert_eq!(
            match_known_agent("node", &["/usr/local/bin/claude"]),
            Some("claude")
        );
    }

    #[test]
    fn unknown_is_none() {
        assert_eq!(match_known_agent("bash", &["-c", "echo hi"]), None);
    }

    #[test]
    fn ambiguous_forge_without_fingerprint_is_none() {
        assert_eq!(
            match_known_agent("forge", &["build", "release"]),
            None,
            "non-agent forge tooling MUST NOT match without fingerprint"
        );
    }

    #[test]
    fn ambiguous_forge_with_fingerprint_matches() {
        assert_eq!(
            match_known_agent("forge", &["forge", "conversation", "list"]),
            Some("forge")
        );
    }

    #[test]
    fn ambiguous_gemini_requires_fingerprint() {
        assert_eq!(match_known_agent("gemini", &[] as &[&str]), None);
        assert_eq!(
            match_known_agent("gemini", &["gemini-cli", "chat"]),
            Some("gemini")
        );
    }
}
