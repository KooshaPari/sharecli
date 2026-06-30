/// TrayPopoverView.swift — compact popover (click tray icon → this appears).
///
/// Shows: connection status, summary stats, quick-action buttons, process list preview.

import SwiftUI
import ShareCLICore

struct TrayPopoverView: View {
    @ObservedObject var state: AppState
    let onOpenDashboard: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            headerBar
            Divider()
            statsRow
            Divider()
            processPreview
            Divider()
            actionBar
        }
        .frame(width: 360)
        .background(.ultraThinMaterial)
    }

    // MARK: - Header

    private var headerBar: some View {
        HStack {
            Image(systemName: "cpu")
                .foregroundStyle(.blue)
            Text("ShareCLI")
                .font(.headline)
            Spacer()
            Circle()
                .fill(state.isConnected ? Color.green : Color.red)
                .frame(width: 8, height: 8)
            Text(state.isConnected ? "connected" : "disconnected")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }

    // MARK: - Stats row

    private var statsRow: some View {
        HStack(spacing: 0) {
            statCell(
                icon: "app.badge",
                value: "\(state.health?.managed_processes ?? state.processes.count)",
                label: "Processes"
            )
            Divider().frame(height: 36)
            statCell(
                icon: "memorychip",
                value: state.health.map { "\($0.used_memory_mb)M / \($0.total_memory_mb)M" } ?? "—",
                label: "Memory"
            )
            Divider().frame(height: 36)
            statCell(
                icon: state.health?.healthy == true ? "checkmark.seal.fill" : "exclamationmark.triangle.fill",
                value: state.health?.healthy == true ? "Healthy" : "Warning",
                label: "Status",
                iconColor: state.health?.healthy == true ? .green : .orange
            )
        }
        .padding(.vertical, 8)
    }

    private func statCell(
        icon: String,
        value: String,
        label: String,
        iconColor: Color = .secondary
    ) -> some View {
        VStack(spacing: 2) {
            Image(systemName: icon).foregroundStyle(iconColor)
            Text(value).font(.system(.body, design: .monospaced)).bold()
            Text(label).font(.caption2).foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
    }

    // MARK: - Process preview list (top 5)

    private var processPreview: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Managed Processes")
                .font(.caption)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 12)
                .padding(.top, 8)

            if state.processes.isEmpty {
                Text(state.isConnected ? "No managed processes" : "Waiting for IPC…")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 16)
            } else {
                ForEach(state.processes.prefix(5)) { proc in
                    processRow(proc)
                }
                if state.processes.count > 5 {
                    Text("+ \(state.processes.count - 5) more…")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 12)
                        .padding(.bottom, 4)
                }
            }
        }
        .frame(minHeight: 60)
    }

    private func processRow(_ proc: ProcessSummary) -> some View {
        HStack {
            VStack(alignment: .leading, spacing: 1) {
                Text(proc.name)
                    .font(.system(.caption, design: .monospaced))
                    .lineLimit(1)
                HStack(spacing: 4) {
                    if let project = proc.project {
                        Badge(text: project, color: .blue)
                    }
                    if let harness = proc.harness {
                        Badge(text: harness, color: .purple)
                    }
                }
            }
            Spacer()
            Text("\(proc.memory_mb)M")
                .font(.caption2)
                .foregroundStyle(.secondary)
            Button {
                Task { await state.kill(pid: proc.pid) }
            } label: {
                Image(systemName: "xmark.circle")
                    .foregroundStyle(.red.opacity(0.7))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 4)
    }

    // MARK: - Action bar

    private var actionBar: some View {
        HStack {
            Button("Dashboard") {
                onOpenDashboard()
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.small)

            Spacer()

            Button {
                Task { await state.refresh() }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .buttonStyle(.bordered)
            .controlSize(.small)

            Button("Kill All") {
                Task { await state.killAll() }
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .foregroundStyle(.red)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }
}

struct Badge: View {
    let text: String
    let color: Color

    var body: some View {
        Text(text)
            .font(.system(size: 9, weight: .medium))
            .padding(.horizontal, 4)
            .padding(.vertical, 1)
            .background(color.opacity(0.15))
            .foregroundStyle(color)
            .clipShape(RoundedRectangle(cornerRadius: 3))
    }
}
