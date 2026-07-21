/// OperatorDisplay.swift — compact gate / host_watch tray strings + thermal visuals
/// (FR-007 / AC-007.56 text, AC-007.57 icon/badge/color).
///
/// Parity with `sharecli-tray-linux/src/operator_display.rs` golden strings.

import Foundation
import SwiftUI

public enum TrayGateSeverity: String, Equatable {
    case normal
    case warning
    case critical
    case offline
}

public struct TrayGateVisual: Equatable {
    public let severity: TrayGateSeverity
    public let decisionClass: String
    public let thermalClass: String
    public let colorHex: String
    public let badgeLabel: String
    public let swiftSymbolName: String

    public var swiftColor: Color {
        switch severity {
        case .normal: return Color(red: 0.247, green: 0.725, blue: 0.314) // #3fb950
        case .warning: return Color(red: 0.824, green: 0.600, blue: 0.133) // #d29922
        case .critical: return Color(red: 0.973, green: 0.318, blue: 0.286) // #f85149
        case .offline: return Color(red: 0.824, green: 0.600, blue: 0.133)
        }
    }
}

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

    /// Compact menu bar title for NSStatusItem (AC-007.62).
    public static func formatMenuBarTitleLine(visual: TrayGateVisual, health: HealthSnapshot) -> String {
        " \(visual.badgeLabel) · \(health.managed_processes) | \(health.used_memory_mb)M"
    }

    /// Offline menu bar title when monitoring.report is unavailable (AC-007.62).
    public static func formatMenuBarTitleOfflineLine(visual: TrayGateVisual) -> String {
        " \(visual.badgeLabel) · offline"
    }

    /// Pool capacity line from `pool.status` IPC (AC-007.69).
    public static func formatPoolTrayLine(_ pool: PoolSnapshot) -> String {
        let healthLabel = pool.healthy ? "healthy" : "degraded"
        return "Pool node \(pool.node_total)/\(pool.node_idle) idle · bun \(pool.bun_total)/\(pool.bun_idle) idle · max \(pool.max_per_type) · \(healthLabel)"
    }

    /// Proc-scan summary line from `status.snapshot` IPC (AC-007.69).
    public static func formatStatusSnapshotTrayLine(_ status: StatusSnapshot) -> String {
        "Proc scan \(status.scanned) · watched \(status.watched) · \(status.total_processes) managed · \(status.agents.count) agent row(s)"
    }

    /// Supplementary pool + status operator lines (AC-007.69).
    public static func formatPoolStatusOperatorLines(
        pool: PoolSnapshot,
        status: StatusSnapshot
    ) -> [String] {
        [formatPoolTrayLine(pool), formatStatusSnapshotTrayLine(status)]
    }

    public static func resolveGateDecisionClass(_ gateDecision: String) -> String {
        switch gateDecision {
        case "ADMIT": return "gate-admit"
        case "DENY": return "gate-deny"
        default: return "gate-unavailable"
        }
    }

    public static func resolveThermalClass(_ thermalPressure: String) -> String {
        switch thermalPressure {
        case "GREEN": return ""
        case "YELLOW": return "warning"
        case "RED": return "critical"
        default: return "warning"
        }
    }

    public static func resolveTrayGateVisual(
        thermalPressure: String,
        gateDecision: String,
        connected: Bool
    ) -> TrayGateVisual {
        if !connected {
            return TrayGateVisual(
                severity: .offline,
                decisionClass: "gate-unavailable",
                thermalClass: "warning",
                colorHex: "#d29922",
                badgeLabel: "Offline",
                swiftSymbolName: "wifi.slash"
            )
        }

        let decisionClass = resolveGateDecisionClass(gateDecision)
        let thermalClass = resolveThermalClass(thermalPressure)

        let severity: TrayGateSeverity
        if gateDecision == "DENY" || thermalPressure == "RED" {
            severity = .critical
        } else if gateDecision == "THROTTLE" || thermalPressure == "YELLOW"
            || thermalPressure == "UNAVAILABLE" || decisionClass == "gate-unavailable"
        {
            severity = .warning
        } else {
            severity = .normal
        }

        switch severity {
        case .critical:
            return TrayGateVisual(
                severity: .critical,
                decisionClass: decisionClass,
                thermalClass: thermalClass,
                colorHex: "#f85149",
                badgeLabel: "Critical",
                swiftSymbolName: "flame.fill"
            )
        case .warning:
            let label = thermalPressure == "UNAVAILABLE" ? "Unavailable" : "Warning"
            return TrayGateVisual(
                severity: .warning,
                decisionClass: decisionClass,
                thermalClass: thermalClass,
                colorHex: "#d29922",
                badgeLabel: label,
                swiftSymbolName: "exclamationmark.triangle.fill"
            )
        case .normal:
            return TrayGateVisual(
                severity: .normal,
                decisionClass: decisionClass,
                thermalClass: thermalClass,
                colorHex: "#3fb950",
                badgeLabel: "Normal",
                swiftSymbolName: "cpu"
            )
        case .offline:
            fatalError("unreachable")
        }
    }

    public static func resolveTrayGateVisual(gate: GateStatusSnapshot, connected: Bool) -> TrayGateVisual {
        resolveTrayGateVisual(
            thermalPressure: gate.thermal_pressure,
            gateDecision: gate.gate_decision,
            connected: connected
        )
    }
}
