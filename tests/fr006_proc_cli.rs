//! FR-006 — `sharecli proc` host agent inventory CLI
//! FR: FR-006
//!
//! AC-006.11 `sharecli proc` lists detected agents with RSS/FD
//! AC-006.13 `sharecli proc --json` structured agent + gate payload

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

/// FR-006 / AC-006.11 — proc subcommand prints host agent inventory header.
#[test]
fn fr006_proc_cli_prints_inventory_header() {
    let out = bin().args(["proc"]).output().expect("spawn sharecli proc");
    assert!(out.status.success(), "proc should exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("Host agents (proc scan)"),
        "proc MUST print host agent inventory; got: {s}"
    );
}

/// FR-006 / AC-006.13 — proc --json emits agents + gate object.
#[test]
fn fr006_proc_cli_json_shape() {
    let out = bin().args(["proc", "--json"]).output().expect("spawn sharecli proc --json");
    assert!(out.status.success(), "proc --json should exit 0; stderr: {:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("proc --json MUST emit valid JSON");
    assert!(v.get("agents").and_then(|a| a.as_array()).is_some());
    assert!(v.get("gate").and_then(|g| g.get("detected_agents")).is_some());
    assert!(v.get("gate").and_then(|g| g.get("agent_total_rss_bytes")).is_some());
}

/// FR-006 / AC-006.13 — status --json includes detected agent inventory.
#[test]
fn fr006_status_json_includes_agents() {
    let out = bin().args(["status", "--json"]).output().expect("spawn sharecli status --json");
    assert!(out.status.success(), "status --json should exit 0; stderr: {:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("status --json MUST emit valid JSON");
    assert!(v.get("total_processes").is_some());
    let agents = v.get("agents").expect("status JSON MUST include agents object");
    assert!(agents.get("gate").is_some());
    assert!(agents.get("scanned").is_some());
}
