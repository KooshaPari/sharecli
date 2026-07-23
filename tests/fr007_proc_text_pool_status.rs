//! FR-007 — `sharecli proc` text pool + proc-scan operator sections (AC-007.75)
//! FR: FR-007
//!
//! `proc` and `proc --tree` (text, one-shot + `--watch`) print pool + proc-scan operator lines on
//! stdout after gate → host_watch (parity with AC-007.74 report text path / AC-007.34 gate ordering).

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

const GATE_MARKER: &str = "=== Thermal Gate (FR-011) ===";
const WATCH_MARKER: &str = "=== Host Resource Watch ===";
const INVENTORY_HEADER: &str = "=== Host agents (proc scan) ===";
const TREE_HEADER: &str = "=== Agent process tree (proc scan) ===";
const POOL_PREFIX: &str = "Pool node";
const PROC_PREFIX: &str = "Proc scan";

fn assert_stderr_silent(stderr: &[u8], context: &str) {
    assert!(
        stderr.is_empty(),
        "{context} MUST NOT print operator companions on stderr (AC-007.75); stderr: {:?}",
        String::from_utf8_lossy(stderr)
    );
}

fn assert_text_operator_order(stdout: &str, body_header: &str, context: &str) {
    assert!(
        stdout.contains(body_header),
        "{context} MUST include proc body header (AC-007.75); got: {stdout}"
    );
    assert!(
        stdout.contains(GATE_MARKER),
        "{context} MUST include gate section (AC-007.75); got: {stdout}"
    );
    assert!(
        stdout.contains(WATCH_MARKER),
        "{context} MUST include host watch section (AC-007.75); got: {stdout}"
    );
    assert!(
        stdout.contains(POOL_PREFIX),
        "{context} MUST include pool operator line (AC-007.75); got: {stdout}"
    );
    assert!(
        stdout.contains(PROC_PREFIX),
        "{context} MUST include proc-scan operator line (AC-007.75); got: {stdout}"
    );

    let body_pos = stdout.find(body_header).expect("proc body header");
    let gate_pos = stdout.find(GATE_MARKER).expect("gate section");
    let watch_pos = stdout.find(WATCH_MARKER).expect("host watch section");
    let pool_pos = stdout.find(POOL_PREFIX).expect("pool operator line");
    let proc_pos = stdout.find(PROC_PREFIX).expect("proc-scan operator line");

    assert!(
        body_pos < gate_pos
            && gate_pos < watch_pos
            && watch_pos < pool_pos
            && pool_pos < proc_pos,
        "{context} MUST serialize body → gate → host_watch → pool → proc-scan (AC-007.75); got: {stdout}"
    );
}

fn drain_watch_pipes(child: &mut Child, dwell: Duration) -> (String, String) {
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = thread::spawn(move || {
        let mut buf = String::new();
        let mut out = stdout;
        let _ = out.read_to_string(&mut buf);
        buf
    });
    let stderr_reader = thread::spawn(move || {
        let mut buf = String::new();
        let mut err = stderr;
        let _ = err.read_to_string(&mut buf);
        buf
    });
    thread::sleep(dwell);
    let _ = child.kill();
    let _ = child.wait();
    let stdout = stdout_reader.join().expect("stdout drain thread");
    let stderr = stderr_reader.join().expect("stderr drain thread");
    (stdout, stderr)
}

fn assert_frame_operator_order(segment: &str, context: &str) {
    let gate_pos = segment
        .find(GATE_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include gate section; got: {segment}"));
    let watch_pos = segment
        .find(WATCH_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include host watch section; got: {segment}"));
    let pool_pos = segment
        .find(POOL_PREFIX)
        .unwrap_or_else(|| panic!("{context} MUST include pool operator line; got: {segment}"));
    let proc_pos = segment.find(PROC_PREFIX).unwrap_or_else(|| {
        panic!("{context} MUST include proc-scan operator line; got: {segment}")
    });
    assert!(
        gate_pos < watch_pos && watch_pos < pool_pos && pool_pos < proc_pos,
        "{context} MUST serialize gate → host_watch → pool → proc-scan (AC-007.75); got: {segment}"
    );
}

fn assert_text_watch_stdout(stdout: &str, frame_header: &str, context: &str) {
    let frame_count = stdout.matches(frame_header).count();
    assert!(
        frame_count >= 2,
        "{context} MUST re-render at least twice in dwell window; got {frame_count} frames in: {stdout}"
    );
    assert!(
        stdout.contains("[watch]"),
        "{context} stdout MUST include [watch] footer (AC-007.75); got: {stdout}"
    );
    for (idx, segment) in stdout.split(frame_header).skip(1).enumerate() {
        if segment.contains(WATCH_MARKER) && segment.contains(POOL_PREFIX) {
            assert_frame_operator_order(segment, &format!("{context} frame {}", idx + 1));
        }
    }
}

/// FR-007 / AC-007.75 — one-shot proc text prints pool + proc-scan after gate → host_watch.
#[test]
#[serial_test::serial]
fn fr007_proc_text_pool_status_order() {
    let out = bin().args(["proc"]).output().expect("spawn sharecli proc");
    assert!(out.status.success(), "proc MUST exit 0; stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_stderr_silent(&out.stderr, "proc");
    assert_text_operator_order(&stdout, INVENTORY_HEADER, "proc");
}

/// FR-007 / AC-007.75 — one-shot proc --tree text prints pool + proc-scan after gate → host_watch.
#[test]
#[serial_test::serial]
fn fr007_proc_tree_text_pool_status_order() {
    let out = bin().args(["proc", "--tree"]).output().expect("spawn sharecli proc --tree");
    assert!(out.status.success(), "proc --tree MUST exit 0; stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_stderr_silent(&out.stderr, "proc --tree");
    assert_text_operator_order(&stdout, TREE_HEADER, "proc --tree");
}

/// FR-007 / AC-007.75 — proc --watch text keeps pool/proc-scan on stdout across refresh cycles.
#[test]
#[serial_test::serial]
fn fr007_proc_watch_text_pool_status_order() {
    let mut child = bin()
        .args(["proc", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(12_000));

    assert_stderr_silent(stderr.as_bytes(), "proc --watch");
    assert_text_watch_stdout(&stdout, INVENTORY_HEADER, "proc --watch");
}

/// FR-007 / AC-007.75 — proc --tree --watch text keeps pool/proc-scan on stdout across refresh cycles.
#[test]
#[serial_test::serial]
fn fr007_proc_tree_watch_text_pool_status_order() {
    let mut child = bin()
        .args(["proc", "--tree", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli proc --tree --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(12_000));

    assert_stderr_silent(stderr.as_bytes(), "proc --tree --watch");
    assert_text_watch_stdout(&stdout, TREE_HEADER, "proc --tree --watch");
}
