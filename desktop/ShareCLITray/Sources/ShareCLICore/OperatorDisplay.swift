/// OperatorDisplay.swift — compact gate / host_watch tray strings (FR-007 / AC-007.56).
///
/// Parity with `sharecli-tray-linux/src/operator_display.rs` golden strings.

import Foundation

public enum OperatorDisplay {
    public static func formatBytesCompact(_ n: UInt64) -> String {
        if n >= 1_048_576 {
            return String(format: "%.1f MB", Double(n) / 1_048_576.0)
        }
        if n >= 1024 {
            return String(format: "%.1f KB", Double(n) / 1024.0)
        }
        return "\(n) B"
    }

    public static func formatGateTrayLine(_ gate: GateStatusSnapshot) -> String {
        "Gate [\(gate.gate_decision)] · \(gate.thermal_pressure) · agents \(gate.detected_agents) · \(gate.agent_contention)"
    }

    public static func formatGateRssTrayLine(_ gate: GateStatusSnapshot) -> String {
        "Agent RSS: \(formatBytesCompact(gate.agent_total_rss_bytes))"
    }

    public static func formatHostWatchTrayLine(_ host: HostResourceWatchJson) -> String {
        String(
            format: "Host load %.2f · FDs %@ · RSS %@",
            host.load_1m,
            String(host.fd_count),
            formatBytesCompact(host.mem_rss_bytes)
        )
    }

    public static func formatHostNetTrayLine(_ host: HostResourceWatchJson) -> String {
        "Net RX \(formatBytesCompact(host.net_rx_bytes)) · TX \(formatBytesCompact(host.net_tx_bytes))"
    }

    public static func formatOperatorTrayLines(
        gate: GateStatusSnapshot,
        host: HostResourceWatchJson
    ) -> [String] {
        [
            formatGateTrayLine(gate),
            formatGateRssTrayLine(gate),
            formatHostWatchTrayLine(host),
            formatHostNetTrayLine(host),
        ]
    }

    public static func formatOperatorStatusSummary(
        gate: GateStatusSnapshot,
        host: HostResourceWatchJson
    ) -> String {
        [
            formatGateTrayLine(gate),
            formatGateRssTrayLine(gate),
            formatHostWatchTrayLine(host),
        ].joined(separator: " | ")
    }
}
