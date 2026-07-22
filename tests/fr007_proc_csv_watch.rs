//! FR-007 — `sharecli proc --csv --watch` / `--tree --csv --watch`
//! FR: FR-007
//!
//! AC-007.88 CSV watch emits inventory + gate → host_watch → pool → status companions
//! each tick on stdout; stderr silent on success.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

const FRAME_MARKER: &str = "# sharecli-proc-watch-frame";
const FLAT_CSV_HEADER: &str = "pid,family,comm,state,mem_rss_bytes,mem_rss,fd_count";
const TREE_CSV_HEADER: &str = "root_index,depth,pid,ppid,family,comm,state,mem_rss_bytes,mem_rss,fd_count";
const GATE_CSV_HEADER: &str =
    "record,thermal_pressure,detected_agents,agent_total_rss_bytes,agent_contention,gate_decision";
const HOST_CSV_HEADER: &str = "record,fd_count,net_rx_bytes,net_tx_bytes,mem_rss_bytes,load_1m";
const POOL_CSV_HEADER: &str =
    "record,node_total,node_idle,bun_total,bun_idle,max_per_type,healthy";
const STATUS_CSV_HEADER: &str = "record,scanned,watched,total_processes,agent_rows";

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
    (
        stdout_reader.join().expect("stdout drain"),
        stderr_reader.join().expect("stderr drain"),
    )
}

fn assert_csv_envelope(frame: &str, body_header: &str, context: &str) {
    let body = frame
        .find(body_header)
        .unwrap_or_else(|| panic!("{context} MUST include inventory CSV header; got: {frame}"));
    let gate = frame
        .find(GATE_CSV_HEADER)
        .unwrap_or_else(|| panic!("{context} MUST include gate CSV companion; got: {frame}"));
    let host = frame
        .find(HOST_CSV_HEADER)
        .unwrap_or_else(|| panic!("{context} MUST include host_watch CSV companion; got: {frame}"));
    let pool = frame
        .find(POOL_CSV_HEADER)
        .unwrap_or_else(|| panic!("{context} MUST include pool CSV companion; got: {frame}"));
    let status = frame
        .find(STATUS_CSV_HEADER)
        .unwrap_or_else(|| panic!("{context} MUST include status CSV companion; got: {frame}"));
    assert!(
        body < gate && gate < host && host < pool && pool < status,
        "{context} MUST order body → gate → host_watch → pool → status (AC-007.88); got: {frame}"
    );
}

/// FR-007 / AC-007.88 — flat proc --csv --watch stderr silent; multi-frame envelope.
#[test]
#[serial_test::serial]
fn fr007_proc_csv_watch_stderr_silent_and_envelope() {
    let mut child = bin()
        .args(["proc", "--csv", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn proc --csv --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(10_000));

    assert!(
        stderr.is_empty(),
        "proc --csv --watch MUST keep stderr silent (AC-007.88); stderr: {stderr:?}"
    );
    assert!(
        !stdout.contains("\x1b[2J"),
        "proc --csv --watch MUST NOT emit ANSI clear (pipe-safe); got: {stdout}"
    );
    let complete_frames: Vec<&str> = stdout
        .split(FRAME_MARKER)
        .skip(1)
        .filter(|frame| frame.contains(FLAT_CSV_HEADER))
        .collect();
    assert!(
        complete_frames.len() >= 2,
        "proc --csv --watch MUST emit >=2 complete frames; got {} in: {stdout}",
        complete_frames.len()
    );
    assert!(
        stdout.contains("[watch]"),
        "proc --csv --watch MUST include [watch] footer comment; got: {stdout}"
    );
    for (idx, frame) in complete_frames.iter().enumerate() {
        assert_csv_envelope(frame, FLAT_CSV_HEADER, &format!("flat csv watch frame {}", idx + 1));
    }
}

/// FR-007 / AC-007.88 — proc --tree --csv --watch same stderr/envelope contract.
#[test]
#[serial_test::serial]
fn fr007_proc_tree_csv_watch_stderr_silent_and_envelope() {
    let mut child = bin()
        .args(["proc", "--tree", "--csv", "--watch", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn proc --tree --csv --watch 1");

    let (stdout, stderr) = drain_watch_pipes(&mut child, Duration::from_millis(10_000));

    assert!(
        stderr.is_empty(),
        "proc --tree --csv --watch MUST keep stderr silent (AC-007.88); stderr: {stderr:?}"
    );
    let complete_frames: Vec<&str> = stdout
        .split(FRAME_MARKER)
        .skip(1)
        .filter(|frame| frame.contains(TREE_CSV_HEADER))
        .collect();
    assert!(
        complete_frames.len() >= 2,
        "proc --tree --csv --watch MUST emit >=2 complete frames; got {} in: {stdout}",
        complete_frames.len()
    );
    for (idx, frame) in complete_frames.iter().enumerate() {
        assert_csv_envelope(frame, TREE_CSV_HEADER, &format!("tree csv watch frame {}", idx + 1));
    }
}

/// FR-007 / AC-007.88 — proc --csv --json --watch remains rejected.
#[test]
fn fr007_proc_csv_json_watch_still_rejected() {
    let out = bin()
        .args(["proc", "--csv", "--json", "--watch", "1"])
        .output()
        .expect("spawn proc --csv --json --watch");
    assert!(
        !out.status.success(),
        "proc --csv --json --watch MUST fail (AC-007.88)"
    );
}
