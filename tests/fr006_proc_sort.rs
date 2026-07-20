//! FR-006 — `sharecli proc --sort`
//! FR: FR-006
//!
//! AC-006.19 `--sort rss|fd|pid` orders inventory rows and tree root forests

use std::process::Command;

use sharecli::commands::proc::{sort_agent_forests, sort_watched_agents, ProcSort};
use sharecli_fleet::{
    proc_scan::{AgentTreeNode, DetectedAgent},
    AgentResourceSample, DetectedAgentWatch,
};
use std::collections::HashMap;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

fn watch(pid: u32, family: &'static str, rss: u64, fds: Option<u64>) -> DetectedAgentWatch {
    DetectedAgentWatch {
        agent: DetectedAgent { pid, family, comm: family.into() },
        resource: AgentResourceSample { mem_rss_bytes: rss, fd_count: fds },
    }
}

fn tree_root(pid: u32, family: Option<&'static str>) -> AgentTreeNode {
    AgentTreeNode { pid, ppid: 1, comm: "agent".into(), family, children: vec![] }
}

/// FR-006 / AC-006.19 — proc help documents --sort.
#[test]
fn fr006_proc_help_documents_sort() {
    let out = bin().args(["proc", "--help"]).output().expect("spawn sharecli proc --help");
    assert!(out.status.success(), "proc --help should exit 0; stderr: {:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("--sort"), "proc --help MUST document --sort; got: {s}");
}

/// FR-006 / AC-006.19 — rss sort is descending with pid tie-break.
#[test]
fn fr006_proc_sort_rss_descending() {
    let rows = vec![
        watch(10, "claude", 100, Some(5)),
        watch(20, "codex", 300, Some(1)),
        watch(30, "aider", 200, Some(2)),
    ];
    let sorted = sort_watched_agents(&rows, ProcSort::Rss);
    assert_eq!(sorted.iter().map(|r| r.agent.pid).collect::<Vec<_>>(), vec![20, 30, 10]);
}

/// FR-006 / AC-006.19 — fd sort treats missing FD as zero, descending.
#[test]
fn fr006_proc_sort_fd_descending() {
    let rows = vec![
        watch(1, "claude", 100, None),
        watch(2, "codex", 100, Some(50)),
        watch(3, "aider", 100, Some(10)),
    ];
    let sorted = sort_watched_agents(&rows, ProcSort::Fd);
    assert_eq!(sorted.iter().map(|r| r.agent.pid).collect::<Vec<_>>(), vec![2, 3, 1]);
}

/// FR-006 / AC-006.19 — pid sort is ascending.
#[test]
fn fr006_proc_sort_pid_ascending() {
    let rows = vec![watch(99, "claude", 1, None), watch(1, "codex", 1, None)];
    let sorted = sort_watched_agents(&rows, ProcSort::Pid);
    assert_eq!(sorted.iter().map(|r| r.agent.pid).collect::<Vec<_>>(), vec![1, 99]);
}

/// FR-006 / AC-006.19 — tree root forests honor rss sort via live samples.
#[test]
fn fr006_proc_sort_tree_roots_by_rss() {
    let forests = vec![tree_root(10, Some("claude")), tree_root(20, Some("codex"))];
    let mut rss = HashMap::new();
    rss.insert(10, 500_u64);
    rss.insert(20, 100_u64);
    let sorted = sort_agent_forests(&forests, ProcSort::Rss, &rss, &HashMap::new());
    assert_eq!(sorted.iter().map(|n| n.pid).collect::<Vec<_>>(), vec![10, 20]);
}

/// FR-006 / AC-006.19 — invalid sort key exits non-zero.
#[test]
fn fr006_proc_sort_invalid_exits_nonzero() {
    let out =
        bin().args(["proc", "--sort", "name"]).output().expect("spawn sharecli proc --sort name");
    assert!(!out.status.success(), "invalid --sort MUST fail; stdout: {:?}", out.stdout);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("rss") || err.contains("sort"),
        "error MUST mention valid keys; got: {err}"
    );
}
