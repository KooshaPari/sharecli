//! Pool + proc-scan operator panel shapes (FR-007 / AC-007.69–007.71).
//!
//! Shared field layout for tray, dashboard, and thermal TUI operator surfaces.

/// Runtime pool capacity snapshot for operator panels (parity with `PoolJson` / IPC `PoolSnapshot`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolOperatorPanel {
    pub node_total: usize,
    pub node_idle: usize,
    pub bun_total: usize,
    pub bun_idle: usize,
    pub max_per_type: usize,
    pub healthy: bool,
}

/// Proc-scan / managed-process summary for operator panels (parity with `StatusJson` / IPC `StatusSnapshot`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusOperatorPanel {
    pub scanned: usize,
    pub watched: usize,
    pub total_processes: usize,
    pub agent_rows: usize,
}

/// Tray/dashboard pool operator line (AC-007.69).
pub fn format_pool_operator_line(pool: &PoolOperatorPanel) -> String {
    format!(
        "Pool node {}/{} idle · bun {}/{} idle · max {} · {}",
        pool.node_total,
        pool.node_idle,
        pool.bun_total,
        pool.bun_idle,
        pool.max_per_type,
        if pool.healthy { "healthy" } else { "degraded" },
    )
}

/// Tray/dashboard proc-scan operator line (AC-007.69).
pub fn format_status_operator_line(status: &StatusOperatorPanel) -> String {
    format!(
        "Proc scan {} · watched {} · {} managed · {} agent row(s)",
        status.scanned,
        status.watched,
        status.total_processes,
        status.agent_rows,
    )
}
