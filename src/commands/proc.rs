//! FR-006 — `sharecli proc` host agent inventory CLI.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};
use serde::Serialize;
use sharecli_fleet::thermal::ThermalGovernor;
use sharecli_fleet::{
    build_host_agent_forests, format_gate_status_section, format_rss_bytes, gate_status_snapshot,
    parse_rss_bytes, scan_host_agents, watch_detected_agents, AgentTreeNode, DetectedAgentWatch,
};
use tokio::time::sleep;

/// Inventory filter for `sharecli proc` (AC-006.17).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcFilter {
    pub family: Option<String>,
    pub min_rss_bytes: Option<u64>,
}

impl ProcFilter {
    pub fn from_cli(family: Option<String>, min_rss: Option<String>) -> Result<Self> {
        let min_rss_bytes = match min_rss {
            None => None,
            Some(raw) => Some(parse_rss_bytes(&raw)?),
        };
        Ok(Self { family, min_rss_bytes })
    }

    fn active(&self) -> bool {
        self.family.is_some() || self.min_rss_bytes.is_some()
    }
}

/// Sort key for `sharecli proc` inventory rows (AC-006.19).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcSort {
    /// Ascending PID (lowest first).
    #[default]
    Pid,
    /// Descending resident memory (highest first); PID tie-break ascending.
    Rss,
    /// Descending open FD count (highest first; missing FD treated as 0); PID tie-break.
    Fd,
}

impl ProcSort {
    pub fn from_cli(raw: Option<&str>) -> Result<Option<Self>> {
        match raw {
            None => Ok(None),
            Some(s) => Ok(Some(s.parse()?)),
        }
    }
}

impl std::str::FromStr for ProcSort {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "pid" => Ok(Self::Pid),
            "rss" => Ok(Self::Rss),
            "fd" => Ok(Self::Fd),
            other => bail!("unknown sort key '{other}'; expected 'rss', 'fd', or 'pid'"),
        }
    }
}

/// Order watched agent rows for text/JSON inventory (`--sort`, AC-006.19).
pub fn sort_watched_agents(
    watched: &[DetectedAgentWatch],
    sort: ProcSort,
) -> Vec<DetectedAgentWatch> {
    let mut rows = watched.to_vec();
    match sort {
        ProcSort::Pid => rows.sort_by_key(|row| row.agent.pid),
        ProcSort::Rss => {
            rows.sort_by(|a, b| {
                b.resource
                    .mem_rss_bytes
                    .cmp(&a.resource.mem_rss_bytes)
                    .then_with(|| a.agent.pid.cmp(&b.agent.pid))
            });
        }
        ProcSort::Fd => {
            rows.sort_by(|a, b| {
                let fd_a = a.resource.fd_count.unwrap_or(0);
                let fd_b = b.resource.fd_count.unwrap_or(0);
                fd_b.cmp(&fd_a).then_with(|| a.agent.pid.cmp(&b.agent.pid))
            });
        }
    }
    rows
}

/// Order tree root forests by live RSS/FD/PID samples (`--sort`, AC-006.19).
pub fn sort_agent_forests(
    forests: &[AgentTreeNode],
    sort: ProcSort,
    rss_by_pid: &HashMap<u32, u64>,
    fd_by_pid: &HashMap<u32, u64>,
) -> Vec<AgentTreeNode> {
    let mut roots = forests.to_vec();
    match sort {
        ProcSort::Pid => roots.sort_by_key(|node| node.pid),
        ProcSort::Rss => {
            roots.sort_by(|a, b| {
                let rss_a = rss_by_pid.get(&a.pid).copied().unwrap_or(0);
                let rss_b = rss_by_pid.get(&b.pid).copied().unwrap_or(0);
                rss_b.cmp(&rss_a).then_with(|| a.pid.cmp(&b.pid))
            });
        }
        ProcSort::Fd => {
            roots.sort_by(|a, b| {
                let fd_a = fd_by_pid.get(&a.pid).copied().unwrap_or(0);
                let fd_b = fd_by_pid.get(&b.pid).copied().unwrap_or(0);
                fd_b.cmp(&fd_a).then_with(|| a.pid.cmp(&b.pid))
            });
        }
    }
    roots
}

/// Apply `--family` / `--min-rss` to watched agent rows.
pub fn filter_watched_agents(
    watched: &[DetectedAgentWatch],
    filter: &ProcFilter,
) -> Vec<DetectedAgentWatch> {
    if !filter.active() {
        return watched.to_vec();
    }
    watched.iter().filter(|row| agent_row_matches_filter(row, filter)).cloned().collect()
}

fn agent_row_matches_filter(row: &DetectedAgentWatch, filter: &ProcFilter) -> bool {
    if let Some(ref family) = filter.family {
        if !row.agent.family.eq_ignore_ascii_case(family) {
            return false;
        }
    }
    if let Some(min) = filter.min_rss_bytes {
        if row.resource.mem_rss_bytes < min {
            return false;
        }
    }
    true
}

fn rss_map_from_watched(watched: &[DetectedAgentWatch]) -> HashMap<u32, u64> {
    watched.iter().map(|row| (row.agent.pid, row.resource.mem_rss_bytes)).collect()
}

fn fd_map_from_watched(watched: &[DetectedAgentWatch]) -> HashMap<u32, u64> {
    watched.iter().map(|row| (row.agent.pid, row.resource.fd_count.unwrap_or(0))).collect()
}

fn apply_sort_watched(
    watched: Vec<DetectedAgentWatch>,
    sort: Option<ProcSort>,
) -> Vec<DetectedAgentWatch> {
    match sort {
        Some(key) => sort_watched_agents(&watched, key),
        None => watched,
    }
}

fn apply_sort_forests(
    forests: Vec<AgentTreeNode>,
    sort: Option<ProcSort>,
    rss_by_pid: &HashMap<u32, u64>,
    fd_by_pid: &HashMap<u32, u64>,
) -> Vec<AgentTreeNode> {
    match sort {
        Some(key) => sort_agent_forests(&forests, key, rss_by_pid, fd_by_pid),
        None => forests,
    }
}

/// Apply filters to agent-rooted forests (family on root; min-rss via live samples).
pub fn filter_agent_forests(
    forests: &[AgentTreeNode],
    filter: &ProcFilter,
    rss_by_pid: &HashMap<u32, u64>,
) -> Vec<AgentTreeNode> {
    if !filter.active() {
        return forests.to_vec();
    }
    forests
        .iter()
        .filter(|root| forest_root_matches_filter(root, filter, rss_by_pid))
        .cloned()
        .collect()
}

fn forest_root_matches_filter(
    root: &AgentTreeNode,
    filter: &ProcFilter,
    rss_by_pid: &HashMap<u32, u64>,
) -> bool {
    if let Some(ref family) = filter.family {
        let Some(root_family) = root.family else {
            return false;
        };
        if !root_family.eq_ignore_ascii_case(family) {
            return false;
        }
    }
    if let Some(min) = filter.min_rss_bytes {
        let rss = rss_by_pid.get(&root.pid).copied().unwrap_or(0);
        if rss < min {
            return false;
        }
    }
    true
}

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

/// One NDJSON watch line for flat inventory (`proc --watch --json`, AC-006.18).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentProcNdjsonLine {
    pub ts: u64,
    #[serde(flatten)]
    pub snapshot: AgentProcSnapshot,
}

/// One NDJSON watch line for tree inventory (`proc --tree --watch --json`, AC-006.18).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentTreeNdjsonLine {
    pub ts: u64,
    #[serde(flatten)]
    pub snapshot: AgentTreeSnapshot,
}

fn unix_ts_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
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
    let family = node.family.map(|f| format!("{f} ")).unwrap_or_else(String::new);
    println!("{prefix}{connector}[{pid}] {family}{comm}", pid = node.pid, comm = node.comm);
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
pub fn render_once(
    json: bool,
    tree: bool,
    filter: &ProcFilter,
    ndjson: bool,
    sort: Option<ProcSort>,
) -> Result<()> {
    let scanned_agents = scan_host_agents();
    let thermal = ThermalGovernor::new().poll()?;
    let gate = gate_status_snapshot(thermal, scanned_agents.len());
    let watched_all = watch_detected_agents(&scanned_agents);
    let rss_by_pid = rss_map_from_watched(&watched_all);
    let fd_by_pid = fd_map_from_watched(&watched_all);

    if tree {
        let forests = filter_agent_forests(&build_host_agent_forests(), filter, &rss_by_pid);
        let forests = apply_sort_forests(forests, sort, &rss_by_pid, &fd_by_pid);
        let snap = AgentTreeSnapshot {
            forests: forests.iter().map(tree_node_to_json).collect(),
            roots: forests.len(),
        };
        if json {
            if ndjson {
                let line = AgentTreeNdjsonLine { ts: unix_ts_secs(), snapshot: snap };
                println!("{}", serde_json::to_string(&line)?);
                return Ok(());
            }
            println!("{}", serde_json::to_string_pretty(&snap)?);
            return Ok(());
        }
        render_agent_tree(&forests);
        print!("{}", format_gate_status_section(thermal, scanned_agents.len()));
        return Ok(());
    }

    let watched = apply_sort_watched(filter_watched_agents(&watched_all, filter), sort);
    if json {
        let snap = AgentProcSnapshot {
            agents: watched.iter().map(agent_row_from_watch).collect(),
            scanned: scanned_agents.len(),
            watched: watched.len(),
            gate,
        };
        if ndjson {
            let line = AgentProcNdjsonLine { ts: unix_ts_secs(), snapshot: snap };
            println!("{}", serde_json::to_string(&line)?);
            return Ok(());
        }
        println!("{}", serde_json::to_string_pretty(&snap)?);
        return Ok(());
    }
    render_agent_inventory(&watched, scanned_agents.len());
    print!("{}", format_gate_status_section(thermal, scanned_agents.len()));
    Ok(())
}

/// `sharecli proc` — list host-detected agents with live RSS/FD samples.
pub async fn run(
    json: bool,
    tree: bool,
    watch: Option<u64>,
    family: Option<String>,
    min_rss: Option<String>,
    sort: Option<String>,
) -> Result<()> {
    let filter = ProcFilter::from_cli(family, min_rss)?;
    let sort_key = ProcSort::from_cli(sort.as_deref())?;
    match watch {
        None => render_once(json, tree, &filter, false, sort_key),
        Some(interval_secs) => {
            if interval_secs == 0 {
                bail!("--watch interval must be >= 1 second");
            }
            let ndjson = json;
            loop {
                if !ndjson {
                    print!("\x1b[2J\x1b[H");
                }
                render_once(json, tree, &filter, ndjson, sort_key)?;
                let footer =
                    format!("\n[watch] Refreshing every {interval_secs}s — press Ctrl-C to stop.");
                if ndjson {
                    eprint!("{footer}");
                } else {
                    println!("{footer}");
                }
                tokio::select! {
                    _ = sleep(Duration::from_secs(interval_secs)) => {},
                    _ = tokio::signal::ctrl_c() => {
                        if ndjson {
                            eprintln!("\nExiting watch mode.");
                        } else {
                            println!("\nExiting watch mode.");
                        }
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
            .block_on(super::run(false, false, Some(0), None, None, None))
            .expect_err("watch 0 MUST fail");
        assert!(
            err.to_string().contains(">= 1"),
            "error MUST mention minimum interval; got: {err}"
        );
    }

    #[test]
    fn ndjson_line_includes_ts_and_agents() {
        let line = AgentProcNdjsonLine {
            ts: 1_750_000_000,
            snapshot: AgentProcSnapshot {
                agents: vec![],
                scanned: 0,
                watched: 0,
                gate: sharecli_fleet::GateStatusSnapshot {
                    thermal_pressure: "GREEN".into(),
                    detected_agents: 0,
                    agent_total_rss_bytes: 0,
                    agent_contention: "OK".into(),
                    gate_decision: "ADMIT".into(),
                },
            },
        };
        let json = serde_json::to_string(&line).expect("serialize");
        assert!(json.contains("\"ts\":1750000000"));
        assert!(json.contains("\"agents\":[]"));
    }

    #[test]
    fn tree_json_from_fixture() {
        let src = FakeProcSource::new(vec![
            ProcSnapshot { pid: 1, ppid: 0, comm: "init".into(), cmdline: vec![] },
            ProcSnapshot {
                pid: 50,
                ppid: 1,
                comm: "cursor-agent".into(),
                cmdline: vec!["cursor-agent".into()],
            },
            ProcSnapshot { pid: 51, ppid: 50, comm: "node".into(), cmdline: vec!["node".into()] },
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
