//! FR-006 — `sharecli proc --tree` process-state on text nodes and tree JSON
//! FR: FR-006
//!
//! AC-006.34 expose `state` on `--tree` text nodes and `--tree --json`
//! `AgentTreeNodeJson` rows (tree CSV already has state from AC-006.32)

use std::collections::HashMap;
use std::process::Command;

use sharecli::commands::proc::agent_tree_node_to_json;
use sharecli_fleet::proc_scan::{FakeProcSource, ProcSnapshot};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

/// FR-006 / AC-006.34 — tree JSON nodes include state letter.
#[test]
fn fr006_proc_tree_state_json_node_includes_state() {
    let src = FakeProcSource::new(vec![
        ProcSnapshot { pid: 1, ppid: 0, comm: "init".into(), cmdline: vec![], state: 'R' },
        ProcSnapshot {
            pid: 50,
            ppid: 1,
            comm: "cursor-agent".into(),
            cmdline: vec!["cursor-agent".into()],
            state: 'S',
        },
        ProcSnapshot { pid: 51, ppid: 50, comm: "node".into(), cmdline: vec!["node".into()], state: 'R' },
    ]);
    let forests = sharecli_fleet::build_agent_forests(&src);
    let mut state_by_pid = HashMap::new();
    state_by_pid.insert(50, 'S');
    state_by_pid.insert(51, 'R');
    let root = agent_tree_node_to_json(&forests[0], &state_by_pid);
    assert_eq!(root.state, "S");
    assert_eq!(root.children[0].state, "R");
    let json = serde_json::to_string(&root).expect("serialize tree node");
    assert!(
        json.contains("\"state\":\"S\""),
        "tree JSON MUST include state on root; got: {json}"
    );
}

/// FR-006 / AC-006.34 — missing tree JSON state serializes as empty string.
#[test]
fn fr006_proc_tree_state_json_missing_state_empty_string() {
    let src = FakeProcSource::new(vec![ProcSnapshot {
        pid: 99,
        ppid: 1,
        comm: "codex".into(),
        cmdline: vec!["codex".into()],
        state: 'R',
    }]);
    let forests = sharecli_fleet::build_agent_forests(&src);
    let node = agent_tree_node_to_json(&forests[0], &HashMap::new());
    assert_eq!(node.state, "");
    let json = serde_json::to_string(&node).expect("serialize tree node");
    assert!(
        json.contains("\"state\":\"\""),
        "missing state MUST be empty JSON string; got: {json}"
    );
}

/// FR-006 / AC-006.34 — CLI proc --tree --json forests nodes expose state key.
#[test]
fn fr006_proc_tree_state_cli_json_includes_state_key() {
    let out = bin()
        .args(["proc", "--tree", "--json"])
        .output()
        .expect("spawn proc --tree --json");
    assert!(out.status.success(), "proc --tree --json MUST exit 0; stderr: {:?}", out.stderr);
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("proc --tree --json MUST emit valid JSON");
    let forests = v.get("forests").and_then(|f| f.as_array()).expect("forests array");
    if let Some(root) = forests.first() {
        assert!(
            root.get("state").is_some(),
            "tree JSON root MUST include state when forests present; got: {root}"
        );
    }
}

/// FR-006 / AC-006.34 — CLI proc --tree text nodes show state letter after PID.
#[test]
fn fr006_proc_tree_state_cli_text_shows_state_on_nodes() {
    let out = bin().args(["proc", "--tree"]).output().expect("spawn proc --tree");
    assert!(out.status.success(), "proc --tree MUST exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    if s.contains('[') && s.chars().any(|c| c.is_ascii_digit()) {
        assert!(
            s.contains("] ") && (s.contains("] R ") || s.contains("] S ") || s.contains("] D ")
                || s.contains("] Z ") || s.contains("] T ") || s.contains("] - ")),
            "tree text nodes MUST show state after PID bracket; got: {s}"
        );
    }
}
