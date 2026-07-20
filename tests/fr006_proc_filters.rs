//! FR-006 — `sharecli proc` inventory filters
//! FR: FR-006
//!
//! AC-006.17 `--family` and `--min-rss` narrow proc inventory and tree forests
//! AC-006.27 `--max-rss` upper-bound RSS filter (complements `--min-rss`)

use std::process::Command;

use sharecli::commands::proc::{filter_agent_forests, filter_watched_agents, ProcFilter};
use sharecli_fleet::{
    proc_scan::{DetectedAgent, FakeProcSource, ProcSnapshot},
    AgentResourceSample, DetectedAgentWatch,
};

fn empty_ppid_map() -> std::collections::HashMap<u32, u32> {
    std::collections::HashMap::new()
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

/// FR-006 / AC-006.17 — proc help documents filter flags.
#[test]
fn fr006_proc_help_documents_filters() {
    let out = bin().args(["proc", "--help"]).output().expect("spawn sharecli proc --help");
    assert!(out.status.success(), "proc --help should exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("--family"), "proc --help MUST document --family; got: {s}");
    assert!(s.contains("--min-rss"), "proc --help MUST document --min-rss; got: {s}");
    assert!(s.contains("--max-rss"), "proc --help MUST document --max-rss; got: {s}");
}

/// FR-006 / AC-006.17 — family filter keeps matching agents only.
#[test]
fn fr006_proc_family_filter() {
    let rows = vec![watch_row("claude", 10, 100), watch_row("codex", 11, 200)];
    let filtered = filter_watched_agents(
        &rows,
        &ProcFilter { family: Some("claude".into()), min_rss_bytes: None, max_rss_bytes: None, ppid: None },
        &empty_ppid_map(),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].agent.family, "claude");
}

/// FR-006 / AC-006.17 — min-rss filter drops agents below threshold.
#[test]
fn fr006_proc_min_rss_filter() {
    let rows =
        vec![watch_row("claude", 10, 50 * 1_048_576), watch_row("codex", 11, 200 * 1_048_576)];
    let filtered = filter_watched_agents(
        &rows,
        &ProcFilter { family: None, min_rss_bytes: Some(100 * 1_048_576), max_rss_bytes: None, ppid: None },
        &empty_ppid_map(),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].agent.pid, 11);
}

/// FR-006 / AC-006.17 — tree forests honor family filter on roots.
#[test]
fn fr006_proc_tree_family_filter() {
    let src = FakeProcSource::new(vec![
        ProcSnapshot { pid: 1, ppid: 0, comm: "init".into(), cmdline: vec![] },
        ProcSnapshot { pid: 50, ppid: 1, comm: "claude".into(), cmdline: vec!["claude".into()] },
        ProcSnapshot { pid: 60, ppid: 1, comm: "codex".into(), cmdline: vec!["codex".into()] },
    ]);
    let forests = sharecli_fleet::build_agent_forests(&src);
    assert_eq!(forests.len(), 2);
    let filtered = filter_agent_forests(
        &forests,
        &ProcFilter { family: Some("codex".into()), min_rss_bytes: None, max_rss_bytes: None, ppid: None },
        &std::collections::HashMap::new(),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].family, Some("codex"));
}

/// FR-006 / AC-006.17 — invalid min-rss exits non-zero.
#[test]
fn fr006_proc_invalid_min_rss_rejected() {
    let out = bin()
        .args(["proc", "--min-rss", "not-a-size"])
        .output()
        .expect("spawn sharecli proc --min-rss");
    assert!(!out.status.success(), "invalid --min-rss MUST fail");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("min-rss") || err.contains("RSS"),
        "error MUST mention min-rss; got: {err}"
    );
}

/// FR-006 / AC-006.27 — max-rss filter drops agents above threshold.
#[test]
fn fr006_proc_max_rss_filter() {
    let rows =
        vec![watch_row("claude", 10, 50 * 1_048_576), watch_row("codex", 11, 200 * 1_048_576)];
    let filtered = filter_watched_agents(
        &rows,
        &ProcFilter {
            family: None,
            min_rss_bytes: None,
            max_rss_bytes: Some(100 * 1_048_576),
            ppid: None,
        },
        &empty_ppid_map(),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].agent.pid, 10);
}

/// FR-006 / AC-006.27 — min-rss and max-rss compose as an RSS band.
#[test]
fn fr006_proc_rss_band_filter() {
    let rows = vec![
        watch_row("claude", 10, 50 * 1_048_576),
        watch_row("codex", 11, 150 * 1_048_576),
        watch_row("amp", 12, 250 * 1_048_576),
    ];
    let filtered = filter_watched_agents(
        &rows,
        &ProcFilter {
            family: None,
            min_rss_bytes: Some(100 * 1_048_576),
            max_rss_bytes: Some(200 * 1_048_576),
            ppid: None,
        },
        &empty_ppid_map(),
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].agent.pid, 11);
}

/// FR-006 / AC-006.27 — tree forests honor max-rss on roots.
#[test]
fn fr006_proc_tree_max_rss_filter() {
    let src = FakeProcSource::new(vec![
        ProcSnapshot { pid: 1, ppid: 0, comm: "init".into(), cmdline: vec![] },
        ProcSnapshot { pid: 50, ppid: 1, comm: "claude".into(), cmdline: vec!["claude".into()] },
        ProcSnapshot { pid: 60, ppid: 1, comm: "codex".into(), cmdline: vec!["codex".into()] },
    ]);
    let forests = sharecli_fleet::build_agent_forests(&src);
    let mut rss_by_pid = std::collections::HashMap::new();
    rss_by_pid.insert(50, 50 * 1_048_576);
    rss_by_pid.insert(60, 200 * 1_048_576);
    let filtered = filter_agent_forests(
        &forests,
        &ProcFilter {
            family: None,
            min_rss_bytes: None,
            max_rss_bytes: Some(100 * 1_048_576),
            ppid: None,
        },
        &rss_by_pid,
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].pid, 50);
}

/// FR-006 / AC-006.27 — invalid max-rss exits non-zero.
#[test]
fn fr006_proc_invalid_max_rss_rejected() {
    let out = bin()
        .args(["proc", "--max-rss", "not-a-size"])
        .output()
        .expect("spawn sharecli proc --max-rss");
    assert!(!out.status.success(), "invalid --max-rss MUST fail");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("max-rss") || err.contains("RSS"),
        "error MUST mention max-rss; got: {err}"
    );
}

/// FR-006 / AC-006.27 — min-rss greater than max-rss fails loudly.
#[test]
fn fr006_proc_min_rss_exceeds_max_rss_rejected() {
    let out = bin()
        .args(["proc", "--min-rss", "200M", "--max-rss", "100M"])
        .output()
        .expect("spawn sharecli proc min>max");
    assert!(!out.status.success(), "min-rss > max-rss MUST fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.contains("min-rss") && combined.contains("max-rss"),
        "error MUST mention both RSS bounds; got: {combined}"
    );
}

fn watch_row(family: &'static str, pid: u32, rss: u64) -> DetectedAgentWatch {
    DetectedAgentWatch {
        agent: DetectedAgent { pid, family, comm: family.into() },
        resource: AgentResourceSample { mem_rss_bytes: rss, fd_count: None },
    }
}
