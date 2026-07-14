//! Level A landmark checks for the embedded serve dashboard.
//! FR: NFR (C09 L81.1 / L81.5)

use std::fs;
use std::path::PathBuf;

fn dashboard_html() -> String {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("src/dashboard.html");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn dashboard_has_lang_and_landmarks() {
    let html = dashboard_html();
    assert!(html.contains(r#"<html lang="en">"#), "missing lang=en");
    assert!(html.contains("<main"), "missing <main> landmark");
    assert!(html.contains("<nav"), "missing <nav> landmark");
    assert!(
        html.contains(r#"aria-label="Dashboard status""#)
            || html.contains("aria-label='Dashboard status'"),
        "nav should have aria-label"
    );
    assert!(html.contains("id=\"main-content\""), "main needs skiplink target id");
}

#[test]
fn dashboard_announces_live_status() {
    let html = dashboard_html();
    assert!(html.contains("aria-live"), "connection/status updates should use aria-live");
}
