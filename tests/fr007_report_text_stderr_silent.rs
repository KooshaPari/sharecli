//! FR-007 — `sharecli report` text stderr silence + host watch parity
//! FR: FR-007
//!
//! AC-007.39 `report` (text, one-shot + `--watch`) MUST NOT print gate or host_watch text
//! companions on stderr; gate/host_watch stay on stdout only after report body (parity with
//! AC-007.38 ps --all / AC-007.37 health/pool).

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

fn assert_stderr_silent(stderr: &[u8], context: &str) {
    assert!(
        stderr.is_empty(),
        "{context} MUST NOT print gate/host_watch companions on stderr (AC-007.39); stderr: {:?}",
        String::from_utf8_lossy(stderr)
    );
}

fn assert_stderr_no_companion_markers(stderr: &[u8], context: &str) {
    let s = String::from_utf8_lossy(stderr);
    assert!(
        !s.contains(GATE_MARKER),
        "{context} stderr MUST NOT include gate companion text (AC-007.39); stderr: {s}"
    );
    assert!(
        !s.contains(WATCH_MARKER),
        "{context} stderr MUST NOT include host watch companion text (AC-007.39); stderr: {s}"
    );
}

fn assert_text_body_has_gate_and_host_watch(stdout: &[u8], context: &str) {
    let s = String::from_utf8_lossy(stdout);
    assert!(
        s.contains(REPORT_HEADER),
        "{context} text body MUST include report header (AC-007.39); got: {s}"
    );
    assert!(
        s.contains(GATE_MARKER),
        "{context} text body MUST include gate section (AC-007.39); got: {s}"
    );
    assert!(
        s.contains(WATCH_MARKER),
        "{context} text body MUST include host watch section (AC-007.39); got: {s}"
    );
    let report_pos = s.find(REPORT_HEADER).expect("report header");
    let gate_pos = s.find(GATE_MARKER).expect("gate section");
    let watch_pos = s.find(WATCH_MARKER).expect("host watch section");
    assert!(
        report_pos < gate_pos,
        "{context} report body MUST precede gate section (AC-007.39); got: {s}"
    );
    assert!(
        gate_pos < watch_pos,
        "{context} gate section MUST precede host watch footer (AC-007.39); got: {s}"
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

fn assert_gate_before_watch(segment: &str, context: &str) {
    let gate_pos = segment
        .find(GATE_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include gate section; got: {segment}"));
    let watch_pos = segment
        .find(WATCH_MARKER)
        .unwrap_or_else(|| panic!("{context} MUST include host watch section; got: {segment}"));
    assert!(
        gate_pos < watch_pos,
        "{context} gate section MUST precede host watch footer (AC-007.39); got: {segment}"
    );
}

fn assert_text_watch_stdout(stdout: &str, context: &str) {
    let frame_count = stdout.matches(REPORT_HEADER).count();
    assert!(
        frame_count >= 2,
        "{context} MUST re-render at least twice in dwell window; got {frame_count} frames in: {stdout}"
    );
    assert!(
        stdout.contains(GATE_MARKER),
        "{context} stdout MUST include gate section (AC-007.39); got: {stdout}"
    );
    assert!(
        stdout.contains(WATCH_MARKER),
        "{context} stdout MUST include host watch section (AC-007.39); got: {stdout}"
    );
    assert!(
        stdout.contains("[watch]"),
        "{context} stdout MUST include [watch] footer (AC-007.39); got: {stdout}"
    );
    for (idx, segment) in stdout.split(REPORT_HEADER).skip(1).enumerate() {
        if segment.contains(WATCH_MARKER) {
            assert_gate_before_watch(segment, &format!("{context} frame {}", idx + 1));
        }
    }
}

/// FR-007 / AC-007.39 — one-shot report text keeps stderr silent; gate/host_watch on stdout only.
#[test]
#[serial_test::serial]
fn fr007_report_text_stderr_silent() {
    let out = bin()
        .args(["report", "--format", "text"])
        .output()
        .expect("spawn sharecli report --format text");
    assert!(out.status.success(), "report text MUST exit 0; stderr: {:?}", out.stderr);
    assert_stderr_silent(&out.stderr, "report");
    assert_stderr_no_companion_markers(&out.stderr, "report");
    assert_text_body_has_gate_and_host_watch(&out.stdout, "report");
}

/// FR-007 / AC-007.39 — report --watch text keeps stderr silent across refresh cycles.
#[test]
#[serial_test::serial]
fn fr007_report_watch_text_stderr_silent() {
    let mut child = bin()
        .args(["report", "--format", "text", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sharecli report --format text --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(12_000));

    assert_stderr_silent(stderr.as_bytes(), "report --watch");
    assert_stderr_no_companion_markers(stderr.as_bytes(), "report --watch");
    assert_text_watch_stdout(&stdout, "report --watch");
}
