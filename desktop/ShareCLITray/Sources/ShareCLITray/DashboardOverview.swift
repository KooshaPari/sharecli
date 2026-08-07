/// DashboardOverview.swift — additive summary view (q-dash-2).
///
/// Composes the existing dashboard surfaces (fleet + host-watch telemetry
/// + process inventory) into a single dashboard grid. This is a zero-risk
/// additive file: it does not modify any existing view, only composes
/// data already published by `AppState` (`fleetHistory`,
/// `hostWatchHistory`, `processes`, `poolStatus`).
///
/// Structure:
///   ┌────────────────────────────────────────────────────────────────┐
///   │  Overview header  (pool health + latest sample timestamp)      │
///   ├────────────────────────────────────────────────────────────────┤
///   │  Fleet section     [processes] [total MB] [used MB] [cpu%]     │
///   │  Systems section   [load avg] [fd count] [net rx/tx] [rss]     │
///   │  Activity section  [top procs] [projects] [harnesses] [spawns] │
///   └────────────────────────────────────────────────────────────────┘
///
/// Status pills (`HealthPill`) are shared with ProcessesPage.swift
/// (same module, internal access). Timestamp formatter is declared
/// locally so this file is self-contained.

import SwiftUI
import ShareCLICore

struct DashboardOverview: View {
    @ObservedObject var state: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                overviewHeader
                fleetSection
                systemsSection
                activitySection
            }
            .padding(16)
        }
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: - Overview header

    private var overviewHeader: some View {
        let latestFleet = state.fleetHistory.last
        let latestHost = state.hostWatchHistory.last
        return VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Fleet dashboard")
                        .font(.title2.weight(.semibold))
                    Text("Live summary across the sharecli sidecar")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                HealthPill(healthy: latestFleet?.poolHealthy, compact: false)
            }
            HStack(spacing: 12) {
                headerStat(
                    label: "Fleet samples",
                    value: "\(state.fleetHistory.count)",
                    sub: "of \(AppState.fleetHistoryCap)",
                    icon: "chart.xyaxis.line",
                    color: .blue
                )
                headerStat(
                    label: "Host samples",
                    value: "\(state.hostWatchHistory.count)",
                    sub: "of \(AppState.hostWatchHistoryCap)",
                    icon: "waveform.path.ecg",
                    color: .indigo
                )
                headerStat(
                    label: "Live processes",
                    value: "\(state.processes.count)",
                    sub: "tracked",
                    icon: "cpu",
                    color: .green
                )
                headerStat(
                    label: "Last update",
                    value: lastUpdatedLabel(latestFleet?.timestamp),
                    sub: "fleet",
                    icon: "clock",
                    color: .secondary
                )
                if let host = latestHost {
                    headerStat(
                        label: "Host load",
                        value: String(format: "%.2f", host.load_1m),
                        sub: "1m avg",
                        icon: "speedometer",
                        color: loadColor(host.load_1m)
                    )
                }
            }
        }
        .padding(16)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
    }

    private func headerStat(label: String, value: String, sub: String, icon: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 10))
                Text(label)
                    .font(.caption2)
            }
            .foregroundStyle(.secondary)
            Text(value)
                .font(.system(.headline, design: .monospaced))
                .foregroundStyle(color)
            Text(sub)
                .font(.system(size: 10, design: .monospaced))
                .foregroundStyle(.tertiary)
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    // MARK: - Fleet section

    private var fleetSection: some View {
        let latest = state.fleetHistory.last
        let procs = latest?.totalProcesses ?? state.processes.count
        let totalMB = latest?.totalMemoryMB ?? 0
        let usedMB = latest?.usedMemoryMB ?? 0
        let cpu = latest?.cpuAvgPercent ?? 0
        return VStack(alignment: .leading, spacing: 8) {
            sectionHeader(
                title: "Fleet telemetry",
                subtitle: "Per-poll aggregates from the sidecar's monitoring.report",
                systemImage: "chart.bar.doc.horizontal"
            )
            HStack(spacing: 12) {
                compactTile(
                    title: "Processes",
                    value: "\(procs)",
                    sub: "tracked",
                    icon: "cpu",
                    color: .green,
                    poolHealthy: latest?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Total memory",
                    value: "\(totalMB) MB",
                    sub: "fleet-wide",
                    icon: "memorychip",
                    color: .orange,
                    poolHealthy: latest?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Used memory",
                    value: "\(usedMB) MB",
                    sub: "excl. caches",
                    icon: "memorychip.fill",
                    color: .red,
                    poolHealthy: latest?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Avg CPU",
                    value: String(format: "%.1f%%", cpu),
                    sub: cpu > 60 ? "high" : "nominal",
                    icon: "cpu.fill",
                    color: cpu > 60 ? .red : .blue,
                    poolHealthy: latest?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
            }
        }
    }

    // MARK: - Systems section

    private var systemsSection: some View {
        let latest = state.hostWatchHistory.last
        let load = latest?.load_1m ?? 0
        let fds = latest?.fd_count ?? 0
        let rxRate = networkDelta(latest: latest, key: \.net_rx_bytes)
        let txRate = networkDelta(latest: latest, key: \.net_tx_bytes)
        let rss = latest?.mem_rss_bytes ?? 0
        return VStack(alignment: .leading, spacing: 8) {
            sectionHeader(
                title: "Host watch",
                subtitle: "System-level counters captured per monitoring.report",
                systemImage: "waveform.path.ecg"
            )
            HStack(spacing: 12) {
                compactTile(
                    title: "Load (1m)",
                    value: String(format: "%.2f", load),
                    sub: load > 4 ? "high" : "ok",
                    icon: "speedometer",
                    color: loadColor(load),
                    poolHealthy: state.fleetHistory.last?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Open FDs",
                    value: "\(fds)",
                    sub: "process-wide",
                    icon: "tray.full",
                    color: .purple,
                    poolHealthy: state.fleetHistory.last?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Net RX",
                    value: ByteCountFormatter.string(fromByteCount: Int64(rxRate), countStyle: .file) + "/s",
                    sub: "delta",
                    icon: "arrow.down.circle",
                    color: .cyan,
                    poolHealthy: state.fleetHistory.last?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "Net TX",
                    value: ByteCountFormatter.string(fromByteCount: Int64(txRate), countStyle: .file) + "/s",
                    sub: "delta",
                    icon: "arrow.up.circle",
                    color: .pink,
                    poolHealthy: state.fleetHistory.last?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
                compactTile(
                    title: "RSS",
                    value: ByteCountFormatter.string(fromByteCount: Int64(rss), countStyle: .memory),
                    sub: "host",
                    icon: "internaldrive",
                    color: .indigo,
                    poolHealthy: state.fleetHistory.last?.poolHealthy,
                    lastUpdated: latest?.timestamp
                )
            }
        }
    }

    // MARK: - Activity section

    private var activitySection: some View {
        let procs = state.processes
        let topProcs = procs.sorted { $0.memory_mb > $1.memory_mb }.prefix(3)
        let projects = Set(procs.compactMap { $0.project })
        let harnesses = Set(procs.compactMap { $0.harness })
        let recentSpawns = state.spawnHistory.prefix(3)
        let topRSS = procs.first?.memory_mb ?? 0
        return VStack(alignment: .leading, spacing: 8) {
            sectionHeader(
                title: "Process activity",
                subtitle: "Live inventory + recent spawn history",
                systemImage: "list.bullet.rectangle"
            )
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 8) {
                    sectionSubheader("Top RSS")
                    if topProcs.isEmpty {
                        emptyTile()
                    } else {
                        ForEach(Array(topProcs), id: \.pid) { p in
                            activityRow(
                                pid: p.pid,
                                name: p.name,
                                value: "\(p.memory_mb) MB",
                                badge: p.project,
                                color: .orange
                            )
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                VStack(alignment: .leading, spacing: 8) {
                    sectionSubheader("Inventory")
                    activityRow(pid: 0, name: "Projects", value: "\(projects.count)", badge: nil, color: .blue)
                    activityRow(pid: 0, name: "Harnesses", value: "\(harnesses.count)", badge: nil, color: .purple)
                    activityRow(pid: 0, name: "Top RSS", value: "\(topRSS) MB", badge: nil, color: .orange)
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                VStack(alignment: .leading, spacing: 8) {
                    sectionSubheader("Recent spawns")
                    if recentSpawns.isEmpty {
                        emptyTile()
                    } else {
                        ForEach(Array(recentSpawns), id: \.id) { entry in
                            activityRow(
                                pid: entry.spawnedPID ?? 0,
                                name: entry.command,
                                value: entry.succeeded ? "ok" : "fail",
                                badge: entry.project,
                                color: entry.succeeded ? .green : .red
                            )
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    // MARK: - Section header

    private func sectionHeader(title: String, subtitle: String, systemImage: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: systemImage)
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.headline)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func sectionSubheader(_ title: String) -> some View {
        Text(title)
            .font(.subheadline.weight(.semibold))
            .foregroundStyle(.secondary)
    }

    // MARK: - Compact tile (compact variant of the trend card concept)

    private func compactTile(
        title: String,
        value: String,
        sub: String,
        icon: String,
        color: Color,
        poolHealthy: Bool?,
        lastUpdated: Date?
    ) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 11))
                    .foregroundStyle(color)
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                if let poolHealthy {
                    HealthPill(healthy: poolHealthy, compact: true)
                }
            }
            Text(value)
                .font(.system(.title3, design: .monospaced))
                .bold()
                .foregroundStyle(color)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            Text(sub)
                .font(.caption2)
                .foregroundStyle(.tertiary)
                .lineLimit(1)
            if let lastUpdated {
                Text("Updated \(overviewRelativeFormatter.localizedString(for: lastUpdated, relativeTo: Date()))")
                    .font(.system(size: 9, design: .monospaced))
                    .foregroundStyle(.tertiary)
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .strokeBorder(tileBorder(poolHealthy: poolHealthy), lineWidth: 1)
        )
    }

    private func activityRow(pid: UInt32, name: String, value: String, badge: String?, color: Color) -> some View {
        HStack(spacing: 8) {
            if pid > 0 {
                Text("\(pid)")
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundStyle(.tertiary)
                    .frame(width: 44, alignment: .leading)
            } else {
                Image(systemName: "circle.fill")
                    .font(.system(size: 4))
                    .foregroundStyle(color)
                    .frame(width: 44, alignment: .leading)
            }
            Text(name)
                .font(.system(.caption, design: .monospaced))
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
            if let badge {
                Text(badge)
                    .font(.system(size: 9, design: .monospaced))
                    .padding(.horizontal, 4)
                    .padding(.vertical, 1)
                    .background(.quaternary)
                    .clipShape(Capsule())
            }
            Text(value)
                .font(.system(.caption2, design: .monospaced))
                .foregroundStyle(color)
                .bold()
        }
        .padding(.vertical, 2)
    }

    private func emptyTile() -> some View {
        VStack(spacing: 4) {
            Image(systemName: "tray")
                .font(.system(size: 18))
                .foregroundStyle(.tertiary)
            Text("No data yet")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, minHeight: 56)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(.quaternary.opacity(0.3))
        )
    }

    // MARK: - Helpers

    /// Tint the tile border subtly based on pool health.
    /// nil → invisible border (waiting for first sample).
    private func tileBorder(poolHealthy: Bool?) -> Color {
        guard let poolHealthy else { return .clear }
        switch poolHealthy {
        case true: return Color.green.opacity(0.35)
        case false: return Color.yellow.opacity(0.45)
        }
    }

    /// Map a load average value to a color: green < 2, yellow < 4, red otherwise.
    private func loadColor(_ load: Double) -> Color {
        if load < 2.0 { return .green }
        if load < 4.0 { return .yellow }
        return .red
    }

    /// Compute per-second network delta between the last two host-watch samples.
    /// Returns 0 if there are fewer than 2 samples or the timestamp delta is non-positive.
    private func networkDelta(latest: HostWatchSample?, key: KeyPath<HostWatchSample, UInt64>) -> UInt64 {
        let samples = state.hostWatchHistory
        guard let latest, samples.count >= 2 else { return 0 }
        guard let idx = samples.firstIndex(where: { $0.id == latest.id }), idx > 0 else { return 0 }
        let prev = samples[idx - 1]
        let dt = latest.timestamp.timeIntervalSince(prev.timestamp)
        guard dt > 0 else { return 0 }
        let cur = latest[keyPath: key]
        let prevVal = prev[keyPath: key]
        guard cur >= prevVal else { return 0 }
        return UInt64(Double(cur - prevVal) / dt)
    }

    /// Short label for "Last update" — falls back to "—" when no sample.
    private func lastUpdatedLabel(_ ts: Date?) -> String {
        guard let ts else { return "—" }
        return overviewRelativeFormatter.localizedString(for: ts, relativeTo: Date())
    }
}

// MARK: - File-scoped formatter

/// Local RelativeDateTimeFormatter (abbreviated) used by compact tiles.
/// Mirrors the formatter in ProcessesPage.swift; kept private so the two
/// files don't need to share internals.
private let overviewRelativeFormatter: RelativeDateTimeFormatter = {
    let f = RelativeDateTimeFormatter()
    f.unitsStyle = .abbreviated
    return f
}()
