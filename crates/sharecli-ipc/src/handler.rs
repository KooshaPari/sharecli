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
use sharecli_fleet::{count_host_agents, gate_status_snapshot, GateStatusSnapshot};
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

/// IPC `health.status` envelope (FR-007 / AC-007.45).
///
/// Runtime health fields precede live `gate` and `host_watch` siblings
/// (parity with `health --json` AC-007.44).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HealthSnapshot {
    pub managed_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub healthy: bool,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MonitoringProcessEntry {
    pub pid: u32,
    pub name: String,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
}

/// IPC `pool.status` envelope (FR-007 / AC-007.67).
///
/// Pool status fields precede live `gate` and `host_watch` siblings
/// (parity with `pool --json` AC-007.44).
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
}

/// IPC `status.snapshot` envelope (FR-007 / AC-007.67).
///
/// Status fields precede live `gate` and `host_watch` siblings
/// (parity with `status --json` AC-007.25).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusSnapshot {
    pub total_processes: usize,
    pub agents: Vec<AgentProcRow>,
    pub scanned: usize,
    pub watched: usize,
    pub gate: GateStatusSnapshot,
    pub host_watch: HostResourceWatchJson,
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
                let snap = HealthSnapshot {
                    managed_processes: procs.len(),
                    used_memory_mb: used,
                    total_memory_mb: total,
                    healthy: used < total / 2,
                    gate,
                    host_watch,
                };
                Ok(serde_json::to_value(snap)?)
            }

            "pool.status" => {
                let runtime = shared_runtime();
                let status = runtime.status().await;
                let health = runtime.health_check().await;
                let (gate, host_watch) = capture_gate_host_watch()?;
                let snap = PoolSnapshot {
                    node_total: status.node_total,
                    node_idle: status.node_idle,
                    bun_total: status.bun_total,
                    bun_idle: status.bun_idle,
                    max_per_type: status.max_per_type,
                    healthy: health.healthy,
                    issues: health.issues,
                    gate,
                    host_watch,
                };
                Ok(serde_json::to_value(snap)?)
            }

            "status.snapshot" => {
                self.pool.refresh().await;
                let procs = self.pool.list().await;
                let snapshot = AgentProcSnapshot::capture()?;
                let snap = StatusSnapshot {
                    total_processes: procs.len(),
                    agents: snapshot.agents,
                    scanned: snapshot.scanned,
                    watched: snapshot.watched,
                    gate: snapshot.gate,
                    host_watch: snapshot.host_watch,
                };
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

            "monitoring.report" => {
                self.pool.refresh().await;
                let procs = self.pool.list().await;
                let (used, total) = self.pool.system_memory_usage().await;
                let (gate, host_watch) = capture_gate_host_watch()?;
                let runtime = shared_runtime();
                let pool_status = runtime.status().await;
                let pool_health = runtime.health_check().await;
                let pool = PoolSnapshot {
                    node_total: pool_status.node_total,
                    node_idle: pool_status.node_idle,
                    bun_total: pool_status.bun_total,
                    bun_idle: pool_status.bun_idle,
                    max_per_type: pool_status.max_per_type,
                    healthy: pool_health.healthy,
                    issues: pool_health.issues,
                    gate: gate.clone(),
                    host_watch: host_watch.clone(),
                };
                let agent_snap = AgentProcSnapshot::capture()?;
                let status = StatusSnapshot {
                    total_processes: procs.len(),
                    agents: agent_snap.agents,
                    scanned: agent_snap.scanned,
                    watched: agent_snap.watched,
                    gate: agent_snap.gate,
                    host_watch: agent_snap.host_watch,
                };
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
