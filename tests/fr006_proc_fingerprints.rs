//! FR-006 — proc_scan cmdline fingerprint accuracy
//! FR: FR-006
//!
//! AC-006.11 ambiguous comm names require cmdline fingerprints (false-positive guard)

use sharecli_fleet::match_known_agent;

/// FR-006 / AC-006.11 — generic forge build tooling is not an agent.
#[test]
fn fr006_forge_build_tool_not_agent() {
    assert_eq!(match_known_agent("forge", &["build", "--release"]), None);
}

/// FR-006 / AC-006.11 — forge agent conversation argv matches.
#[test]
fn fr006_forge_agent_conversation_matches() {
    assert_eq!(
        match_known_agent("forge", &["forge", "conversation", "list"]),
        Some("forge")
    );
}

/// FR-006 / AC-006.11 — bare gemini comm without fingerprint is rejected.
#[test]
fn fr006_bare_gemini_comm_rejected() {
    assert_eq!(match_known_agent("gemini", &[] as &[&str]), None);
}

/// FR-006 / AC-006.11 — gemini-cli fingerprint accepted.
#[test]
fn fr006_gemini_cli_fingerprint_accepted() {
    assert_eq!(
        match_known_agent("node", &["/opt/bin/gemini-cli", "chat"]),
        Some("gemini")
    );
}
