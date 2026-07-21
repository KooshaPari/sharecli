//! Agent process pattern detection (FR-006).
//!
//! Discovers known coding agents by process name / cmdline tokens.
//! Detection is observation-only — sharecli MUST NOT wrap or replace vendor
//! agent binaries as the primary integration path.

/// Known agent family ids returned by [`match_known_agent`].
pub const KNOWN_AGENT_FAMILIES: &[&str] =
    &["claude", "codex", "gemini", "cursor-agent", "aider", "amp", "goose", "forge"];

/// Families whose short `comm` names collide with non-agent tooling — require cmdline fingerprints.
const AMBIGUOUS_FAMILIES: &[&str] = &["forge", "goose", "gemini"];

/// Cmdline substrings that fingerprint each family (AC-006.11, AC-006.20).
const CMDLINE_FINGERPRINTS: &[(&str, &[&str])] = &[
    ("claude", &["claude", "claude-code", ".claude"]),
    ("codex", &["openai-codex", "@openai/codex", "codex-cli", "/bin/codex"]),
    ("gemini", &["gemini", "gemini-cli", "google-gemini"]),
    ("cursor-agent", &["cursor-agent", ".cursor", "cursor agent", "@cursor/agent"]),
    ("aider", &["aider", "aider-chat", ".aider"]),
    ("amp", &["amp", "amp-code", "sourcegraph/amp", "@sourcegraph/amp", ".amp"]),
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
        let exact_comm = is_exact_comm_basename(&comm_l, family);
        if family_allowed(family, cmdline, exact_comm) {
            return Some(family);
        }
    }
    for arg in cmdline {
        let t = arg.as_ref().to_ascii_lowercase();
        let base = t.rsplit('/').next().unwrap_or(&t);
        let base = base.rsplit('\\').next().unwrap_or(base);
        if let Some(family) = match_token(base) {
            if family_allowed(family, cmdline, false) {
                return Some(family);
            }
        }
        if let Some(family) = match_token(&t) {
            if family_allowed(family, cmdline, false) {
                return Some(family);
            }
        }
    }
    match_fingerprint_only(cmdline)
}

/// Match argv fingerprints when comm/token heuristics did not resolve (AC-006.20).
fn match_fingerprint_only(cmdline: &[impl AsRef<str>]) -> Option<&'static str> {
    for (family, _) in CMDLINE_FINGERPRINTS {
        if cmdline_has_fingerprint(family, cmdline) {
            return Some(family);
        }
    }
    None
}

fn family_allowed(
    family: &'static str,
    cmdline: &[impl AsRef<str>],
    exact_comm_basename: bool,
) -> bool {
    if !AMBIGUOUS_FAMILIES.contains(&family) {
        return true;
    }
    // AC-006.1: exact comm basename with empty cmdline is a bare-name hit.
    if exact_comm_basename && cmdline.is_empty() {
        return true;
    }
    cmdline_has_fingerprint(family, cmdline)
}

fn is_exact_comm_basename(comm: &str, family: &str) -> bool {
    comm == family
}

fn cmdline_has_fingerprint(family: &str, cmdline: &[impl AsRef<str>]) -> bool {
    let Some(markers) = CMDLINE_FINGERPRINTS.iter().find(|(f, _)| *f == family).map(|(_, m)| *m)
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
    if token == "codex" || token == "openai-codex" {
        return Some("codex");
    }
    if token == "gemini" || token.starts_with("gemini-") {
        return Some("gemini");
    }
    if token == "cursor-agent" || token == "cursor-agent.exe" {
        return Some("cursor-agent");
    }
    if token == "aider" {
        return Some("aider");
    }
    if token == "amp" || token == "amp-cli" {
        return Some("amp");
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
        assert_eq!(match_known_agent("node", &["/usr/local/bin/claude"]), Some("claude"));
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
        assert_eq!(match_known_agent("forge", &["forge", "conversation", "list"]), Some("forge"));
    }

    #[test]
    fn ambiguous_gemini_bare_comm_matches() {
        assert_eq!(match_known_agent("gemini", &[] as &[&str]), Some("gemini"));
        assert_eq!(match_known_agent("gemini", &["gemini-cli", "chat"]), Some("gemini"));
    }
}
