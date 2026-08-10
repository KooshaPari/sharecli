//! FR-007 — `sharecli report` text pool + proc-scan operator sections (AC-007.74)
//! FR: FR-007
//!
//! `report` (text, one-shot + `--watch`) prints pool + proc-scan operator lines on stdout
//! after gate → host_watch (parity with AC-007.39 / AC-007.73 JSON key order).

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

const GATE_MARKER: &str = "=== Thermal Gate (FR-011) ===";
const WATCH_MARKER: &str = "=== Host Resource Watch ===";
const REPORT_HEADER: &str = "=== Fleet Analytics Report ===";
const POOL_PREFIX: &str = "Pool node";
const PROC_PREFIX: &str = "Proc scan";

fn assert_stderr_silent(stderr: &[u8], context: &str) {
    // dhat (heap profiler) is enabled by `--all-features` and writes its
    // summary to stderr on process exit. Filter those out so the helper
    // is checking for operator companions leakage, not profiler noise.
    let binding = String::from_utf8_lossy(stderr).into_owned();
    let filtered: Vec<&str> = binding
        .lines()
        .filter(|l| !l.trim_start().starts_with("dhat:"))
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert!(
        filtered.is_empty(),
        "{context} MUST NOT print operator companions on stderr (AC-007.74); stderr: {:?}",
        filtered
    );
}

fn assert_text_operator_order(stdout: &str, context: &str) {
    assert!(
        stdout.contains(REPORT_HEADER),
        "{context} MUST include report header (AC-007.74); got: {stdout}"
    );
    assert!(
        stdout.contains(GATE_MARKER),
        "{context} MUST include gate section (AC-007.74); got: {stdout}"
    );
    assert!(
        stdout.contains(WATCH_MARKER),
        "{context} MUST include host watch section (AC-007.74); got: {stdout}"
    );
    assert!(
        stdout.contains(POOL_PREFIX),
        "{context} MUST include pool operator line (AC-007.74); got: {stdout}"
    );
    assert!(
        stdout.contains(PROC_PREFIX),
        "{context} MUST include proc-scan operator line (AC-007.74); got: {stdout}"
    );

    let report_pos = stdout.find(REPORT_HEADER).expect("report header");
    let gate_pos = stdout.find(GATE_MARKER).expect("gate section");
    let watch_pos = stdout.find(WATCH_MARKER).expect("host watch section");
    let pool_pos = stdout.find(POOL_PREFIX).expect("pool operator line");
    let proc_pos = stdout.find(PROC_PREFIX).expect("proc-scan operator line");

    assert!(
        report_pos < gate_pos
            && gate_pos < watch_pos
            && watch_pos < pool_pos
            && pool_pos < proc_pos,
        "{context} MUST serialize report → gate → host_watch → pool → proc-scan (AC-007.74); got: {stdout}"
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
        "{context} MUST serialize gate → host_watch → pool → proc-scan (AC-007.74); got: {segment}"
    );
}

fn assert_text_watch_stdout(stdout: &str, context: &str) {
    let frame_count = stdout.matches(REPORT_HEADER).count();
    assert!(
        frame_count >= 2,
        "{context} MUST re-render at least twice in dwell window; got {frame_count} frames in: {stdout}"
    );
    assert!(
        stdout.contains("[watch]"),
        "{context} stdout MUST include [watch] footer (AC-007.74); got: {stdout}"
    );
    for (idx, segment) in stdout.split(REPORT_HEADER).skip(1).enumerate() {
        if segment.contains(WATCH_MARKER) && segment.contains(POOL_PREFIX) {
            assert_frame_operator_order(segment, &format!("{context} frame {}", idx + 1));
        }
    }
}

/// FR-007 / AC-007.74 — one-shot report text prints pool + proc-scan after gate → host_watch.
#[test]
#[serial_test::serial]
fn fr007_report_text_pool_status_order() {
    let out = bin()
        .args(["report", "--format", "text"])
        .output()
        .expect("spawn sharecli report --format text");
    assert!(out.status.success(), "report text MUST exit 0; stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_stderr_silent(&out.stderr, "report");
    assert_text_operator_order(&stdout, "report");
}

/// FR-007 / AC-007.74 — report --watch text keeps pool/proc-scan on stdout across refresh cycles.
#[test]
#[serial_test::serial]
fn fr007_report_watch_text_pool_status_order() {
    let mut child = bin()
        .args(["report", "--format", "text", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli report --format text --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(12_000));

    assert_stderr_silent(stderr.as_bytes(), "report --watch");
    assert_text_watch_stdout(&stdout, "report --watch");
}
