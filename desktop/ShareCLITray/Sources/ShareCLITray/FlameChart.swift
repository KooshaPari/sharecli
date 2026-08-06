// P2-10 — FlameChart.swift
//
// Additive file. Multi-panel flame/spark chart of fleet + host-watch telemetry.
// Reads only real fields from AppState.fleetHistory ([FleetSample]) and
// AppState.hostWatchHistory ([HostWatchSample]) — no hallucinated fields.
//
// Canonical fields (AppState.swift):
//   FleetSample:        timestamp, totalProcesses, totalMemoryMB,
//                       usedMemoryMB, cpuAvgPercent, poolHealthy
//   HostWatchSample:    timestamp, fd_count, net_rx_bytes, net_tx_bytes,
//                       mem_rss_bytes, load_1m
//
// Not wired into TrendsView yet (TrendsView lives inside the 2090-line
// ProcessesPage.swift; this session avoids edits to that file). The view
// is exposed at top level so a future wiring can drop it in with a
// one-line `case .trends: return AnyView(FlameChartView(state: state))`.

import SwiftUI
import Charts
import ShareCLICore

/// Top-level multi-panel flame chart for the fleet + host-watch telemetry
/// rolling windows. Renders the last `Self.maxSamples` samples
/// (default = full history; cap is `AppState.fleetHistoryCap = 60`).
struct FlameChartView: View {
    let state: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                header
                if state.fleetHistory.isEmpty && state.hostWatchHistory.isEmpty {
                    emptyState
                } else {
                    cpuPanel
                    memoryPanel
                    processCountPanel
                    networkPanel
                    loadPanel
                    summaryFooter
                }
            }
            .padding(16)
        }
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: - Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Fleet Flame")
                .font(.title2.weight(.semibold))
            Text("Last \(maxSamples) samples · \(fleetSampleCount) fleet / \(hostSampleCount) host-watch")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Image(systemName: "flame")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No telemetry yet")
                .font(.headline)
            Text("The IPC sidecar will populate this view as monitoring.report snapshots land.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(32)
    }

    // MARK: - CPU

    private var cpuPanel: some View {
        Panel(title: "CPU avg (%)", systemImage: "cpu") {
            Chart(state.fleetHistory) { sample in
                AreaMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("CPU %", sample.cpuAvgPercent)
                )
                .foregroundStyle(LinearGradient(
                    colors: [.orange.opacity(0.5), .orange.opacity(0.05)],
                    startPoint: .top, endPoint: .bottom
                ))
                LineMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("CPU %", sample.cpuAvgPercent)
                )
                .foregroundStyle(.orange)
            }
            .chartYAxisLabel("percent")
            .chartYScale(domain: 0...100)
            .frame(height: 110)
        }
    }

    // MARK: - Memory

    private var memoryPanel: some View {
        Panel(title: "Memory (MB)", systemImage: "memorychip") {
            Chart(state.fleetHistory) { sample in
                LineMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("Total MB", sample.totalMemoryMB)
                )
                .foregroundStyle(.blue)
                .lineStyle(StrokeStyle(lineWidth: 2))
                LineMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("Used MB", sample.usedMemoryMB)
                )
                .foregroundStyle(.purple)
                .lineStyle(StrokeStyle(lineWidth: 2))
            }
            .chartLegend(position: .bottom)
            .frame(height: 110)
        }
    }

    // MARK: - Process count

    private var processCountPanel: some View {
        Panel(title: "Process count", systemImage: "list.bullet") {
            Chart(state.fleetHistory) { sample in
                BarMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("Processes", sample.totalProcesses)
                )
                .foregroundStyle(.green.opacity(0.7))
            }
            .frame(height: 90)
        }
    }

    // MARK: - Network

    private var networkPanel: some View {
        Panel(title: "Network (bytes/s)", systemImage: "network") {
            Chart(networkDeltas) { delta in
                LineMark(
                    x: .value("Time", delta.timestamp),
                    y: .value("rx", delta.rxBytesPerSec)
                )
                .foregroundStyle(.cyan)
                .lineStyle(StrokeStyle(lineWidth: 1.5))
                LineMark(
                    x: .value("Time", delta.timestamp),
                    y: .value("tx", delta.txBytesPerSec)
                )
                .foregroundStyle(.pink)
                .lineStyle(StrokeStyle(lineWidth: 1.5))
            }
            .chartLegend(position: .bottom)
            .frame(height: 110)
        }
    }

    // MARK: - Load

    private var loadPanel: some View {
        Panel(title: "Load average (1m)", systemImage: "speedometer") {
            Chart(state.hostWatchHistory) { sample in
                LineMark(
                    x: .value("Time", sample.timestamp),
                    y: .value("load_1m", sample.load_1m)
                )
                .foregroundStyle(.indigo)
                .lineStyle(StrokeStyle(lineWidth: 2))
            }
            .chartYAxisLabel("load")
            .frame(height: 90)
        }
    }

    // MARK: - Footer

    private var summaryFooter: some View {
        VStack(alignment: .leading, spacing: 4) {
            if let last = state.fleetHistory.last {
                Divider()
                HStack(spacing: 16) {
                    Label("\(last.totalProcesses) procs", systemImage: "list.bullet")
                    Label(String(format: "%.1f%% CPU", last.cpuAvgPercent), systemImage: "cpu")
                    Label("\(last.usedMemoryMB) / \(last.totalMemoryMB) MB", systemImage: "memorychip")
                    if last.poolHealthy {
                        Label("healthy", systemImage: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Label("degraded", systemImage: "exclamationmark.triangle.fill")
                            .foregroundStyle(.orange)
                    }
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }
        }
    }

    // MARK: - Helpers

    private var maxSamples: Int {
        max(state.fleetHistory.count, state.hostWatchHistory.count)
    }

    private var fleetSampleCount: Int { state.fleetHistory.count }
    private var hostSampleCount: Int { state.hostWatchHistory.count }

    /// Compute per-second RX/TX deltas from the host-watch history so the
    /// chart shows throughput rather than absolute cumulative bytes.
    private var networkDeltas: [NetworkDelta] {
        let samples = state.hostWatchHistory
        guard samples.count >= 2 else { return [] }
        var deltas: [NetworkDelta] = []
        for i in 1..<samples.count {
            let prev = samples[i - 1]
            let cur = samples[i]
            let dt = cur.timestamp.timeIntervalSince(prev.timestamp)
            guard dt > 0 else { continue }
            let rxDelta = cur.net_rx_bytes >= prev.net_rx_bytes
                ? cur.net_rx_bytes - prev.net_rx_bytes
                : 0
            let txDelta = cur.net_tx_bytes >= prev.net_tx_bytes
                ? cur.net_tx_bytes - prev.net_tx_bytes
                : 0
            deltas.append(NetworkDelta(
                timestamp: cur.timestamp,
                rxBytesPerSec: Double(rxDelta) / dt,
                txBytesPerSec: Double(txDelta) / dt
            ))
        }
        return deltas
    }
}

// MARK: - Supporting types

private struct NetworkDelta: Identifiable, Hashable {
    let timestamp: Date
    let rxBytesPerSec: Double
    let txBytesPerSec: Double
    var id: Date { timestamp }
}

private struct Panel<Content: View>: View {
    let title: String
    let systemImage: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Label(title, systemImage: systemImage)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)
            content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
    }
}
