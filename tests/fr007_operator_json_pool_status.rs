//! FR-007 — operator CLI JSON embedded pool + status siblings (AC-007.77)
//! FR: FR-007
//!
//! `health --json`, `pool --json`, `status --json`, `ps --all --json`, and `proc --json`
//! embed operator `pool` / `status` siblings after `gate` → `host_watch` (parity with
//! `report --format json` AC-007.73). `pool --json` adds nested `status` only (top-level
//! fields already are the pool panel); `status --json` adds nested `pool` only.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

const GATE_MARKER: &str = "=== Thermal Gate (FR-011) ===";
const WATCH_MARKER: &str = "=== Host Resource Watch ===";

const HOST_WATCH_KEYS: [&str; 5] =
    ["fd_count", "net_rx_bytes", "net_tx_bytes", "mem_rss_bytes", "load_1m"];

fn assert_pool_object(pool: &serde_json::Value, context: &str) {
    assert!(
        pool.get("node_total").is_some() && pool.get("healthy").is_some(),
        "{context} pool MUST include capacity fields (AC-007.77); got: {pool}"
    );
}

fn assert_status_object(status: &serde_json::Value, context: &str) {
    assert!(
        status.get("total_processes").is_some()
            && status.get("scanned").is_some()
            && status.get("watched").is_some(),
        "{context} status MUST include proc-scan fields (AC-007.77); got: {status}"
    );
}

fn assert_gate_host_watch(raw: &str, v: &serde_json::Value, context: &str) {
    let gate = v.get("gate").expect("{context} MUST include gate (AC-007.77)");
    let host = v
        .get("host_watch")
        .expect("{context} MUST include host_watch (AC-007.77)");
    assert!(
        gate.get("gate_decision").is_some(),
        "gate MUST include gate_decision (AC-007.77)"
    );
    for key in HOST_WATCH_KEYS {
        assert!(
            host.get(key).is_some(),
            "host_watch MUST include {key} (AC-007.77); got: {host}"
        );
    }
    let gate_pos = raw.find("\"gate\"").expect("gate key in raw JSON");
    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in raw JSON");
    assert!(
        gate_pos < host_pos,
        "{context} MUST serialize gate before host_watch (AC-007.77); got: {raw}"
    );
}

fn assert_json_gate_host_watch_pool_status_order(raw: &str, context: &str) {
    let v: serde_json::Value =
        serde_json::from_str(raw.trim()).expect("{context} MUST emit valid JSON");
    assert_gate_host_watch(raw, &v, context);
    let pool = v.get("pool").expect("{context} MUST include pool (AC-007.77)");
    let status = v.get("status").expect("{context} MUST include status (AC-007.77)");
    assert_pool_object(pool, context);
    assert_status_object(status, context);

    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in raw JSON");
    let pool_pos = raw.find("\"pool\"").expect("pool key in raw JSON (AC-007.77)");
    let status_pos = raw.find("\"status\"").expect("status key in raw JSON (AC-007.77)");
    assert!(
        host_pos < pool_pos && pool_pos < status_pos,
        "{context} MUST serialize gate → host_watch → pool → status (AC-007.77); got: {raw}"
    );
}

fn assert_json_gate_host_watch_status_only(raw: &str, context: &str) {
    let v: serde_json::Value =
        serde_json::from_str(raw.trim()).expect("{context} MUST emit valid JSON");
    assert_gate_host_watch(raw, &v, context);
    assert!(
        v.get("node_total").is_some(),
        "{context} top-level MUST include pool panel fields (AC-007.77); got: {v}"
    );
    let status = v.get("status").expect("{context} MUST include nested status (AC-007.77)");
    assert_status_object(status, context);

    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in raw JSON");
    let status_pos = raw.rfind("\"status\"").expect("status key in raw JSON (AC-007.77)");
    assert!(
        host_pos < status_pos,
        "{context} MUST serialize host_watch before nested status (AC-007.77); got: {raw}"
    );
}

fn assert_json_gate_host_watch_pool_only(raw: &str, context: &str) {
    let v: serde_json::Value =
        serde_json::from_str(raw.trim()).expect("{context} MUST emit valid JSON");
    assert_gate_host_watch(raw, &v, context);
    assert!(
        v.get("total_processes").is_some(),
        "{context} top-level MUST include proc-scan fields (AC-007.77); got: {v}"
    );
    let pool = v.get("pool").expect("{context} MUST include nested pool (AC-007.77)");
    assert_pool_object(pool, context);

    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in raw JSON");
    let pool_pos = raw.rfind("\"pool\"").expect("pool key in raw JSON (AC-007.77)");
    assert!(
        host_pos < pool_pos,
        "{context} MUST serialize host_watch before nested pool (AC-007.77); got: {raw}"
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

fn assert_stderr_silent(stderr: &[u8], context: &str) {
    assert!(
        stderr.is_empty(),
        "{context} MUST NOT print companions on stderr (AC-007.77); stderr: {:?}",
        String::from_utf8_lossy(stderr)
    );
    let s = String::from_utf8_lossy(stderr);
    assert!(
        !s.contains(GATE_MARKER) && !s.contains(WATCH_MARKER),
        "{context} stderr MUST NOT include gate/host_watch companion text (AC-007.77)"
    );
}

macro_rules! one_shot_json_test {
    ($name:ident, $args:expr, $assert_fn:ident) => {
        #[test]
        #[serial_test::serial]
        fn $name() {
            let out = bin().args($args).output().expect(concat!("spawn ", stringify!($name)));
            assert!(
                out.status.success(),
                "{} MUST exit 0; stderr: {:?}",
                stringify!($name),
                out.stderr
            );
            assert_stderr_silent(&out.stderr, stringify!($name));
            $assert_fn(
                &String::from_utf8_lossy(&out.stdout),
                stringify!($name),
            );
        }
    };
}

one_shot_json_test!(
    fr007_health_json_pool_status_shape,
    ["health", "--json"],
    assert_json_gate_host_watch_pool_status_order
);
one_shot_json_test!(
    fr007_pool_json_status_sibling_shape,
    ["pool", "--json"],
    assert_json_gate_host_watch_status_only
);
one_shot_json_test!(
    fr007_status_json_pool_sibling_shape,
    ["status", "--json"],
    assert_json_gate_host_watch_pool_only
);
one_shot_json_test!(
    fr007_ps_all_json_pool_status_shape,
    ["ps", "--all", "--json"],
    assert_json_gate_host_watch_pool_status_order
);
one_shot_json_test!(
    fr007_proc_json_pool_status_shape,
    ["proc", "--json"],
    assert_json_gate_host_watch_pool_status_order
);

macro_rules! watch_ndjson_test {
    ($name:ident, $args:expr, $assert_fn:ident) => {
        #[test]
        #[serial_test::serial]
        fn $name() {
            let mut child = bin()
                .args($args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect(concat!("spawn ", stringify!($name)));

            let (stdout, _stderr) = drain_watch_pipes(&mut child, Duration::from_millis(12_000));
            let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            assert!(
                lines.len() >= 2,
                "{} MUST emit at least two NDJSON lines; got: {stdout}",
                stringify!($name)
            );
            for (idx, line) in lines.iter().enumerate() {
                $assert_fn(line, &format!("{} line {}", stringify!($name), idx + 1));
            }
        }
    };
}

watch_ndjson_test!(
    fr007_health_watch_ndjson_pool_status_ordering,
    ["health", "--json", "--watch", "1"],
    assert_json_gate_host_watch_pool_status_order
);
watch_ndjson_test!(
    fr007_pool_watch_ndjson_status_sibling_ordering,
    ["pool", "--json", "--watch", "1"],
    assert_json_gate_host_watch_status_only
);
watch_ndjson_test!(
    fr007_status_watch_ndjson_pool_sibling_ordering,
    ["status", "--json", "--watch", "1"],
    assert_json_gate_host_watch_pool_only
);
watch_ndjson_test!(
    fr007_ps_all_watch_ndjson_pool_status_ordering,
    ["ps", "--all", "--json", "--watch", "1"],
    assert_json_gate_host_watch_pool_status_order
);
watch_ndjson_test!(
    fr007_proc_watch_ndjson_pool_status_ordering,
    ["proc", "--json", "--watch", "1"],
    assert_json_gate_host_watch_pool_status_order
);
