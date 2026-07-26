/// DashboardView.swift — full NSWindow dashboard with 7 sidebar pages.
///
/// Navigation sidebar (Cmd+1..7 to jump):
///   1. Processes    → live process table with filter + bulk kill + export (PR 2)
///   2. Agents       → fleet agent process roster (PR 1)
///   3. Pool         → runtime pool composition + issues + host watch (PR 3)
///   4. Effectiveness → cache/slot-queue hit rate counters from sharecli-fleet (PR 4)
///   5. Config       → spawn_policy + pool + monitoring config editor (PR 7)
///   6. Health       → Memory / Thermal gate / Host watch + subpages (PR 6)
///   7. Logs         → live-tailing log viewer with filter + export (PR 8)

import SwiftUI
import ShareCLICore

struct DashboardView: View {
    @ObservedObject var state: AppState
    @AppStorage("dashboard.sidebar.selection") private var selectionRaw: String = Section.processes.rawValue
    @AppStorage("dashboard.sidebar.columnWidth") private var sidebarColumnWidth: Double = 168

    private var selection: Binding<Section> {
        Binding(
            get: { Section(rawValue: selectionRaw) ?? .processes },
            set: { selectionRaw = $0.rawValue }
        )
    }

    enum Section: String, CaseIterable, Identifiable {
        case processes = "Processes"
        case agents = "Agents"
        case pool = "Pool"
        case effectiveness = "Pool effectiveness"
        case config = "Config"
        case health = "Health"
        case logs = "Logs"
        var id: String { rawValue }
        var icon: String {
            switch self {
            case .processes: return "cpu"
            case .agents: return "person.2.fill"
            case .pool: return "rectangle.stack.fill"
            case .effectiveness: return "chart.line.uptrend.xyaxis"
            case .config: return "gearshape"
            case .health: return "heart.fill"
            case .logs: return "text.alignleft"
            }
        }
        /// Cmd+N index for keyboard shortcuts (1-based per the plan §5.4).
        var shortcutIndex: Int {
            Section.allCases.firstIndex(of: self).map { $0 + 1 } ?? 0
        }
    }

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detail
                .frame(minWidth: 600)
        }
        .navigationSplitViewColumnWidth(min: 140, ideal: sidebarColumnWidth)
        .frame(minWidth: 900, minHeight: 560)
        .background(WindowAccessor { window in
            attachShortcutMonitor(window: window)
        })
        .toolbar { toolbar }
        .onAppear {
            // Persist a sensible default if the @AppStorage above was missing.
            if Section(rawValue: selectionRaw) == nil {
                selectionRaw = Section.processes.rawValue
            }
        }
    }

    @ViewBuilder
    private var sidebar: some View {
        List(Section.allCases, selection: selection) { sec in
            Label {
                HStack {
                    Text(sec.rawValue)
                    Spacer()
                    if sec.shortcutIndex > 0 {
                        Text("⌘\(sec.shortcutIndex)")
                            .font(.caption2.monospaced())
                            .foregroundStyle(.tertiary)
                    }
                }
            } icon: {
                Image(systemName: sec.icon)
            }
            .help(sectionSummary(sec))
        }
    }

    @ViewBuilder
    private var detail: some View {
        switch Section(rawValue: selectionRaw) ?? .processes {
        case .processes: ProcessesPage(state: state)
        case .agents: AgentsPage(state: state)
        case .pool: PoolPage(state: state)
        case .effectiveness: PoolEffectivenessPage(state: state)
        case .config: ConfigPage(state: state)
        case .health: HealthPage(state: state)
        case .logs: LogsPage(state: state)
        }
    }

    @ToolbarContentBuilder
    private var toolbar: some ToolbarContent {
        ToolbarItem(placement: .primaryAction) {
            Button {
                Task { await state.refresh() }
            } label: {
                Label("Refresh", systemImage: "arrow.clockwise")
            }
            .help("Refresh all panels (⌘R)")
        }
        ToolbarItem(placement: .primaryAction) {
            if let err = state.lastError {
                Label(err, systemImage: "exclamationmark.triangle.fill")
                    .foregroundStyle(.red)
                    .help(err)
            }
        }
    }

    // MARK: - Keyboard shortcut plumbing

    private func attachShortcutMonitor(window: NSWindow?) -> Void {
        // Use a local event monitor so Cmd+1..7 work even when focus is on
        // a SwiftUI subview (SwiftUI's .keyboardShortcut(_, modifiers:)
        // attaches to a specific button; we want it for the whole window).
        let monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            if event.modifierFlags.contains(.command),
               let chars = event.charactersIgnoringModifiers,
               let c = chars.first,
               let n = Int(String(c)),
               (1...Section.allCases.count).contains(n),
               let sec = Section.allCases[safe: n - 1] {
                self.selectionRaw = sec.rawValue
                NSLog("[DashboardView] Cmd+%d → %@", n, sec.rawValue)
                return nil
            }
            if event.modifierFlags.contains(.command),
               event.charactersIgnoringModifiers == "r" {
                Task { await state.refresh() }
                return nil
            }
            return event
        }
        if let window = window, let token = monitor {
            // Keep the monitor alive at least as long as the window.
            objc_setAssociatedObject(window, &Self.monitorKey, token, .OBJC_ASSOCIATION_RETAIN)
        }
    }

    private static var monitorKey: UInt8 = 0

    // MARK: - Helpers

    private func sectionSummary(_ sec: Section) -> String {
        switch sec {
        case .processes: return "All live processes — bulk kill, JSON/CSV export, by-project/by-harness grouping. ⌘1"
        case .agents: return "Spawned fleet agents (claude / forge / node / bun / etc). ⌘2"
        case .pool: return "Runtime pool composition + gate decisions + host watch sparklines. ⌘3"
        case .effectiveness: return "Hypervisor coalesce cache + slot-queue counters. ⌘4"
        case .config: return "Spawn policy / pool / monitoring config editor with live apply. ⌘5"
        case .health: return "Memory / Thermal gate / Host resource watch with subpages. ⌘6"
        case .logs: return "Live tail of ~/.sharecli/logs/sharecli.log with filter + export. ⌘7"
        }
    }
}

private extension Array {
    subscript(safe idx: Int) -> Element? {
        indices.contains(idx) ? self[idx] : nil
    }
}

// MARK: - Window accessor (SwiftUI 4+ on macOS)

private struct WindowAccessor: NSViewRepresentable {
    let onResolve: (NSWindow?) -> Void
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        DispatchQueue.main.async { [weak view] in
            onResolve(view?.window)
        }
        return view
    }
    func updateNSView(_ nsView: NSView, context: Context) {}
}

// MARK: - ProcessTableView (kept for backward compat with @AppStorage refs)
//
// The legacy single-table ProcessTableView has been superseded by
// ProcessesPage (PR 2). This minimal stub remains so any leftover
// Build reference compiles — the only call site (DashboardView switch)
// now routes to ProcessesPage directly.

struct ProcessTableView: View {
    @ObservedObject var state: AppState
    var body: some View {
        ProcessesPage(state: state)
    }
}
