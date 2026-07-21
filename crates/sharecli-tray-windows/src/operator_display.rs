//! Compact gate / host_watch strings for Windows tray operator UI (FR-007 / AC-007.56).
//!
//! Parity with `sharecli-tray-linux/src/operator_display.rs`; integration tests MUST keep
//! Linux/Windows format strings identical.

use crate::ipc::{GateStatusSnapshot, HostResourceWatchJson};

/// Human-readable byte count for tray lines (parity with dashboard `formatBytes`).
pub fn format_bytes_compact(n: u64) -> String {
    if n >= 1_048_576 {
        format!("{:.1} MB", n as f64 / 1_048_576.0)
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} B")
    }
}

/// Primary thermal gate line for tray menus / status bars.
pub fn format_gate_tray_line(gate: &GateStatusSnapshot) -> String {
    format!(
        "Gate [{}] · {} · agents {} · {}",
        gate.gate_decision, gate.thermal_pressure, gate.detected_agents, gate.agent_contention,
    )
}

/// Agent RSS companion line (gate snapshot field).
pub fn format_gate_rss_tray_line(gate: &GateStatusSnapshot) -> String {
    format!("Agent RSS: {}", format_bytes_compact(gate.agent_total_rss_bytes))
}

/// Host load / FD / RSS line for tray menus / status bars.
pub fn format_host_watch_tray_line(host: &HostResourceWatchJson) -> String {
    format!(
        "Host load {:.2} · FDs {} · RSS {}",
        host.load_1m,
        host.fd_count,
        format_bytes_compact(host.mem_rss_bytes),
    )
}

/// Host network RX/TX line for tray menus / status bars.
pub fn format_host_net_tray_line(host: &HostResourceWatchJson) -> String {
    format!(
        "Net RX {} · TX {}",
        format_bytes_compact(host.net_rx_bytes),
        format_bytes_compact(host.net_tx_bytes),
    )
}

/// Ordered tray menu / tooltip lines: gate → host_watch (AC-007.56).
pub fn format_operator_tray_lines(
    gate: &GateStatusSnapshot,
    host: &HostResourceWatchJson,
) -> Vec<String> {
    vec![
        format_gate_tray_line(gate),
        format_gate_rss_tray_line(gate),
        format_host_watch_tray_line(host),
        format_host_net_tray_line(host),
    ]
}

/// Single-line operator summary for compact status bars.
pub fn format_operator_status_summary(
    gate: &GateStatusSnapshot,
    host: &HostResourceWatchJson,
) -> String {
    format!(
        "{} | {} | {}",
        format_gate_tray_line(gate),
        format_gate_rss_tray_line(gate),
        format_host_watch_tray_line(host),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{GateStatusSnapshot, HostResourceWatchJson};

    fn sample_gate() -> GateStatusSnapshot {
        GateStatusSnapshot {
            thermal_pressure: "YELLOW".into(),
            detected_agents: 1,
            agent_total_rss_bytes: 512,
            agent_contention: "WARN".into(),
            gate_decision: "THROTTLE".into(),
        }
    }

    fn sample_host() -> HostResourceWatchJson {
        HostResourceWatchJson {
            fd_count: 10,
            net_rx_bytes: 100,
            net_tx_bytes: 200,
            mem_rss_bytes: 4096,
            load_1m: 1.25,
        }
    }

    #[test]
    fn format_operator_tray_lines_gate_before_host_watch() {
        let lines = format_operator_tray_lines(&sample_gate(), &sample_host());
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("Gate ["));
        assert!(lines[2].starts_with("Host load"));
    }
}
