//! FR-007 — operator envelope matrix parity regression suite (AC-007.84)
//! FR: FR-007
//!
//! Locks the full FR-007 operator envelope across proc/report/health/pool/status/ps --all
//! (text/JSON/CSV one-shot), IPC, WS decode, dashboard, tray, and thermal TUI companion
//! markers. No long `--watch` dwell cycles — those stay in per-AC integration files.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use sharecli_fleet::{PoolOperatorPanel, StatusOperatorPanel};
use sharecli_thermal_tui::{pool_panel_lines, status_panel_lines, HELP_OVERLAY_HINT};
use sharecli_tray_linux::ipc::{GateStatusSnapshot, HostResourceWatchJson, PoolSnapshot, StatusSnapshot};
use sharecli_tray_linux::operator_display as linux_display;
use sharecli_tray_windows::operator_display as win_display;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

fn manifest_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

const GATE_MARKER: &str = "=== Thermal Gate (FR-011) ===";
const WATCH_MARKER: &str = "=== Host Resource Watch ===";
const POOL_PREFIX: &str = "Pool node";
const PROC_PREFIX: &str = "Proc scan";

const GATE_CSV_HEADER: &str =
    "record,thermal_pressure,detected_agents,agent_total_rss_bytes,agent_contention,gate_decision";
const HOST_CSV_HEADER: &str = "record,fd_count,net_rx_bytes,net_tx_bytes,mem_rss_bytes,load_1m";
const POOL_COMPANION_CSV_HEADER: &str =
    "record,node_total,node_idle,bun_total,bun_idle,max_per_type,healthy";
const STATUS_COMPANION_CSV_HEADER: &str = "record,scanned,watched,total_processes,agent_rows";

const HOST_WATCH_KEYS: [&str; 5] =
    ["fd_count", "net_rx_bytes", "net_tx_bytes", "mem_rss_bytes", "load_1m"];

struct TextCase {
    label: &'static str,
    args: &'static [&'static str],
    body_header: &'static str,
}

struct JsonCase {
    label: &'static str,
    args: &'static [&'static str],
    mode: JsonEnvelopeMode,
}

#[derive(Clone, Copy)]
enum JsonEnvelopeMode {
    FullPoolStatus,
    PoolTopStatusNested,
    StatusTopPoolNested,
}

struct CsvCase {
    label: &'static str,
    args: &'static [&'static str],
    body_header: &'static str,
}

fn assert_stderr_silent(stderr: &[u8], context: &str) {
    assert!(
        stderr.is_empty(),
        "{context} MUST keep stderr silent on success (AC-007.84); stderr: {:?}",
        String::from_utf8_lossy(stderr)
    );
}

fn assert_text_envelope(stdout: &str, body_header: &str, context: &str) {
    assert!(
        stdout.contains(body_header),
        "{context} MUST include command body (AC-007.84); got: {stdout}"
    );
    assert!(stdout.contains(GATE_MARKER), "{context} MUST include gate (AC-007.84)");
    assert!(stdout.contains(WATCH_MARKER), "{context} MUST include host_watch (AC-007.84)");
    assert!(stdout.contains(POOL_PREFIX), "{context} MUST include pool line (AC-007.84)");
    assert!(stdout.contains(PROC_PREFIX), "{context} MUST include proc-scan line (AC-007.84)");

    let body_pos = stdout.find(body_header).expect("body header");
    let gate_pos = stdout.find(GATE_MARKER).expect("gate");
    let watch_pos = stdout.find(WATCH_MARKER).expect("host_watch");
    let pool_pos = stdout.find(POOL_PREFIX).expect("pool");
    let proc_pos = stdout.find(PROC_PREFIX).expect("proc-scan");
    assert!(
        body_pos < gate_pos && gate_pos < watch_pos && watch_pos < pool_pos && pool_pos < proc_pos,
        "{context} MUST serialize body → gate → host_watch → pool → proc-scan (AC-007.84); got: {stdout}"
    );
}

fn assert_pool_object(pool: &serde_json::Value, context: &str) {
    assert!(
        pool.get("node_total").is_some() && pool.get("healthy").is_some(),
        "{context} pool MUST include capacity fields (AC-007.84); got: {pool}"
    );
}

fn assert_status_object(status: &serde_json::Value, context: &str) {
    assert!(
        status.get("total_processes").is_some()
            && status.get("scanned").is_some()
            && status.get("watched").is_some(),
        "{context} status MUST include proc-scan fields (AC-007.84); got: {status}"
    );
}

fn assert_json_gate_host_watch(raw: &str, v: &serde_json::Value, context: &str) {
    let gate = v.get("gate").expect("{context} MUST include gate (AC-007.84)");
    let host = v
        .get("host_watch")
        .expect("{context} MUST include host_watch (AC-007.84)");
    assert!(gate.get("gate_decision").is_some(), "gate MUST include gate_decision (AC-007.84)");
    for key in HOST_WATCH_KEYS {
        assert!(
            host.get(key).is_some(),
            "host_watch MUST include {key} (AC-007.84); got: {host}"
        );
    }
    let gate_pos = raw.find("\"gate\"").expect("gate key in raw JSON");
    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in raw JSON");
    assert!(
        gate_pos < host_pos,
        "{context} MUST serialize gate before host_watch (AC-007.84); got: {raw}"
    );
}

fn assert_json_envelope(raw: &str, mode: JsonEnvelopeMode, context: &str) {
    let v: serde_json::Value =
        serde_json::from_str(raw.trim()).expect("{context} MUST emit valid JSON (AC-007.84)");
    assert_json_gate_host_watch(raw, &v, context);

    match mode {
        JsonEnvelopeMode::FullPoolStatus => {
            let pool = v.get("pool").expect("{context} MUST include pool (AC-007.84)");
            let status = v.get("status").expect("{context} MUST include status (AC-007.84)");
            assert_pool_object(pool, context);
            assert_status_object(status, context);
            let host_pos = raw.find("\"host_watch\"").expect("host_watch key");
            let pool_pos = raw.find("\"pool\"").expect("pool key (AC-007.84)");
            let status_pos = raw.find("\"status\"").expect("status key (AC-007.84)");
            assert!(
                host_pos < pool_pos && pool_pos < status_pos,
                "{context} MUST serialize gate → host_watch → pool → status (AC-007.84); got: {raw}"
            );
        }
        JsonEnvelopeMode::PoolTopStatusNested => {
            assert!(
                v.get("node_total").is_some(),
                "{context} top-level MUST include pool panel fields (AC-007.84); got: {v}"
            );
            let status = v.get("status").expect("{context} MUST include nested status (AC-007.84)");
            assert_status_object(status, context);
            let host_pos = raw.find("\"host_watch\"").expect("host_watch key");
            let status_pos = raw.rfind("\"status\"").expect("status key (AC-007.84)");
            assert!(
                host_pos < status_pos,
                "{context} MUST serialize host_watch before nested status (AC-007.84); got: {raw}"
            );
        }
        JsonEnvelopeMode::StatusTopPoolNested => {
            assert!(
                v.get("total_processes").is_some(),
                "{context} top-level MUST include proc-scan fields (AC-007.84); got: {v}"
            );
            let pool = v.get("pool").expect("{context} MUST include nested pool (AC-007.84)");
            assert_pool_object(pool, context);
            let host_pos = raw.find("\"host_watch\"").expect("host_watch key");
            let pool_pos = raw.rfind("\"pool\"").expect("pool key (AC-007.84)");
            assert!(
                host_pos < pool_pos,
                "{context} MUST serialize host_watch before nested pool (AC-007.84); got: {raw}"
            );
        }
    }
}

fn assert_csv_companion_order(stdout: &str, body_header: &str, context: &str) {
    assert!(
        stdout.contains(body_header),
        "{context} MUST include command CSV body (AC-007.84); got: {stdout}"
    );
    for header in [
        GATE_CSV_HEADER,
        HOST_CSV_HEADER,
        POOL_COMPANION_CSV_HEADER,
        STATUS_COMPANION_CSV_HEADER,
    ] {
        assert!(stdout.contains(header), "{context} MUST include {header} (AC-007.84)");
    }
    for prefix in ["gate,", "host,", "pool,", "status,"] {
        assert!(
            stdout.lines().any(|line| line.starts_with(prefix)),
            "{context} MUST include {prefix} companion row (AC-007.84); got: {stdout}"
        );
    }

    let body_pos = stdout.find(body_header).expect("body header");
    let gate_pos = stdout.find(GATE_CSV_HEADER).expect("gate header");
    let host_pos = stdout[gate_pos..]
        .find(HOST_CSV_HEADER)
        .map(|p| gate_pos + p)
        .expect("host header after gate");
    let pool_pos = stdout[host_pos..]
        .find(POOL_COMPANION_CSV_HEADER)
        .map(|p| host_pos + p)
        .expect("pool companion after host");
    let status_pos = stdout[pool_pos..]
        .find(STATUS_COMPANION_CSV_HEADER)
        .map(|p| pool_pos + p)
        .expect("status companion after pool");

    assert!(
        body_pos < gate_pos
            && gate_pos < host_pos
            && host_pos < pool_pos
            && pool_pos < status_pos,
        "{context} MUST serialize body → gate → host_watch → pool → status (AC-007.84); got: {stdout}"
    );
}

fn run_cli(args: &[&str]) -> std::process::Output {
    bin().args(args).output().expect("spawn sharecli")
}

fn sample_pool_snapshot() -> PoolSnapshot {
    PoolSnapshot {
        node_total: 2,
        node_idle: 1,
        bun_total: 1,
        bun_idle: 0,
        max_per_type: 4,
        healthy: true,
        issues: vec![],
        gate: GateStatusSnapshot {
            thermal_pressure: "GREEN".into(),
            detected_agents: 0,
            agent_total_rss_bytes: 0,
            agent_contention: "OK".into(),
            gate_decision: "ADMIT".into(),
        },
        host_watch: HostResourceWatchJson {
            fd_count: 1,
            net_rx_bytes: 2,
            net_tx_bytes: 3,
            mem_rss_bytes: 4,
            load_1m: 0.5,
        },
    }
}

fn sample_status_snapshot() -> StatusSnapshot {
    StatusSnapshot {
        total_processes: 2,
        agents: vec![],
        scanned: 50,
        watched: 1,
        gate: GateStatusSnapshot {
            thermal_pressure: "GREEN".into(),
            detected_agents: 0,
            agent_total_rss_bytes: 0,
            agent_contention: "OK".into(),
            gate_decision: "ADMIT".into(),
        },
        host_watch: HostResourceWatchJson {
            fd_count: 1,
            net_rx_bytes: 2,
            net_tx_bytes: 3,
            mem_rss_bytes: 4,
            load_1m: 0.5,
        },
    }
}

/// FR-007 / AC-007.84 — CLI text one-shot matrix: body → gate → host_watch → pool → proc-scan.
#[test]
#[serial_test::serial]
fn fr007_operator_matrix_cli_text_one_shot() {
    let cases = [
        TextCase {
            label: "report",
            args: &["report"],
            body_header: "=== Fleet Analytics Report ===",
        },
        TextCase {
            label: "proc",
            args: &["proc"],
            body_header: "=== Host agents (proc scan) ===",
        },
        TextCase {
            label: "health",
            args: &["health"],
            body_header: "Shared runtime health:",
        },
        TextCase {
            label: "pool",
            args: &["pool"],
            body_header: "=== Shared Runtime Pool Status ===",
        },
        TextCase {
            label: "status",
            args: &["status"],
            body_header: "=== Process Status ===",
        },
        TextCase {
            label: "ps --all",
            args: &["ps", "--all"],
            body_header: "=== Host agents (proc scan) ===",
        },
    ];

    for case in cases {
        let out = run_cli(case.args);
        assert!(out.status.success(), "{} MUST exit 0 (AC-007.84)", case.label);
        assert_stderr_silent(&out.stderr, case.label);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_text_envelope(&stdout, case.body_header, case.label);
    }
}

/// FR-007 / AC-007.84 — CLI JSON one-shot matrix: gate → host_watch → pool/status siblings.
#[test]
#[serial_test::serial]
fn fr007_operator_matrix_cli_json_one_shot() {
    let cases = [
        JsonCase {
            label: "health --json",
            args: &["health", "--json"],
            mode: JsonEnvelopeMode::FullPoolStatus,
        },
        JsonCase {
            label: "pool --json",
            args: &["pool", "--json"],
            mode: JsonEnvelopeMode::PoolTopStatusNested,
        },
        JsonCase {
            label: "status --json",
            args: &["status", "--json"],
            mode: JsonEnvelopeMode::StatusTopPoolNested,
        },
        JsonCase {
            label: "ps --all --json",
            args: &["ps", "--all", "--json"],
            mode: JsonEnvelopeMode::FullPoolStatus,
        },
        JsonCase {
            label: "proc --json",
            args: &["proc", "--json"],
            mode: JsonEnvelopeMode::FullPoolStatus,
        },
        JsonCase {
            label: "report --format json",
            args: &["report", "--format", "json"],
            mode: JsonEnvelopeMode::FullPoolStatus,
        },
    ];

    for case in cases {
        let out = run_cli(case.args);
        assert!(out.status.success(), "{} MUST exit 0 (AC-007.84)", case.label);
        assert_stderr_silent(&out.stderr, case.label);
        let raw = String::from_utf8_lossy(&out.stdout);
        assert_json_envelope(&raw, case.mode, case.label);
    }
}

/// FR-007 / AC-007.84 — CLI CSV one-shot matrix: body → gate → host_watch → pool → status.
#[test]
#[serial_test::serial]
fn fr007_operator_matrix_cli_csv_one_shot() {
    let cases = [
        CsvCase {
            label: "proc --csv",
            args: &["proc", "--csv"],
            body_header: "pid,family,comm,state,mem_rss_bytes,mem_rss,fd_count",
        },
        CsvCase {
            label: "report --format csv",
            args: &["report", "--format", "csv"],
            body_header: "record,timestamp,uptime_seconds,total_processes,total_memory_mb,thermal_pressure,detected_agents,agent_contention,gate_decision",
        },
        CsvCase {
            label: "health --csv",
            args: &["health", "--csv"],
            body_header: "record,healthy,node_total,node_idle,node_in_use,bun_total,bun_idle,bun_in_use,max_per_type,issues",
        },
        CsvCase {
            label: "pool --csv",
            args: &["pool", "--csv"],
            body_header: "record,node_total,node_idle,bun_total,bun_idle,max_per_type,healthy,issues",
        },
        CsvCase {
            label: "status --csv",
            args: &["status", "--csv"],
            body_header: "record,total_processes,scanned,watched,agent_rows",
        },
        CsvCase {
            label: "ps --all --csv",
            args: &["ps", "--all", "--csv"],
            body_header: "record,pid,name,memory_mb,project,harness,agent",
        },
    ];

    for case in cases {
        let out = run_cli(case.args);
        assert!(out.status.success(), "{} MUST exit 0 (AC-007.84)", case.label);
        assert_stderr_silent(&out.stderr, case.label);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_csv_companion_order(&stdout, case.body_header, case.label);
    }
}

/// FR-007 / AC-007.84 — IPC health.status + monitoring.report carry gate → host_watch → pool/status.
#[tokio::test]
async fn fr007_operator_matrix_ipc_envelopes() {
    let handler = sharecli_ipc::handler::Handler::new().await.expect("Handler::new");

    let health = handler
        .dispatch(r#"{"id":1,"method":"health.status","params":{}}"#)
        .await;
    assert!(health.error.is_none(), "health.status MUST succeed (AC-007.84)");
    let health_raw = serde_json::to_string(&health.result).expect("serialize health.status");
    assert_json_envelope(&health_raw, JsonEnvelopeMode::FullPoolStatus, "health.status");

    let report = handler
        .dispatch(r#"{"id":2,"method":"monitoring.report","params":{}}"#)
        .await;
    assert!(report.error.is_none(), "monitoring.report MUST succeed (AC-007.84)");
    let report_raw = serde_json::to_string(&report.result).expect("serialize monitoring.report");
    assert_json_envelope(&report_raw, JsonEnvelopeMode::FullPoolStatus, "monitoring.report");
}

/// FR-007 / AC-007.84 — WS health_update decode stays on the expanded operator envelope path.
#[test]
fn fr007_operator_matrix_ws_health_update_decode() {
    use sharecli_ipc::ws_client::ClientMessage;

    let health = r#"{
        "managed_processes":3,"used_memory_mb":2048,"total_memory_mb":16384,"healthy":true,
        "gate":{"thermal_pressure":"GREEN","detected_agents":0,
        "agent_total_rss_bytes":0,"agent_contention":"OK","gate_decision":"ADMIT"},
        "host_watch":{"fd_count":1,"net_rx_bytes":2,"net_tx_bytes":3,"mem_rss_bytes":4,"load_1m":0.5},
        "pool":{"node_total":2,"node_idle":1,"bun_total":1,"bun_idle":0,"max_per_type":4,
        "healthy":true,"issues":[],
        "gate":{"thermal_pressure":"GREEN","detected_agents":0,
        "agent_total_rss_bytes":0,"agent_contention":"OK","gate_decision":"ADMIT"},
        "host_watch":{"fd_count":1,"net_rx_bytes":2,"net_tx_bytes":3,"mem_rss_bytes":4,"load_1m":0.5}},
        "status":{"total_processes":2,"agents":[],"scanned":50,"watched":1,
        "gate":{"thermal_pressure":"GREEN","detected_agents":0,
        "agent_total_rss_bytes":0,"agent_contention":"OK","gate_decision":"ADMIT"},
        "host_watch":{"fd_count":1,"net_rx_bytes":2,"net_tx_bytes":3,"mem_rss_bytes":4,"load_1m":0.5}}
    }"#;
    let raw = format!(r#"{{"type":"health_update","health":{health}}}"#);
    match ClientMessage::from_json(&raw) {
        ClientMessage::HealthUpdate(h) => {
            assert_eq!(h.pool.node_total, 2);
            assert_eq!(h.status.scanned, 50);
            assert_json_envelope(health, JsonEnvelopeMode::FullPoolStatus, "HealthUpdate wire");
        }
        other => panic!("expected HealthUpdate, got {other:?} (AC-007.84)"),
    }
}

/// FR-007 / AC-007.84 — tray formatter markers align across Linux + Windows.
#[test]
fn fr007_operator_matrix_tray_formatter_markers() {
    let pool = sample_pool_snapshot();
    let status = sample_status_snapshot();

    let linux_pool = linux_display::format_pool_tray_line(&pool);
    let linux_status = linux_display::format_status_snapshot_tray_line(&status);

    use sharecli_tray_windows::ipc::{
        GateStatusSnapshot as WinGate, HostResourceWatchJson as WinHost, PoolSnapshot as WinPool,
        StatusSnapshot as WinStatus,
    };

    let win_pool = WinPool {
        node_total: pool.node_total,
        node_idle: pool.node_idle,
        bun_total: pool.bun_total,
        bun_idle: pool.bun_idle,
        max_per_type: pool.max_per_type,
        healthy: pool.healthy,
        issues: pool.issues.clone(),
        gate: WinGate {
            thermal_pressure: pool.gate.thermal_pressure.clone(),
            detected_agents: pool.gate.detected_agents,
            agent_total_rss_bytes: pool.gate.agent_total_rss_bytes,
            agent_contention: pool.gate.agent_contention.clone(),
            gate_decision: pool.gate.gate_decision.clone(),
        },
        host_watch: WinHost {
            fd_count: pool.host_watch.fd_count,
            net_rx_bytes: pool.host_watch.net_rx_bytes,
            net_tx_bytes: pool.host_watch.net_tx_bytes,
            mem_rss_bytes: pool.host_watch.mem_rss_bytes,
            load_1m: pool.host_watch.load_1m,
        },
    };
    let win_status = WinStatus {
        total_processes: status.total_processes,
        agents: vec![],
        scanned: status.scanned,
        watched: status.watched,
        gate: WinGate {
            thermal_pressure: status.gate.thermal_pressure.clone(),
            detected_agents: status.gate.detected_agents,
            agent_total_rss_bytes: status.gate.agent_total_rss_bytes,
            agent_contention: status.gate.agent_contention.clone(),
            gate_decision: status.gate.gate_decision.clone(),
        },
        host_watch: WinHost {
            fd_count: status.host_watch.fd_count,
            net_rx_bytes: status.host_watch.net_rx_bytes,
            net_tx_bytes: status.host_watch.net_tx_bytes,
            mem_rss_bytes: status.host_watch.mem_rss_bytes,
            load_1m: status.host_watch.load_1m,
        },
    };

    let win_pool_line = win_display::format_pool_tray_line(&win_pool);
    let win_status_line = win_display::format_status_snapshot_tray_line(&win_status);

    for (label, line) in [
        ("linux pool", &linux_pool),
        ("linux status", &linux_status),
        ("windows pool", &win_pool_line),
        ("windows status", &win_status_line),
    ] {
        assert!(
            line.starts_with(POOL_PREFIX) || line.starts_with(PROC_PREFIX),
            "{label} MUST use operator prefix markers (AC-007.84); got: {line}"
        );
    }
    assert!(linux_pool.starts_with(POOL_PREFIX) && win_pool_line.starts_with(POOL_PREFIX));
    assert!(linux_status.starts_with(PROC_PREFIX) && win_status_line.starts_with(PROC_PREFIX));
}

/// FR-007 / AC-007.84 — Swift + C# tray sources ship pool/proc-scan operator formatters.
#[test]
fn fr007_operator_matrix_tray_source_markers() {
    let swift = fs::read_to_string(manifest_path(
        "desktop/ShareCLITray/Sources/ShareCLICore/OperatorDisplay.swift",
    ))
    .expect("read Swift OperatorDisplay.swift");
    assert!(swift.contains("Pool node"), "Swift tray MUST format pool line (AC-007.84)");
    assert!(swift.contains("Proc scan"), "Swift tray MUST format proc-scan line (AC-007.84)");

    let csharp = fs::read_to_string(manifest_path("windows/ShareCLITray/OperatorDisplay.cs"))
        .expect("read OperatorDisplay.cs");
    assert!(csharp.contains("Pool node"), "WinUI tray MUST format pool line (AC-007.84)");
    assert!(csharp.contains("Proc scan"), "WinUI tray MUST format proc-scan line (AC-007.84)");
}

/// FR-007 / AC-007.84 — dashboard HTML + thermal TUI companion panel markers present.
#[test]
fn fr007_operator_matrix_dashboard_tui_markers() {
    let html = fs::read_to_string(manifest_path("src/dashboard.html")).expect("read dashboard.html");
    for marker in [
        "data-operator-panels",
        "panel-gate",
        "panel-host-watch",
        "panel-pool",
        "panel-status",
        "renderOperatorPanels",
    ] {
        assert!(html.contains(marker), "dashboard MUST include {marker} (AC-007.84)");
    }

    assert!(HELP_OVERLAY_HINT.contains("2 pool"));
    assert!(HELP_OVERLAY_HINT.contains("3 status"));

    let pool_lines = pool_panel_lines(
        Some(PoolOperatorPanel {
            node_total: 2,
            node_idle: 1,
            bun_total: 1,
            bun_idle: 0,
            max_per_type: 4,
            healthy: true,
        }),
        false,
    );
    let status_lines = status_panel_lines(
        Some(StatusOperatorPanel {
            scanned: 50,
            watched: 1,
            total_processes: 2,
            agent_rows: 1,
        }),
        false,
    );
    let pool_text: String = pool_lines.iter().map(|l| l.to_string()).collect();
    let status_text: String = status_lines.iter().map(|l| l.to_string()).collect();
    assert!(pool_text.contains("Node idle/total"), "TUI pool panel MUST render (AC-007.84)");
    assert!(status_text.contains("Scanned:"), "TUI status panel MUST render (AC-007.84)");
}
