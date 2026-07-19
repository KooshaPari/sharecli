//! C10 L101 — designed error / failure states with recovery actions.
//! FR: FR-003

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// FR-003 / C10 L101 — dashboard ships disconnect error panel with recovery CTA.
#[test]
fn c10_l101_dashboard_disconnect_error_markup() {
    let html = fs::read_to_string(repo_root().join("src/dashboard.html")).expect("read dashboard.html");
    for needle in [
        "error-state",
        "data-error-kind=\"disconnect\"",
        "renderDisconnectError",
        "Dashboard disconnected",
        "sharecli serve",
        "error-retry",
        "Retry now",
    ] {
        assert!(html.contains(needle), "dashboard MUST include {needle} for L101");
    }
}

/// FR-003 / C10 L101 — error-state contract documented.
#[test]
fn c10_l101_error_states_doc_present() {
    let doc = fs::read_to_string(repo_root().join("docs/visual/error-states.md"))
        .expect("read error-states.md");
    assert!(
        doc.contains("data-error-kind=\"disconnect\""),
        "error-states.md must document disconnect kind"
    );
    assert!(
        doc.contains("Retry now"),
        "error-states.md must document retry CTA"
    );
    assert!(
        doc.contains("sharecli serve --bind"),
        "error-states.md must document serve recovery command"
    );
}
