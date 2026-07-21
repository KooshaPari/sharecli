//! FR-007 — thermal gate ordering on `sharecli proc --tree --watch` refresh surfaces
//! FR: FR-007
//!
//! AC-007.23 `proc --tree --watch` text and NDJSON preserve gate → host_watch ordering
//! on every refresh (parity with flat watch AC-007.22 and one-shot tree text AC-007.20)

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

const GATE_MARKER: &str = "=== Thermal Gate (FR-011) ===";
const WATCH_MARKER: &str = "=== Host Resource Watch ===";
const TREE_HEADER: &str = "=== Agent process tree (proc scan) ===";

fn assert_gate_before_watch(segment: &str, context: &str) {
    let gate_pos = segment
        .find(GATE_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include gate section; got: {segment}"));
    let watch_pos = segment
        .find(WATCH_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include host watch section; got: {segment}"));
    assert!(
        gate_pos < watch_pos,
        "{context} MUST print gate before host watch; got: {segment}"
    );
}

fn drain_stdout_after_watch(child: &mut Child, dwell: Duration) -> String {
    let stdout = child.stdout.take().expect("piped stdout");
    let reader = thread::spawn(move || {
        let mut buf = String::new();
        let mut out = stdout;
        let _ = out.read_to_string(&mut buf);
        buf
    });
    thread::sleep(dwell);
    let _ = child.kill();
    let _ = child.wait();
    reader.join().expect("stdout drain thread")
}

/// FR-007 / AC-007.23 — tree watch text re-renders gate before host watch on each refresh.
#[test]
#[serial_test::serial]
fn fr007_proc_tree_watch_text_gate_ordering() {
    let mut child = bin()
        .args(["proc", "--tree", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --tree --watch 1");

    let stdout = drain_stdout_after_watch(&mut child, Duration::from_millis(30_000));

    let frame_count = stdout.matches(TREE_HEADER).count();
    assert!(
        frame_count >= 2,
        "tree watch MUST re-render at least twice in ~30s; got {frame_count} frames in: {stdout}"
    );

    for (idx, segment) in stdout.split(TREE_HEADER).skip(1).enumerate() {
        if segment.contains(WATCH_MARKER) {
            assert_gate_before_watch(segment, &format!("tree watch text frame {}", idx + 1));
        }
    }
    assert!(
        stdout.contains(WATCH_MARKER),
        "tree watch text MUST eventually render host watch footer; got: {stdout}"
    );
}

/// FR-007 / AC-007.23 — tree watch NDJSON lines embed gate before host_watch on every snapshot.
#[test]
#[serial_test::serial]
fn fr007_proc_tree_watch_ndjson_gate_ordering() {
    let mut child = bin()
        .args(["proc", "--tree", "--json", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --tree --json --watch 1");

    let stdout = drain_stdout_after_watch(&mut child, Duration::from_millis(30_000));

    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "tree watch --json MUST emit at least two NDJSON lines in ~30s; got: {stdout}"
    );
    for (idx, line) in lines.iter().enumerate() {
        assert_ndjson_gate_before_host_watch(line, &format!("tree NDJSON line {}", idx + 1));
    }
}

fn assert_ndjson_gate_before_host_watch(line: &str, context: &str) {
    let v: serde_json::Value =
        serde_json::from_str(line.trim()).expect("tree watch NDJSON line MUST be valid JSON");
    assert!(v.get("ts").is_some(), "{context} MUST include ts");
    assert!(v.get("gate").is_some(), "{context} MUST include gate (AC-007.23)");
    assert!(
        v.get("host_watch").is_some(),
        "{context} MUST include host_watch (AC-007.23)"
    );
    let gate_pos = line.find("\"gate\"").expect("gate key in raw JSON");
    let host_pos = line.find("\"host_watch\"").expect("host_watch key in raw JSON");
    assert!(
        gate_pos < host_pos,
        "{context} MUST serialize gate before host_watch (AC-007.23); got: {line}"
    );
}
