//! FR-006 — `sharecli proc --watch` live refresh
//! FR: FR-006
//!
//! AC-006.15 `sharecli proc --watch N` re-renders agent inventory until Ctrl-C

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

/// FR-006 / AC-006.15 — proc help documents live watch mode.
#[test]
fn fr006_proc_help_documents_watch() {
    let out = bin().args(["proc", "--help"]).output().expect("spawn sharecli proc --help");
    assert!(out.status.success(), "proc --help should exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("--watch"), "proc --help MUST document --watch; got: {s}");
}

/// FR-006 / AC-006.15 — watch mode prints refresh banner and re-renders inventory.
#[test]
fn fr006_proc_watch_renders_twice_before_exit() {
    let mut child = bin()
        .args(["proc", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --watch 1");

    thread::sleep(Duration::from_millis(2_500));

    let _ = child.kill();

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    let _ = child.wait();

    assert!(
        stdout.contains("Host agents (proc scan)"),
        "watch MUST render inventory header; got: {stdout}"
    );
    assert!(stdout.contains("[watch]"), "watch MUST print refresh footer; got: {stdout}");
    assert!(
        stdout.matches("Host agents (proc scan)").count() >= 2,
        "watch MUST re-render at least twice in ~2.5s; got {} headers",
        stdout.matches("Host agents (proc scan)").count()
    );
}

/// FR-006 / AC-006.15 — watch honors --json (valid JSON each refresh).
#[test]
fn fr006_proc_watch_json_emits_valid_payload() {
    let mut child = bin()
        .args(["proc", "--json", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --json --watch 1");

    thread::sleep(Duration::from_millis(1_500));
    let _ = child.kill();

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    let _ = child.wait();

    assert!(stdout.contains("\"agents\""), "watch --json MUST emit agents array; got: {stdout}");
    assert!(stdout.contains("[watch]"), "watch --json MUST still print footer; got: {stdout}");
    // First JSON object in output must parse.
    let json_start = stdout.find('{').expect("JSON payload");
    let json_end =
        stdout[json_start..].find("\n[watch]").map(|i| json_start + i).unwrap_or(stdout.len());
    let _: serde_json::Value =
        serde_json::from_str(stdout[json_start..json_end].trim()).expect("valid proc JSON");
}
