//! FR-006 — `sharecli proc` host agent inventory CLI.

use std::time::Duration;

use anyhow::{bail, Result};
use serde::Serialize;
use sharecli_fleet::thermal::ThermalGovernor;
use sharecli_fleet::{
    build_host_agent_forests, format_gate_status_section, format_rss_bytes, gate_status_snapshot,
    scan_host_agents, watch_detected_agents, AgentTreeNode, DetectedAgentWatch,
};
use tokio::time::sleep;

/// One detected agent row for text/JSON surfaces (AC-006.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentProcRow {
    pub pid: u32,
    pub family: String,
    pub comm: String,
    pub mem_rss_bytes: u64,
    pub mem_rss: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fd_count: Option<u64>,
}

/// JSON payload for `sharecli proc --json` and `sharecli status --json` (AC-006.13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentProcSnapshot {
    pub agents: Vec<AgentProcRow>,
    pub scanned: usize,
    pub watched: usize,
    pub gate: sharecli_fleet::GateStatusSnapshot,
}

/// JSON payload for `sharecli proc --tree --json` (AC-006.16).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentTreeSnapshot {
    pub forests: Vec<AgentTreeNodeJson>,
    pub roots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentTreeNodeJson {
    pub pid: u32,
    pub ppid: u32,
    pub comm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    pub children: Vec<AgentTreeNodeJson>,
}

impl AgentProcSnapshot {
    pub fn capture() -> Result<Self> {
        let agents = scan_host_agents();
        let watched = watch_detected_agents(&agents);
        let thermal = ThermalGovernor::new().poll()?;
        let gate = gate_status_snapshot(thermal, agents.len());
        Ok(Self {
            agents: watched.iter().map(agent_row_from_watch).collect(),
            scanned: agents.len(),
            watched: watched.len(),
            gate,
        })
    }
}

fn agent_row_from_watch(row: &DetectedAgentWatch) -> AgentProcRow {
    AgentProcRow {
        pid: row.agent.pid,
        family: row.agent.family.to_string(),
        comm: row.agent.comm.clone(),
        mem_rss_bytes: row.resource.mem_rss_bytes,
        mem_rss: format_rss_bytes(row.resource.mem_rss_bytes),
        fd_count: row.resource.fd_count,
    }
}

fn tree_node_to_json(node: &AgentTreeNode) -> AgentTreeNodeJson {
    AgentTreeNodeJson {
        pid: node.pid,
        ppid: node.ppid,
        comm: node.comm.clone(),
        family: node.family.map(str::to_string),
        children: node.children.iter().map(tree_node_to_json).collect(),
    }
}

fn capture_tree_snapshot() -> AgentTreeSnapshot {
    let forests = build_host_agent_forests();
    let roots = forests.len();
    AgentTreeSnapshot {
        forests: forests.iter().map(tree_node_to_json).collect(),
        roots,
    }
}

/// Render host agent inventory (text mode).
pub fn render_agent_inventory(watched: &[DetectedAgentWatch], scanned: usize) {
    println!("=== Host agents (proc scan) ===\n");
    if watched.is_empty() {
        println!("No known agent processes detected on this host.");
        if scanned > 0 {
            println!("\n({scanned} agent(s) omitted — process exited before resource sample)");
        }
        return;
    }
    println!("{:<8} {:<16} {:<10} {:<8} COMM", "PID", "FAMILY", "RSS", "FD");
    println!("{}", "-".repeat(56));
    for row in watched {
        let fd = row.resource.fd_count.map(|n| n.to_string()).unwrap_or_else(|| "-".into());
        println!(
            "{:<8} {:<16} {:<10} {:<8} {}",
            row.agent.pid,
            row.agent.family,
            format_rss_bytes(row.resource.mem_rss_bytes),
            fd,
            row.agent.comm
        );
    }
    if watched.len() < scanned {
        println!(
            "\n({} agent(s) omitted — process exited before resource sample)",
            scanned - watched.len()
        );
    }
    println!("\nTotal: {} agent process(es)", watched.len());
}

fn render_tree_node(node: &AgentTreeNode, prefix: &str, is_last: bool) {
    let connector = if prefix.is_empty() {
        String::new()
    } else if is_last {
        "└── ".to_string()
    } else {
        "├── ".to_string()
    };
    let family = node
        .family
        .map(|f| format!("{f} "))
        .unwrap_or_else(String::new);
    println!(
        "{prefix}{connector}[{pid}] {family}{comm}",
        pid = node.pid,
        comm = node.comm
    );
    let child_prefix = if prefix.is_empty() {
        String::new()
    } else {
        format!("{}{}", prefix, if is_last { "    " } else { "│   " })
    };
    for (i, child) in node.children.iter().enumerate() {
        render_tree_node(child, &child_prefix, i + 1 == node.children.len());
    }
}

/// Render parent-child agent process forests (text mode, AC-006.16).
pub fn render_agent_tree(forests: &[AgentTreeNode]) {
    println!("=== Agent process tree (proc scan) ===\n");
    if forests.is_empty() {
        println!("No known agent processes detected on this host.");
        return;
    }
    for (i, root) in forests.iter().enumerate() {
        if i > 0 {
            println!();
        }
        render_tree_node(root, "", true);
    }
    println!("\nTotal: {} agent root(s)", forests.len());
}

/// Render one host agent inventory snapshot (text or JSON).
pub fn render_once(json: bool, tree: bool) -> Result<()> {
    let scanned_agents = scan_host_agents();
    let thermal = ThermalGovernor::new().poll()?;
    let gate = gate_status_snapshot(thermal, scanned_agents.len());

    if tree {
        let snap = capture_tree_snapshot();
        if json {
            println!("{}", serde_json::to_string_pretty(&snap)?);
            return Ok(());
        }
        render_agent_tree(&build_host_agent_forests());
        print!("{}", format_gate_status_section(thermal, scanned_agents.len()));
        return Ok(());
    }

    let watched = watch_detected_agents(&scanned_agents);
    if json {
        let snap = AgentProcSnapshot {
            agents: watched.iter().map(agent_row_from_watch).collect(),
            scanned: scanned_agents.len(),
            watched: watched.len(),
            gate,
        };
        println!("{}", serde_json::to_string_pretty(&snap)?);
        return Ok(());
    }
    render_agent_inventory(&watched, scanned_agents.len());
    print!("{}", format_gate_status_section(thermal, scanned_agents.len()));
    Ok(())
}

/// `sharecli proc` — list host-detected agents with live RSS/FD samples.
pub async fn run(json: bool, tree: bool, watch: Option<u64>) -> Result<()> {
    match watch {
        None => render_once(json, tree),
        Some(interval_secs) => {
            if interval_secs == 0 {
                bail!("--watch interval must be >= 1 second");
            }
            loop {
                print!("\x1b[2J\x1b[H");
                render_once(json, tree)?;
                println!("\n[watch] Refreshing every {interval_secs}s — press Ctrl-C to stop.");
                tokio::select! {
                    _ = sleep(Duration::from_secs(interval_secs)) => {},
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nExiting watch mode.");
                        break;
                    }
                }
            }
            Ok(())
        }
    }
}

/// Host inventory from a proc source (used by `ps --all` tests).
#[cfg(test)]
fn host_agent_inventory_from_source(
    source: &dyn sharecli_fleet::ProcSource,
) -> (Vec<DetectedAgentWatch>, usize) {
    let agents = sharecli_fleet::scan_agents(source);
    let watched = watch_detected_agents(&agents);
    (watched, agents.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sharecli_fleet::proc_scan::{DetectedAgent, FakeProcSource, ProcSnapshot};

    #[test]
    fn agent_row_from_watch_formats_rss() {
        let row = agent_row_from_watch(&DetectedAgentWatch {
            agent: DetectedAgent { pid: 42, family: "claude", comm: "claude".into() },
            resource: sharecli_fleet::AgentResourceSample {
                mem_rss_bytes: 52_428_800,
                fd_count: Some(10),
            },
        });
        assert_eq!(row.mem_rss, "50M");
        assert_eq!(row.fd_count, Some(10));
    }

    #[test]
    fn host_inventory_from_fixture() {
        let src = FakeProcSource::new(vec![ProcSnapshot {
            pid: 100,
            ppid: 1,
            comm: "claude".into(),
            cmdline: vec!["claude".into()],
        }]);
        let (watched, scanned) = host_agent_inventory_from_source(&src);
        assert_eq!(scanned, 1);
        assert_eq!(watched.len(), 0, "fixture PID is not live on host");
        let _ = watched;
    }

    #[test]
    fn zero_watch_interval_is_rejected() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        let err = rt
            .block_on(super::run(false, false, Some(0)))
            .expect_err("watch 0 MUST fail");
        assert!(
            err.to_string().contains(">= 1"),
            "error MUST mention minimum interval; got: {err}"
        );
    }

    #[test]
    fn tree_json_from_fixture() {
        let src = FakeProcSource::new(vec![
            ProcSnapshot {
                pid: 1,
                ppid: 0,
                comm: "init".into(),
                cmdline: vec![],
            },
            ProcSnapshot {
                pid: 50,
                ppid: 1,
                comm: "cursor-agent".into(),
                cmdline: vec!["cursor-agent".into()],
            },
            ProcSnapshot {
                pid: 51,
                ppid: 50,
                comm: "node".into(),
                cmdline: vec!["node".into()],
            },
        ]);
        let forests = sharecli_fleet::build_agent_forests(&src);
        let snap = AgentTreeSnapshot {
            forests: forests.iter().map(tree_node_to_json).collect(),
            roots: forests.len(),
        };
        assert_eq!(snap.roots, 1);
        assert_eq!(snap.forests[0].children.len(), 1);
        assert_eq!(snap.forests[0].children[0].pid, 51);
    }
}
