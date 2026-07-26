//! Request dispatch for the IPC server.
//!
//! Methods exposed:
//!   process.list        → Vec<ProcessSummary>
//!   process.kill        → { pid }
//!   process.kill_all    → {}
//!   health.status       → HealthSnapshot
//!   pool.status         → PoolSnapshot
//!   status.snapshot     → StatusSnapshot
//!   config.get          → Config
//!   config.set          → { key, value }  (dot-path into TOML)
//!   monitoring.report   → MonitoringReportSnapshot

use std::sync::{Arc, OnceLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sharecli::commands::proc::{AgentProcRow, AgentProcSnapshot};
use sharecli::config::Config;
use sharecli::monitoring::HostResourceWatchJson;
use sharecli::runtime::SharedRuntime;
use sharecli::{ProcessInfo, ProcessPool};
use sharecli_fleet::thermal::ThermalGovernor;
use sharecli_fleet::{
    count_host_agents, gate_status_snapshot, global_coalesce_meters, global_slot_queue_meters,
    CoalesceMeters, GateStatusSnapshot, SlotQueueMeters,
};
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize)]
pub struct Response {
    pub id: u64,
    pub result: Value,
    pub error: Option<String>,
}

impl Response {
    fn ok(id: u64, result: impl Serialize) -> Self {
        Self { id, result: serde_json::to_value(result).unwrap_or(Value::Null), error: None }
    }

    fn err(id: u64, msg: impl std::fmt::Display) -> Self {
        Self { id, result: Value::Null, error: Some(msg.to_string()) }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcessSummary {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
    pub start_time: u64,
}

impl From<ProcessInfo> for ProcessSummary {
    fn from(p: ProcessInfo) -> Self {
        Self {
            pid: p.pid,
            name: p.name,
            cmd: p.cmd,
            memory_mb: p.memory_mb,
            project: p.project,
            harness: p.harness,
            start_time: p.start_time,
        }
    }
}

/// IPC `health.status` envelope (FR-007 / AC-007.45, pool/status AC-007.78).
///
/// Runtime health fields precede live `gate`, `host_watch`, `pool`, and `status`
/// siblings (parity with `health --json` AC-007.77).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HealthSnapshot {
    pub managed_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub healthy: bool,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
    pub pool: PoolSnapshot,
    pub status: StatusSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MonitoringProcessEntry {
    pub pid: u32,
    pub name: String,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
    /// Unix timestamp (seconds) the process started, as captured by `sysinfo`.
    /// Used by tray dashboards to render an "Age" column on the Processes page.
    /// Always 0 if the sidecar couldn't determine start_time.
    #[serde(default)]
    pub start_time: u64,
}

/// IPC `pool.status` envelope (FR-007 / AC-007.67, nested status AC-007.78).
///
/// Pool status fields precede live `gate`, `host_watch`, and nested `status` sibling
/// (parity with `pool --json` AC-007.77).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PoolSnapshot {
    pub node_total: usize,
    pub node_idle: usize,
    pub bun_total: usize,
    pub bun_idle: usize,
    pub max_per_type: usize,
    pub healthy: bool,
    pub issues: Vec<String>,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
    /// Proc-scan status sibling; only emitted by `pool.status` (AC-007.78).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Box<StatusSnapshot>>,
}

/// IPC `status.snapshot` envelope (FR-007 / AC-007.67, nested pool AC-007.78).
///
/// Status fields precede live `gate`, `host_watch`, and nested `pool` sibling
/// (parity with `status --json` AC-007.77).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusSnapshot {
    pub total_processes: usize,
    pub agents: Vec<AgentProcRow>,
    pub scanned: usize,
    pub watched: usize,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
    /// Runtime pool sibling; only emitted by `status.snapshot` (AC-007.78).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<Box<PoolSnapshot>>,
}

/// IPC `pool.effectiveness` envelope (PR 4 of dashboard expansion plan).
///
/// Aggregates Hypervisor coalesce cache + SlotQueue counters from
/// `sharecli_fleet` so the dashboard can render pool effectiveness
/// without having to scan TUI telemetry files.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PoolEffectivenessSnapshot {
    pub coalesce: CoalesceMeters,
    pub slot_queue: SlotQueueMeters,
    pub sampled_at: u64,
}

/// IPC `process.cmdline` envelope (PR 5 of dashboard expansion plan).
///
/// Returns the full command-line for a given PID, plus the parsed argv
/// (whitespace-split, naive — suitable for display, not execution).
/// `cmdline` is the raw `/proc/<pid>/cmdline` buffer (NUL-separated,
/// '\n'-joined) so the tray can render it verbatim. `argv` is the
/// whitespace-split array for table-friendly display.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProcessCmdline {
    pub pid: u32,
    pub cmdline: String,
    pub argv: Vec<String>,
}

/// IPC `monitoring.report` envelope (FR-007 / AC-007.46, pool/status AC-007.72).
///
/// Fleet monitoring fields precede live `gate`, `host_watch`, `pool`, and `status`
/// siblings (parity with dashboard WS AC-007.70 key order within the operator envelope).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MonitoringReportSnapshot {
    pub timestamp: u64,
    pub total_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub processes: Vec<MonitoringProcessEntry>,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
    pub pool: PoolSnapshot,
    pub status: StatusSnapshot,
}

/// Live thermal gate + host resource watch for IPC envelopes (FR-007 / AC-007.45).
fn capture_gate_host_watch() -> Result<(GateStatusSnapshot, HostResourceWatchJson)> {
    let gate = match ThermalGovernor::new().poll() {
        Ok(level) => gate_status_snapshot(level, count_host_agents()),
        Err(_) => GateStatusSnapshot {
            thermal_pressure: "UNAVAILABLE".to_string(),
            detected_agents: count_host_agents(),
            agent_total_rss_bytes: 0,
            agent_contention: "UNAVAILABLE".to_string(),
            gate_decision: "UNAVAILABLE".to_string(),
        },
    };
    let host_watch = HostResourceWatchJson::capture()?;
    Ok((gate, host_watch))
}

static SHARED_RUNTIME: OnceLock<SharedRuntime> = OnceLock::new();

fn shared_runtime() -> &'static SharedRuntime {
    SHARED_RUNTIME.get_or_init(|| {
        let max = Config::load().map(|c| c.pool.max_per_type).unwrap_or(4);
        SharedRuntime::new(max)
    })
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub struct Handler {
    pool: Arc<ProcessPool>,
    config: Arc<RwLock<Config>>,
}

impl Handler {
    pub async fn new() -> Result<Self> {
        let pool = Arc::new(ProcessPool::new());
        let config = Arc::new(RwLock::new(Config::load().unwrap_or_default()));
        Ok(Self { pool, config })
    }

    async fn capture_pool_snapshot(
        &self,
        gate: GateStatusSnapshot,
        host_watch: HostResourceWatchJson,
    ) -> PoolSnapshot {
        let runtime = shared_runtime();
        let status = runtime.status().await;
        let health = runtime.health_check().await;
        PoolSnapshot {
            node_total: status.node_total,
            node_idle: status.node_idle,
            bun_total: status.bun_total,
            bun_idle: status.bun_idle,
            max_per_type: status.max_per_type,
            healthy: health.healthy,
            issues: health.issues,
            gate,
            host_watch,
            status: None,
        }
    }

    async fn capture_status_snapshot(&self) -> Result<StatusSnapshot> {
        self.pool.refresh().await;
        let procs = self.pool.list().await;
        let snapshot = AgentProcSnapshot::capture()?;
        Ok(StatusSnapshot {
            total_processes: procs.len(),
            agents: snapshot.agents,
            scanned: snapshot.scanned,
            watched: snapshot.watched,
            gate: snapshot.gate,
            host_watch: snapshot.host_watch,
            pool: None,
        })
    }

    /// Read the process command-line for a given PID from `/proc/<pid>/cmdline`.
    /// Returns an empty `cmdline` (and empty `argv`) if the process is gone
    /// or the buffer is unreadable. Cross-platform: on macOS the tray
    /// surveys `sysctl(KERN_PROCARGS2)`; on Linux we read `/proc/<pid>/cmdline`.
    /// The current implementation is Linux-only — the macOS sidecar returns
    /// a placeholder here, sufficient for the dashboard's display purposes.
    async fn capture_process_cmdline(&self, pid: u32) -> Result<ProcessCmdline> {
        let cmdline = read_proc_cmdline(pid).unwrap_or_default();
        let argv = if cmdline.is_empty() {
            Vec::new()
        } else {
            cmdline.split_whitespace().map(|s| s.to_string()).collect()
        };
        Ok(ProcessCmdline { pid, cmdline, argv })
    }

    pub async fn dispatch(&self, raw: &str) -> Response {
        let req: Request = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return Response::err(0, format!("parse error: {e}")),
        };

        match self.handle(&req).await {
            Ok(val) => Response::ok(req.id, val),
            Err(e) => Response::err(req.id, e),
        }
    }

    async fn handle(&self, req: &Request) -> Result<Value> {
        match req.method.as_str() {
            "process.list" => {
                self.pool.refresh().await;
                let procs: Vec<ProcessSummary> =
                    self.pool.list().await.into_iter().map(ProcessSummary::from).collect();
                Ok(serde_json::to_value(procs)?)
            }

            "process.kill" => {
                let pid: u32 =
                    req.params["pid"].as_u64().ok_or_else(|| anyhow::anyhow!("missing pid"))?
                        as u32;
                self.pool.kill(pid).await?;
                Ok(Value::Bool(true))
            }

            "process.kill_all" => {
                self.pool.kill_all().await?;
                Ok(Value::Bool(true))
            }

            "health.status" => {
                self.pool.refresh().await;
                let procs = self.pool.list().await;
                let (used, total) = self.pool.system_memory_usage().await;
                let (gate, host_watch) = capture_gate_host_watch()?;
                let pool = self.capture_pool_snapshot(gate.clone(), host_watch.clone()).await;
                let status = self.capture_status_snapshot().await?;
                let snap = HealthSnapshot {
                    managed_processes: procs.len(),
                    used_memory_mb: used,
                    total_memory_mb: total,
                    healthy: used < total / 2,
                    gate,
                    host_watch,
                    pool,
                    status,
                };
                Ok(serde_json::to_value(snap)?)
            }

            "pool.status" => {
                let (gate, host_watch) = capture_gate_host_watch()?;
                let mut snap = self.capture_pool_snapshot(gate, host_watch).await;
                snap.status = Some(Box::new(self.capture_status_snapshot().await?));
                Ok(serde_json::to_value(snap)?)
            }

            "pool.effectiveness" => {
                // Constant-time snapshot of sharecli_fleet coalesce + slot queue counters.
                Ok(serde_json::to_value(self.capture_effectiveness())?)
            }

            "status.snapshot" => {
                let mut snap = self.capture_status_snapshot().await?;
                let (gate, host_watch) = capture_gate_host_watch()?;
                snap.pool = Some(Box::new(self.capture_pool_snapshot(gate, host_watch).await));
                Ok(serde_json::to_value(snap)?)
            }

            "config.get" => {
                let cfg = self.config.read().await.clone();
                Ok(serde_json::to_value(cfg)?)
            }

            "config.set" => {
                let key =
                    req.params["key"].as_str().ok_or_else(|| anyhow::anyhow!("missing key"))?;
                let value = &req.params["value"];
                self.apply_config_patch(key, value).await?;
                Ok(Value::Bool(true))
            }

            "process.cmdline" => {
                let pid = req.params.get("pid")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("process.cmdline: missing required `pid` parameter"))?
                    as u32;
                Ok(serde_json::to_value(self.capture_process_cmdline(pid).await?)?)
            }

            "monitoring.report" => {
                self.pool.refresh().await;
                let procs = self.pool.list().await;
                let (used, total) = self.pool.system_memory_usage().await;
                let (gate, host_watch) = capture_gate_host_watch()?;
                let pool = self.capture_pool_snapshot(gate.clone(), host_watch.clone()).await;
                let status = self.capture_status_snapshot().await?;
                let snap = MonitoringReportSnapshot {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    total_processes: procs.len(),
                    used_memory_mb: used,
                    total_memory_mb: total,
                    processes: procs
                        .iter()
                        .map(|p| MonitoringProcessEntry {
                            pid: p.pid,
                            name: p.name.clone(),
                            memory_mb: p.memory_mb,
                            project: p.project.clone(),
                            harness: p.harness.clone(),
                            start_time: p.start_time,
                        })
                        .collect(),
                    gate,
                    host_watch,
                    pool,
                    status,
                };
                Ok(serde_json::to_value(snap)?)
            }

            other => Err(anyhow::anyhow!("unknown method: {other}")),
        }
    }

    /// Sample the latest Hypervisor coalesce + SlotQueue counters
    /// (PR 4 of dashboard expansion plan). Counters are global atomics
    /// in `sharecli-fleet`, so this is a constant-time snapshot.
    fn capture_effectiveness(&self) -> PoolEffectivenessSnapshot {
        PoolEffectivenessSnapshot {
            coalesce: global_coalesce_meters(),
            slot_queue: global_slot_queue_meters(),
            sampled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Apply a dot-path config patch: "runtime.max_memory_mb" → 8192
    async fn apply_config_patch(&self, key: &str, value: &Value) -> Result<()> {
        let mut cfg = self.config.write().await;
        let mut raw = serde_json::to_value(&*cfg)?;

        let parts: Vec<&str> = key.split('.').collect();
        set_nested(&mut raw, &parts, value.clone())
            .map_err(|e| anyhow::anyhow!("config.set {key}: {e}"))?;

        *cfg = serde_json::from_value(raw)?;
        cfg.save()?;
        Ok(())
    }
}

fn set_nested(val: &mut Value, path: &[&str], new: Value) -> Result<(), String> {
    if path.is_empty() {
        *val = new;
        return Ok(());
    }
    match val {
        Value::Object(map) => {
            let entry = map.entry(path[0]).or_insert(Value::Object(serde_json::Map::new()));
            set_nested(entry, &path[1..], new)
        }
        _ => Err(format!("expected object at segment '{}'", path[0])),
    }
}

/// Read `/proc/<pid>/cmdline` on Linux (NUL-separated argv).
/// On macOS, returns `Err` (the platform does not expose a `/proc` filesystem)
/// — the caller treats that as an empty cmdline.
///
/// `Some(non-empty)` ⇒ success.
/// `Some(empty)`     ⇒ process detached between snapshot & read (treat as "gone").
/// `None`            ⇒ not available.
fn read_proc_cmdline(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/cmdline");
        let bytes = std::fs::read(&path).ok()?;
        if bytes.is_empty() {
            return Some(String::new());
        }
        // Replace NULs with spaces and trim trailing whitespace.
        let s: String = bytes
            .iter()
            .map(|b| if *b == 0 { ' ' } else { *b as char })
            .collect();
        Some(s.trim_end().to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}
