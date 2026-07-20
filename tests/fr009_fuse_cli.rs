//! FR-009 — fuse CLI provenance inspect (operator surface)
//! FR: FR-009
//!
//! AC-009.11 CLI `fuse provenance <path>` reads write xattrs via read_provenance

use sharecli_fuse::{annotate_write_at, read_provenance};
use std::process::Command;
use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// FR-009 / AC-009.11 — `sharecli fuse provenance --json` mirrors read_provenance.
#[test]
fn fr009_cli_fuse_provenance_json() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tracked.txt");
    std::fs::write(&path, b"payload").expect("write");
    annotate_write_at(&path, "cli-session-7", 1_750_000_000).expect("stamp");

    let out = bin()
        .args(["fuse", "provenance"])
        .arg(&path)
        .arg("--json")
        .output()
        .expect("spawn fuse provenance");
    assert!(
        out.status.success(),
        "fuse provenance MUST exit 0; stderr={}",
        stderr(&out)
    );
    let body = stdout(&out);
    let v: serde_json::Value = serde_json::from_str(&body).expect("json provenance");
    assert_eq!(v["session_id"], "cli-session-7");
    assert_eq!(v["written_at_unix"], 1_750_000_000);
}

/// FR-009 / AC-009.11 — absent xattrs emit JSON null.
#[test]
fn fr009_cli_fuse_provenance_json_null() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("plain.txt");
    std::fs::write(&path, b"plain").expect("write");

    let out = bin()
        .args(["fuse", "provenance"])
        .arg(&path)
        .arg("--json")
        .output()
        .expect("spawn fuse provenance");
    assert!(out.status.success(), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "null");
}

/// FR-009 / AC-009.11 — missing path fails loudly.
#[test]
fn fr009_cli_fuse_provenance_missing_path() {
    let out = bin()
        .args(["fuse", "provenance", "/no/such/sharecli-fuse-file"])
        .output()
        .expect("spawn fuse provenance");
    assert!(!out.status.success(), "missing path MUST fail");
    assert!(
        stderr(&out).contains("does not exist"),
        "stderr={}",
        stderr(&out)
    );
}

/// FR-009 / AC-009.11 — library round-trip matches CLI JSON (no mount).
#[test]
fn fr009_cli_fuse_provenance_matches_read_provenance() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("lib.txt");
    std::fs::write(&path, b"x").expect("write");
    annotate_write_at(&path, "lib-session", 42).expect("stamp");
    let lib = read_provenance(&path).expect("read").expect("present");

    let out = bin()
        .args(["fuse", "provenance"])
        .arg(&path)
        .arg("--json")
        .output()
        .expect("spawn");
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("json");
    assert_eq!(v["session_id"], lib.session_id);
    assert_eq!(v["written_at_unix"], lib.written_at_unix);
}
