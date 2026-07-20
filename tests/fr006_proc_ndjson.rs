//! FR-006 — `sharecli proc --watch --json` NDJSON stream
//! FR: FR-006
//!
//! AC-006.18 watch + JSON emits one compact JSON object per line (NDJSON)

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

/// FR-006 / AC-006.18 — each watch refresh is a single parseable NDJSON line with `ts`.
#[test]
fn fr006_proc_watch_ndjson_one_line_per_refresh() {
    let mut child = bin()
        .args(["proc", "--json", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --json --watch 1");

    thread::sleep(Duration::from_millis(2_500));
    let _ = child.kill();

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    let _ = child.wait();

    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "NDJSON watch MUST emit at least two lines in ~2.5s; got {} line(s): {stdout}",
        lines.len()
    );
    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line).expect("each NDJSON line MUST parse");
        assert!(v.get("ts").and_then(|t| t.as_u64()).is_some(), "line MUST include ts");
        assert!(v.get("agents").and_then(|a| a.as_array()).is_some(), "line MUST include agents");
    }
}

/// FR-006 / AC-006.18 — NDJSON stdout stays pipe-clean (no watch footer, no ANSI clear).
#[test]
fn fr006_proc_watch_ndjson_stdout_is_pipe_clean() {
    let mut child = bin()
        .args(["proc", "--json", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --json --watch 1");

    thread::sleep(Duration::from_millis(1_500));
    let _ = child.kill();

    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }
    let _ = child.wait();

    assert!(
        !stdout.contains("[watch]"),
        "NDJSON stdout MUST NOT contain watch footer; got: {stdout}"
    );
    assert!(!stdout.contains("\x1b[2J"), "NDJSON stdout MUST NOT contain terminal clear sequences");
    assert!(
        stderr.contains("[watch]"),
        "watch footer MUST appear on stderr in NDJSON mode; stderr: {stderr}"
    );
}

/// FR-006 / AC-006.18 — one-shot proc --json remains pretty-printed (non-NDJSON).
#[test]
fn fr006_proc_json_snapshot_not_ndjson() {
    let out = bin().args(["proc", "--json"]).output().expect("spawn sharecli proc --json");
    assert!(out.status.success(), "proc --json should exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\n'), "one-shot --json MUST remain multi-line pretty JSON; got: {s}");
    assert!(!s.contains("\"ts\""), "one-shot --json MUST NOT inject ts field; got: {s}");
}
