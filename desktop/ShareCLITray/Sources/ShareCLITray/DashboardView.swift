/// DashboardView.swift — full NSWindow dashboard (process table + config editor).
///
/// Navigation sidebar:
///   Processes  → live process table with per-row kill + filter by project/harness
///   Agents     → fleet agent process roster (PR 1)
///   Config     → spawn_policy + pool + monitoring config editor with live apply
///   Health     → Memory / Thermal gate / Host watch (PR 6, subpages in HealthPage)

import SwiftUI
import ShareCLICore

struct DashboardView: View {
    @ObservedObject var state: AppState
    @State private var selection: Section = .processes

    enum Section: String, CaseIterable, Identifiable {
        var id: String { rawValue }
        case processes = "Processes"
        case agents = "Agents"
        case pool = "Pool"
        case effectiveness = "Pool effectiveness"
        case config = "Config"
        case health = "Health"
        case logs = "Logs"
    }

    var body: some View {
        NavigationSplitView {
            List(Section.allCases, selection: $selection) { sec in
                Label(sec.rawValue, systemImage: iconName(for: sec))
                    .tag(sec)
            }
            .navigationSplitViewColumnWidth(min: 140, ideal: 160)
        } detail: {
            Group {
                switch selection {
                case .processes: ProcessesPage(state: state)
                case .agents: AgentsPage(state: state)
                case .pool: PoolPage(state: state)
                case .effectiveness: PoolEffectivenessPage(state: state)
                case .config: ConfigPage(state: state)
                case .health: HealthPage(state: state)
                case .logs: LogsPage(state: state)
                }
            }
            .frame(minWidth: 600)
        }
        .frame(minWidth: 800, minHeight: 500)
        .toolbar {
            ToolbarItem {
                Button {
                    Task { await state.refresh() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
            }
            ToolbarItem {
                if let err = state.lastError {
                    Label(err, systemImage: "exclamationmark.triangle")
                        .foregroundStyle(.red)
                        .font(.caption)
                }
            }
        }
    }

    private func iconName(for sec: Section) -> String {
        switch sec {
        case .processes: return "cpu"
        case .agents: return "person.2.fill"
        case .pool: return "rectangle.stack.fill"
        case .effectiveness: return "chart.line.uptrend.xyaxis"
        case .config: return "gearshape"
        case .health: return "heart.fill"
        case .logs: return "text.alignleft"
        }
    }
}

// MARK: - Process Table

struct ProcessTableView: View {
    @ObservedObject var state: AppState
    @State private var filterText = ""
    @State private var sortOrder = [KeyPathComparator(\ProcessSummary.memory_mb, order: .reverse)]

    private var filtered: [ProcessSummary] {
        let q = filterText.lowercased()
        if q.isEmpty { return state.processes }
        return state.processes.filter {
            $0.name.lowercased().contains(q)
            || ($0.project?.lowercased().contains(q) ?? false)
            || ($0.harness?.lowercased().contains(q) ?? false)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                TextField("Filter by name / project / harness", text: $filterText)
                    .textFieldStyle(.plain)
                Spacer()
                Text("\(filtered.count) processes")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .padding(10)
            .background(.quaternary)

            Table(filtered, sortOrder: $sortOrder) {
                TableColumn("PID", value: \.pid) { p in
                    Text("\(p.pid)").font(.system(.body, design: .monospaced))
                }
                .width(60)

                TableColumn("Name", value: \.name) { p in
                    Text(p.name).font(.system(.body, design: .monospaced))
                }

                TableColumn("Project") { p in
                    if let proj = p.project {
                        Badge(text: proj, color: .blue)
                    }
                }
                .width(100)

                TableColumn("Harness") { p in
                    if let h = p.harness {
                        Badge(text: h, color: .purple)
                    }
                }
                .width(80)

                TableColumn("Memory (MB)", value: \.memory_mb) { p in
                    Text("\(p.memory_mb)").font(.system(.body, design: .monospaced))
                }
                .width(100)

                TableColumn("Actions") { p in
                    Button("Kill") {
                        Task { await state.kill(pid: p.pid) }
                    }
                    .buttonStyle(.borderless)
                    .foregroundStyle(.red)
                }
                .width(50)
            }
        }
    }
}

// MARK: - Config Editor
//
// Removed in PR 7 of the dashboard expansion plan. The new ConfigPage
// (Sources/ShareCLITray/ConfigPage.swift) replaces it with a live-fetched
// form + JSON preview + Defaults subpanel. The page is wired into
// DashboardView.Section.config.