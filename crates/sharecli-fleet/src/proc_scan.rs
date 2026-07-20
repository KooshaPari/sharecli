//! Live process-tree agent detection (FR-006).
//!
//! Walks parent chains (`/proc` on Linux; `sysinfo` elsewhere) and matches
//! [`crate::detect::match_known_agent`] — observation only, no vendor-bin wrap.

use crate::detect::match_known_agent;
use std::collections::HashMap;

/// One process row used for tree walks and scans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcSnapshot {
    pub pid: u32,
    pub ppid: u32,
    pub comm: String,
    pub cmdline: Vec<String>,
}

/// A detected agent instance on the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedAgent {
    pub pid: u32,
    pub family: &'static str,
    pub comm: String,
}

/// Abstract process table — production backends + in-memory fixtures for tests.
pub trait ProcSource {
    fn list(&self) -> Vec<ProcSnapshot>;
}

/// In-memory process table for unit tests (FR fixtures).
#[derive(Debug, Default, Clone)]
pub struct FakeProcSource {
    procs: Vec<ProcSnapshot>,
}

impl FakeProcSource {
    pub fn new(procs: Vec<ProcSnapshot>) -> Self {
        Self { procs }
    }
}

impl ProcSource for FakeProcSource {
    fn list(&self) -> Vec<ProcSnapshot> {
        self.procs.clone()
    }
}

/// Scan every process; return those whose `comm`/cmdline match known agents.
pub fn scan_agents(source: &dyn ProcSource) -> Vec<DetectedAgent> {
    let mut out = Vec::new();
    for p in source.list() {
        if let Some(family) = match_known_agent(&p.comm, &p.cmdline) {
            out.push(DetectedAgent {
                pid: p.pid,
                family,
                comm: p.comm,
            });
        }
    }
    out.sort_by_key(|a| a.pid);
    out
}

/// Walk `pid` and its ancestors via `ppid` until a known agent is found.
pub fn walk_agent_ancestors(source: &dyn ProcSource, pid: u32) -> Option<DetectedAgent> {
    let by_pid: HashMap<u32, ProcSnapshot> =
        source.list().into_iter().map(|p| (p.pid, p)).collect();
    let mut seen = std::collections::HashSet::new();
    let mut cur = pid;
    while seen.insert(cur) {
        let Some(proc) = by_pid.get(&cur) else {
            break;
        };
        if let Some(family) = match_known_agent(&proc.comm, &proc.cmdline) {
            return Some(DetectedAgent {
                pid: proc.pid,
                family,
                comm: proc.comm.clone(),
            });
        }
        if proc.ppid == 0 || proc.ppid == proc.pid {
            break;
        }
        cur = proc.ppid;
    }
    None
}

/// True when `pid` itself or any ancestor matches a known agent family.
pub fn is_under_agent(source: &dyn ProcSource, pid: u32) -> bool {
    walk_agent_ancestors(source, pid).is_some()
}

/// Table cell for the `sharecli ps` AGENT column: nearest ancestor family or `"-"`.
pub fn agent_label_for_pid(source: &dyn ProcSource, pid: u32) -> &'static str {
    walk_agent_ancestors(source, pid)
        .map(|a| a.family)
        .unwrap_or("-")
}

/// Live host process source.
#[derive(Debug, Default, Clone, Copy)]
pub struct HostProcSource;

impl ProcSource for HostProcSource {
    fn list(&self) -> Vec<ProcSnapshot> {
        #[cfg(target_os = "linux")]
        {
            list_linux_proc()
        }
        #[cfg(all(unix, not(target_os = "linux")))]
        {
            list_sysinfo_proc()
        }
        #[cfg(not(unix))]
        {
            list_sysinfo_proc()
        }
    }
}

/// Convenience: scan the live host for known agents.
pub fn scan_host_agents() -> Vec<DetectedAgent> {
    scan_agents(&HostProcSource)
}

/// Nearest known-agent ancestor for the current process (hypervisor spawn context).
pub fn detect_caller_agent() -> Option<DetectedAgent> {
    walk_agent_ancestors(&HostProcSource, std::process::id())
}

#[cfg(target_os = "linux")]
fn list_linux_proc() -> Vec<ProcSnapshot> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in entries.flatten() {
        let name = ent.file_name();
        let Some(pid_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let base = ent.path();
        let comm = std::fs::read_to_string(base.join("comm"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let cmdline_raw = std::fs::read(base.join("cmdline")).unwrap_or_default();
        let cmdline = parse_cmdline(&cmdline_raw);
        let ppid = read_ppid(&base.join("status")).unwrap_or(0);
        out.push(ProcSnapshot {
            pid,
            ppid,
            comm,
            cmdline,
        });
    }
    out
}

#[cfg(target_os = "linux")]
fn parse_cmdline(raw: &[u8]) -> Vec<String> {
    raw.split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect()
}

#[cfg(target_os = "linux")]
fn read_ppid(status: &std::path::Path) -> Option<u32> {
    let text = std::fs::read_to_string(status).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("PPid:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

#[cfg(any(all(unix, not(target_os = "linux")), not(unix)))]
fn list_sysinfo_proc() -> Vec<ProcSnapshot> {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut out = Vec::new();
    for (pid, proc) in sys.processes() {
        let pid_u = pid.as_u32();
        let ppid = proc.parent().map(|p| p.as_u32()).unwrap_or(0);
        let comm = proc.name().to_string_lossy().into_owned();
        let cmdline: Vec<String> = proc
            .cmd()
            .iter()
            .map(|c| c.to_string_lossy().into_owned())
            .collect();
        out.push(ProcSnapshot {
            pid: pid_u,
            ppid,
            comm,
            cmdline,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> FakeProcSource {
        FakeProcSource::new(vec![
            ProcSnapshot {
                pid: 1,
                ppid: 0,
                comm: "init".into(),
                cmdline: vec![],
            },
            ProcSnapshot {
                pid: 10,
                ppid: 1,
                comm: "launchd".into(),
                cmdline: vec![],
            },
            ProcSnapshot {
                pid: 100,
                ppid: 10,
                comm: "claude".into(),
                cmdline: vec!["/usr/local/bin/claude".into()],
            },
            ProcSnapshot {
                pid: 200,
                ppid: 100,
                comm: "bash".into(),
                cmdline: vec!["bash".into()],
            },
            ProcSnapshot {
                pid: 300,
                ppid: 200,
                comm: "ruff".into(),
                cmdline: vec!["ruff".into(), "check".into(), ".".into()],
            },
            ProcSnapshot {
                pid: 400,
                ppid: 10,
                comm: "zsh".into(),
                cmdline: vec!["-l".into()],
            },
        ])
    }

    #[test]
    fn scan_finds_claude_only() {
        let agents = scan_agents(&tree());
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].family, "claude");
        assert_eq!(agents[0].pid, 100);
    }

    #[test]
    fn walk_from_ruff_finds_claude_ancestor() {
        let hit = walk_agent_ancestors(&tree(), 300).expect("under claude");
        assert_eq!(hit.family, "claude");
        assert_eq!(hit.pid, 100);
        assert!(is_under_agent(&tree(), 300));
        assert!(is_under_agent(&tree(), 200));
        assert!(!is_under_agent(&tree(), 400), "human zsh MUST not be agent path");
    }

    #[test]
    fn walk_self_agent() {
        let hit = walk_agent_ancestors(&tree(), 100).expect("self");
        assert_eq!(hit.pid, 100);
    }

    #[test]
    fn agent_label_for_child_and_human() {
        assert_eq!(agent_label_for_pid(&tree(), 300), "claude");
        assert_eq!(agent_label_for_pid(&tree(), 400), "-");
    }
}
