//! FR-006 — `sharecli proc` host agent inventory CLI.

use std::time::Duration;

use anyhow::{bail, Result};
use serde::Serialize;
use sharecli_fleet::thermal::ThermalGovernor;
use sharecli_fleet::{
    format_gate_status_section, format_rss_bytes, gate_status_snapshot, scan_host_agents,
    watch_detected_agents, DetectedAgentWatch,
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

/// Render one host agent inventory snapshot (text or JSON).
pub fn render_once(json: bool) -> Result<()> {
    let scanned_agents = scan_host_agents();
    let watched = watch_detected_agents(&scanned_agents);
    let thermal = ThermalGovernor::new().poll()?;
    let gate = gate_status_snapshot(thermal, scanned_agents.len());
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
pub async fn run(json: bool, watch: Option<u64>) -> Result<()> {
    match watch {
        None => render_once(json),
        Some(interval_secs) => {
            if interval_secs == 0 {
                bail!("--watch interval must be >= 1 second");
            }
            loop {
                print!("\x1b[2J\x1b[H");
                render_once(json)?;
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
        let err = rt.block_on(super::run(false, Some(0))).expect_err("watch 0 MUST fail");
        assert!(
            err.to_string().contains(">= 1"),
            "error MUST mention minimum interval; got: {err}"
        );
    }
}
