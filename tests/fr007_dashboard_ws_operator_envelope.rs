//! FR-007 — dashboard WebSocket operator envelope (gate + host_watch + pool + status parity)
//! FR: FR-007
//!
//! AC-007.41 `sharecli serve` `/ws` snapshots carry top-level gate + host_watch siblings.
//! AC-007.70 extends the envelope with pool + status (proc scan) siblings for dashboard parity.

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const HOST_WATCH_KEYS: [&str; 5] =
    ["fd_count", "net_rx_bytes", "net_tx_bytes", "mem_rss_bytes", "load_1m"];

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

fn dashboard_html() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/dashboard.html");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn pick_port() -> u16 {
    19_500 + (std::process::id() % 800) as u16
}

struct ServeChild {
    child: Child,
    port: u16,
}

impl ServeChild {
    fn spawn(port: u16) -> Self {
        let child = bin()
            .args(["serve", "--bind", &format!("127.0.0.1:{port}"), "--on-conflict", "replace"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sharecli serve");
        Self { child, port }
    }

    fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/ws", self.port)
    }
}

impl Drop for ServeChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn wait_ws_message(url: &str, timeout: Duration) -> Option<String> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        let out = Command::new("websocat").args(["-n1", url]).output();
        if let Ok(out) = out {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    None
}

fn assert_ws_envelope(raw: &str) {
    let v: serde_json::Value =
        serde_json::from_str(raw).expect("dashboard WS MUST emit valid JSON (AC-007.41)");
    assert!(
        v.get("gate").and_then(|g| g.get("gate_decision")).is_some(),
        "dashboard WS MUST include gate (AC-007.41); got: {v}"
    );
    let host = v.get("host_watch").expect("dashboard WS MUST include host_watch (AC-007.41)");
    for key in HOST_WATCH_KEYS {
        assert!(host.get(key).is_some(), "host_watch MUST include {key} (AC-007.41)");
    }
    assert!(
        v.get("agents").and_then(|a| a.get("scanned")).is_some(),
        "dashboard WS MUST include agents summary (AC-007.41); got: {v}"
    );
    assert!(v.get("processes").and_then(|p| p.as_array()).is_some());

    let pool = v.get("pool").expect("dashboard WS MUST include pool (AC-007.70)");
    assert!(
        pool.get("node_total").is_some() && pool.get("healthy").is_some(),
        "pool MUST include capacity fields (AC-007.70); got: {pool}"
    );
    let status = v.get("status").expect("dashboard WS MUST include status (AC-007.70)");
    assert!(
        status.get("total_processes").is_some()
            && status.get("scanned").is_some()
            && status.get("watched").is_some(),
        "status MUST include proc-scan fields (AC-007.70); got: {status}"
    );

    let gate_pos = raw.find("\"gate\"").expect("gate key in WS JSON");
    let host_pos = raw.find("\"host_watch\"").expect("host_watch key in WS JSON");
    let pool_pos = raw.find("\"pool\"").expect("pool key in WS JSON (AC-007.70)");
    let status_pos = raw.find("\"status\"").expect("status key in WS JSON (AC-007.70)");
    let agents_pos = raw.find("\"agents\"").expect("agents key in WS JSON");
    let processes_pos = raw.find("\"processes\"").expect("processes key in WS JSON");
    assert!(
        gate_pos < host_pos,
        "dashboard WS MUST serialize gate before host_watch (AC-007.41); got: {raw}"
    );
    assert!(
        gate_pos < host_pos
            && host_pos < pool_pos
            && pool_pos < status_pos
            && status_pos < agents_pos
            && agents_pos < processes_pos,
        "dashboard WS MUST serialize gate → host_watch → pool → status → agents → processes (AC-007.70); got: {raw}"
    );
}

/// FR-007 / AC-007.41 + AC-007.70 — embedded dashboard ships operator panels for gate/host/pool/status/agents.
#[test]
fn fr007_dashboard_operator_panel_markup() {
    let html = dashboard_html();
    assert!(
        html.contains("data-operator-panels"),
        "dashboard MUST ship operator panels region (AC-007.41)"
    );
    assert!(html.contains("panel-gate"), "dashboard MUST include gate panel");
    assert!(html.contains("panel-host-watch"), "dashboard MUST include host watch panel");
    assert!(html.contains("panel-pool"), "dashboard MUST include pool panel (AC-007.70)");
    assert!(
        html.contains("panel-status"),
        "dashboard MUST include status snapshot panel (AC-007.70)"
    );
    assert!(html.contains("panel-agents"), "dashboard MUST include agents summary panel");
    assert!(
        html.contains("renderOperatorPanels"),
        "dashboard MUST render operator envelope from WS payload (AC-007.41)"
    );
}

/// FR-007 / AC-007.41 + AC-007.70 — live serve `/ws` snapshot matches operator envelope contract.
#[test]
#[serial_test::serial]
fn fr007_dashboard_ws_operator_envelope_e2e() {
    if Command::new("websocat").arg("--version").status().is_err() {
        eprintln!("websocat not available; skipping fr007_dashboard_ws_operator_envelope_e2e");
        return;
    }

    let port = pick_port();
    let serve = ServeChild::spawn(port);
    let url = serve.ws_url();

    let raw = wait_ws_message(&url, Duration::from_secs(20))
        .unwrap_or_else(|| panic!("dashboard WS must deliver a snapshot at {url}"));
    assert_ws_envelope(&raw);
}

/// FR-007 / AC-007.41 + AC-007.70 — library snapshot builder matches operator envelope shapes.
#[tokio::test]
async fn fr007_dashboard_ws_snapshot_lib_shape() {
    let _ = sharecli::config::init_global();
    let snap = sharecli::commands::serve::build_dashboard_ws_snapshot()
        .await
        .expect("build_dashboard_ws_snapshot");
    let raw = serde_json::to_string(&snap).expect("serialize");
    assert_ws_envelope(&raw);
}
