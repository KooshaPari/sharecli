//! Wire types and mapping helpers for Windows tray `monitoring.report` consume (AC-007.51).
//!
//! Mirrors `sharecli-tray-linux/src/ipc.rs` mapping contract; the WinUI tray uses
//! equivalent C# helpers in `windows/ShareCLITray/MonitoringReportSnapshot.cs`.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessSummary {
    pub pid: u32,
    pub name: String,
    #[allow(dead_code)]
    pub cmd: Vec<String>,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
    #[allow(dead_code)]
    pub start_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GateStatusSnapshot {
    pub thermal_pressure: String,
    pub detected_agents: usize,
    pub agent_total_rss_bytes: u64,
    pub agent_contention: String,
    pub gate_decision: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HostResourceWatchJson {
    pub fd_count: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub mem_rss_bytes: u64,
    pub load_1m: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthSnapshot {
    pub managed_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub healthy: bool,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringProcessEntry {
    pub pid: u32,
    pub name: String,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringReportSnapshot {
    pub timestamp: u64,
    pub total_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub processes: Vec<MonitoringProcessEntry>,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
}

impl MonitoringReportSnapshot {
    /// Map fleet monitoring snapshot → tray health fields (parity with `health.status`).
    pub fn health_snapshot(&self) -> HealthSnapshot {
        HealthSnapshot {
            managed_processes: self.total_processes,
            used_memory_mb: self.used_memory_mb,
            total_memory_mb: self.total_memory_mb,
            healthy: self.used_memory_mb < self.total_memory_mb / 2,
            gate: self.gate.clone(),
            host_watch: self.host_watch.clone(),
        }
    }

    /// Map fleet monitoring processes → tray process rows (parity with `process.list`).
    pub fn process_summaries(&self) -> Vec<ProcessSummary> {
        self.processes
            .iter()
            .map(|p| ProcessSummary {
                pid: p.pid,
                name: p.name.clone(),
                cmd: Vec::new(),
                memory_mb: p.memory_mb,
                project: p.project.clone(),
                harness: p.harness.clone(),
                start_time: 0,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_report_snapshot_matches_server_wire_shape() {
        let raw = r#"{"timestamp":1700000000,"total_processes":1,"used_memory_mb":256,
            "total_memory_mb":16384,"processes":[{"pid":99,"name":"worker","memory_mb":64,
            "project":null,"harness":"native"}],
            "gate":{"thermal_pressure":"GREEN","detected_agents":0,
            "agent_total_rss_bytes":0,"agent_contention":"OK","gate_decision":"ADMIT"},
            "host_watch":{"fd_count":1,"net_rx_bytes":2,"net_tx_bytes":3,
            "mem_rss_bytes":4,"load_1m":0.5}}"#;
        let snap: MonitoringReportSnapshot = serde_json::from_str(raw).unwrap();
        assert_eq!(snap.timestamp, 1_700_000_000);
        assert_eq!(snap.total_processes, 1);
        assert_eq!(snap.gate.gate_decision, "ADMIT");
        assert_eq!(snap.host_watch.load_1m, 0.5);
    }

    #[test]
    fn monitoring_report_snapshot_maps_health_and_processes() {
        let raw = r#"{"timestamp":1700000000,"total_processes":2,"used_memory_mb":256,
            "total_memory_mb":16384,"processes":[{"pid":99,"name":"worker","memory_mb":64,
            "project":"demo","harness":"native"},{"pid":100,"name":"agent","memory_mb":32,
            "project":null,"harness":null}],
            "gate":{"thermal_pressure":"GREEN","detected_agents":0,
            "agent_total_rss_bytes":0,"agent_contention":"OK","gate_decision":"ADMIT"},
            "host_watch":{"fd_count":1,"net_rx_bytes":2,"net_tx_bytes":3,
            "mem_rss_bytes":4,"load_1m":0.5}}"#;
        let snap: MonitoringReportSnapshot = serde_json::from_str(raw).unwrap();
        let health = snap.health_snapshot();
        assert_eq!(health.managed_processes, 2);
        assert_eq!(health.gate.gate_decision, "ADMIT");
        assert_eq!(health.host_watch.load_1m, 0.5);

        let procs = snap.process_summaries();
        assert_eq!(procs.len(), 2);
        assert_eq!(procs[0].pid, 99);
        assert_eq!(procs[0].project.as_deref(), Some("demo"));
    }
}
