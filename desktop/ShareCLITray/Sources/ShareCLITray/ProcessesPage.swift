/// ProcessesPage.swift — expanded Processes page (PR 2 of dashboard expansion plan).
///
/// Replaces the simple `ProcessTableView` inside `DashboardView` with a 3-subpage
/// layout driven by `state.processes: [ProcessSummary]` (which now carries
/// `start_time` after PR 2's sidecar extension — see
/// `crates/sharecli-ipc/src/handler.rs:99-109`).
///
/// Subpages (segmented at top):
///   ┌─────────────────────────────────────────────────────────────────┐
///   │ [All] [By Project] [By Harness]                                 │
///   ├─────────────────────────────────────────────────────────────────┤
///   │ All:     Filter bar + sortable Table (PID/Name/Project/Harness/│
///   │          Memory/Age/Actions) + bulk-action bar + detail drawer  │
///   │ By Proj: Grouped sections w/ project cards (count, RSS, top     │
///   │          harness) + horizontal RSS bar chart + expandable rows  │
///   │ By Har:  Same as By Project but grouped by harness              │
///   └─────────────────────────────────────────────────────────────────┘
///
/// Layout:
///   - Summary strip (4 cards): total / running-aware / total RSS / by project
///   - Filter bar: text + min-RSS slider + bulk-select checkboxes (All only)
///   - Detail drawer (All only): cmd preview + age + per-row kill
///
/// Persistence:
///   - `processes.selectedSubpage` (String: "all" / "byProject" / "byHarness")
///   - `processes.sortFingerprint` (JSON of KeyPathComparator order; restored)
///
/// Bulk actions (All subpage only):
///   - "Kill selected" → state.kill(pid) per selected row
///   - "Kill all" → state.killAll()
///   - "Export JSON" / "Export CSV" → NSSavePanel of filtered set
///
/// Part of: plans/2026-07-25-tray-dashboard-expanded-v1.md §2.1 Page 1.

import SwiftUI
import ShareCLICore
import AppKit
import UniformTypeIdentifiers

struct ProcessesPage: View {
    @ObservedObject var state: AppState

    @AppStorage("processes.subpage") private var subpageRaw: String = Subpage.all.rawValue
    @State private var subpage: Subpage = .all
    @State private var didLoadSubpage = false

    enum Subpage: String, CaseIterable, Identifiable {
        case all = "all"
        case byProject = "byProject"
        case byHarness = "byHarness"
        case tree = "tree"
        var id: String { rawValue }
        var label: String {
            switch self {
            case .all: return "All"
            case .byProject: return "By Project"
            case .byHarness: return "By Harness"
            case .tree: return "Tree"
            }
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            Picker("", selection: $subpage) {
                ForEach(Subpage.allCases) { sp in
                    Text(sp.label).tag(sp)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .onChange(of: subpage) { _, newValue in
                subpageRaw = newValue.rawValue
            }

            Divider()

            switch subpage {
            case .all: allSubpage
            case .byProject: groupedSubpage(by: \.project, groupLabel: "Project")
            case .byHarness: groupedSubpage(by: \.harness, groupLabel: "Harness")
            case .tree: treeSubpage
            }
        }
        .frame(minWidth: 720, minHeight: 460)
        .onAppear {
            if !didLoadSubpage {
                subpage = Subpage(rawValue: subpageRaw) ?? .all
                didLoadSubpage = true
            }
        }
    }

    // MARK: - All subpage

    private var allSubpage: some View {
        AllProcessesView(state: state)
    }

    // MARK: - Tree subpage

    private var treeSubpage: some View {
        TreeView(state: state)
    }

    // MARK: - Grouped subpage (By Project / By Harness)

    private func groupedSubpage(by keyPath: KeyPath<ProcessSummary, String?>, groupLabel: String) -> some View {
        let groups = groupBy(state.processes, by: keyPath)
        return ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                summaryStripGrouped(groups: groups)
                ForEach(groups, id: \.label) { group in
                    ProjectGroupCard(
                        group: group,
                        keyLabel: groupLabel,
                        kill: { pid in Task { await state.kill(pid: pid) } }
                    )
                }
                if groups.isEmpty {
                    emptyState(message: "No processes to group by \(groupLabel.lowercased()).")
                }
            }
            .padding(16)
        }
    }

    private func summaryStripGrouped(groups: [ProcessGroup]) -> some View {
        let total = state.processes.count
        let totalRSS = state.processes.reduce(UInt64(0)) { $0 + $1.memory_mb * 1024 * 1024 }
        let projectCount = Set(state.processes.compactMap { $0.project }).count
        let harnessCount = Set(state.processes.compactMap { $0.harness }).count
        return HStack(spacing: 12) {
            summaryCard("Total", "\(total)", "processes", .blue)
            summaryCard("Total RSS", ByteCountFormatter.string(fromByteCount: Int64(totalRSS), countStyle: .memory), "across fleet", .orange)
            summaryCard("Projects", "\(projectCount)", "unique", .purple)
            summaryCard("Harnesses", "\(harnessCount)", "unique", .green)
        }
    }

    private func summaryCard(_ title: String, _ value: String, _ sub: String, _ color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(.secondary)
            Text(value).font(.system(.title3, design: .monospaced)).bold().foregroundStyle(color)
            Text(sub).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(.quaternary)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    // MARK: - Empty state

    private func emptyState(message: String) -> some View {
        VStack(spacing: 8) {
            Image(systemName: "tray")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text(message)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, minHeight: 240)
        .padding(24)
    }

    // MARK: - Grouping helper

    private func groupBy(_ rows: [ProcessSummary], by keyPath: KeyPath<ProcessSummary, String?>) -> [ProcessGroup] {
        var map: [String: [ProcessSummary]] = [:]
        var order: [String] = []
        for row in rows {
            let key = row[keyPath: keyPath] ?? "(none)"
            if map[key] == nil { order.append(key) }
            map[key, default: []].append(row)
        }
        return order.map { key in
            let members = map[key] ?? []
            let totalRSS = members.reduce(UInt64(0)) { $0 + $1.memory_mb * 1024 * 1024 }
            // Top harness per project: count of harness values within members
            var harnessCounts: [String: Int] = [:]
            for m in members {
                let h = m.harness ?? "(none)"
                harnessCounts[h, default: 0] += 1
            }
            let topHarness = harnessCounts.max(by: { $0.value < $1.value })?.key ?? "(none)"
            return ProcessGroup(
                label: key,
                members: members.sorted { $0.memory_mb > $1.memory_mb },
                totalRSSBytes: totalRSS,
                topHarness: topHarness,
                uniqueHarnessCount: harnessCounts.count
            )
        }
        .sorted { $0.totalRSSBytes > $1.totalRSSBytes }
    }
}

// MARK: - ProcessGroup data

struct ProcessGroup: Hashable {
    let label: String
    let members: [ProcessSummary]
    let totalRSSBytes: UInt64
    let topHarness: String
    let uniqueHarnessCount: Int
}

// MARK: - ProjectGroupCard (renders one grouped section)

struct ProjectGroupCard: View {
    let group: ProcessGroup
    let keyLabel: String
    let kill: (UInt32) -> Void
    @State private var expanded = true

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(keyLabel).font(.caption2).foregroundStyle(.secondary)
                        Text(group.label)
                            .font(.system(.headline, design: .monospaced))
                            .bold()
                    }
                    HStack(spacing: 12) {
                        Label("\(group.members.count)", systemImage: "cpu")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Label(
                            ByteCountFormatter.string(fromByteCount: Int64(group.totalRSSBytes), countStyle: .memory),
                            systemImage: "memorychip"
                        )
                        .font(.caption)
                        .foregroundStyle(.orange)
                        Label("top: \(group.topHarness)", systemImage: "tag")
                            .font(.caption)
                            .foregroundStyle(.purple)
                        if group.uniqueHarnessCount > 1 {
                            Text("+\(group.uniqueHarnessCount - 1) more harness")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
                Spacer()
                Button {
                    withAnimation(.easeInOut(duration: 0.15)) { expanded.toggle() }
                } label: {
                    Image(systemName: expanded ? "chevron.up" : "chevron.down")
                }
                .buttonStyle(.borderless)
            }

            // RSS bar — relative to max group in this set
            if expanded {
                Divider()
                rssBarChart
                VStack(spacing: 4) {
                    ForEach(group.members.prefix(5), id: \.pid) { p in
                        ProcessRowInline(p: p, kill: kill)
                    }
                    if group.members.count > 5 {
                        Text("+\(group.members.count - 5) more (sorted by RSS desc)")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
        }
        .padding(12)
        .background(.quaternary)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }

    private var rssBarChart: some View {
        let maxBytes = group.members.map { $0.memory_mb }.max() ?? 1
        return VStack(spacing: 2) {
            ForEach(group.members.prefix(8), id: \.pid) { p in
                let frac = Double(p.memory_mb) / Double(max(maxBytes, 1))
                HStack(spacing: 6) {
                    Text(p.name)
                        .font(.system(.caption, design: .monospaced))
                        .frame(width: 110, alignment: .leading)
                        .lineLimit(1)
                    GeometryReader { geo in
                        ZStack(alignment: .leading) {
                            RoundedRectangle(cornerRadius: 3).fill(.quaternary.opacity(0.5))
                            RoundedRectangle(cornerRadius: 3)
                                .fill(rssBarColor(p.memory_mb))
                                .frame(width: max(2, geo.size.width * CGFloat(frac)))
                        }
                    }
                    .frame(height: 10)
                    Text("\(p.memory_mb) MB")
                        .font(.system(.caption2, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .frame(width: 60, alignment: .trailing)
                }
            }
        }
    }

    private func rssBarColor(_ mb: UInt64) -> Color {
        if mb > 1024 { return .red }
        if mb > 512 { return .orange }
        if mb > 128 { return .yellow }
        return .blue
    }
}

struct ProcessRowInline: View {
    let p: ProcessSummary
    let kill: (UInt32) -> Void
    var body: some View {
        HStack(spacing: 8) {
            Text("\(p.pid)").font(.system(.caption, design: .monospaced)).frame(width: 50, alignment: .leading)
            Text(p.name).font(.system(.caption, design: .monospaced)).frame(width: 140, alignment: .leading).lineLimit(1)
            if let proj = p.project { Badge(text: proj, color: .blue) }
            if let h = p.harness { Badge(text: h, color: .purple) }
            Spacer()
            Text("\(p.memory_mb) MB")
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.secondary)
            Button {
                kill(p.pid)
            } label: {
                Image(systemName: "xmark.circle.fill").foregroundStyle(.red)
            }
            .buttonStyle(.borderless)
            .help("Kill PID \(p.pid)")
        }
    }
}

// MARK: - AllProcessesView

struct AllProcessesView: View {
    @ObservedObject var state: AppState

    @State private var filterText: String = ""
    @State private var minRSS: Double = 0  // MB threshold
    @State private var sortOrder: [KeyPathComparator<ProcessSummary>] = [
        KeyPathComparator(\ProcessSummary.memory_mb, order: .reverse)
    ]
    @State private var selection: Set<UInt32> = []
    @State private var bulkStatus: String = ""

    private var filtered: [ProcessSummary] {
        let q = filterText.lowercased()
        let minBytes = UInt64(max(0, minRSS)) * 1024 * 1024
        return state.processes.filter { p in
            if minBytes > 0 && p.memory_mb * 1024 * 1024 < minBytes { return false }
            if q.isEmpty { return true }
            return p.name.lowercased().contains(q)
                || (p.project?.lowercased().contains(q) ?? false)
                || (p.harness?.lowercased().contains(q) ?? false)
                || String(p.pid).contains(q)
        }
        .sorted(using: sortOrder)
    }

    var body: some View {
        VStack(spacing: 0) {
            summaryStrip
            filterBar
            bulkActionBar
            Divider()
            if filtered.isEmpty {
                EmptyStateView(
                    icon: "tray",
                    title: state.processes.isEmpty
                        ? "No processes yet"
                        : "No processes match your filter",
                    subtitle: state.processes.isEmpty
                        ? "The fleet pool will list registered processes here once the host directory contains bun sockets or any host-tracked CLI processes."
                        : "Try widening the text filter, lowering the minimum RSS slider, or clearing the filter entirely.",
                    variant: .hero,
                    primaryTitle: state.processes.isEmpty ? "Refresh now" : "Clear filter",
                    primaryIcon: state.processes.isEmpty ? "arrow.clockwise" : "xmark.circle",
                    primaryAction: {
                        if state.processes.isEmpty {
                            Task { await state.refresh() }
                        } else {
                            filterText = ""
                            minRSS = 0
                        }
                    },
                    secondaryTitle: state.processes.isEmpty ? "How does this work?" : nil,
                    secondaryIcon: "questionmark.circle",
                    secondaryAction: state.processes.isEmpty ? {
                        if let url = URL(string: "https://docs.sharecli.dev/processes") { NSWorkspace.shared.open(url) }
                    } : nil
                )
            } else {
                Table(filtered, selection: $selection, sortOrder: $sortOrder) {
                TableColumn("PID", value: \.pid) { p in
                    Text("\(p.pid)").font(.system(.body, design: .monospaced))
                }
                .width(60)

                TableColumn("Name", value: \.name) { p in
                    Text(p.name).font(.system(.body, design: .monospaced)).lineLimit(1)
                }

                TableColumn("Project") { p in
                    if let proj = p.project { Badge(text: proj, color: .blue) }
                }
                .width(100)

                TableColumn("Harness") { p in
                    if let h = p.harness { Badge(text: h, color: .purple) }
                }
                .width(80)

                TableColumn("Memory (MB)", value: \.memory_mb) { p in
                    Text("\(p.memory_mb)").font(.system(.body, design: .monospaced))
                        .foregroundStyle(rssColor(p.memory_mb))
                }
                .width(110)

                TableColumn("CPU %", value: \.cpu_percent) { p in
                    HStack(spacing: 4) {
                        Text(String(format: "%.1f%%", p.cpu_percent))
                            .font(.system(.body, design: .monospaced))
                            .foregroundStyle(cpuColor(p.cpu_percent))
                            .frame(width: 56, alignment: .trailing)
                        cpuBar(p.cpu_percent)
                    }
                }
                .width(140)

                TableColumn("Age") { p in
                    Text(formatAge(p.start_time))
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(.secondary)
                }
                .width(80)

                TableColumn("Actions") { p in
                    Button {
                        Task { await state.kill(pid: p.pid) }
                    } label: {
                        Image(systemName: "xmark.circle.fill").foregroundStyle(.red)
                    }
                    .buttonStyle(.borderless)
                    .help("Kill PID \(p.pid) (\(p.name))")
                }
                .width(40)
            }
            .frame(minHeight: 240)
            } // end Table
        }
    }

    // MARK: - Summary

    private var summaryStrip: some View {
        let total = state.processes.count
        let filteredCount = filtered.count
        let totalRSSBytes = state.processes.reduce(UInt64(0)) { $0 + $1.memory_mb * 1024 * 1024 }
        let selectedRSS = filtered
            .filter { selection.contains($0.pid) }
            .reduce(UInt64(0)) { $0 + $1.memory_mb * 1024 * 1024 }
        let topCPU = state.processes
            .max(by: { $0.cpu_percent < $1.cpu_percent })?.cpu_percent ?? 0
        return HStack(spacing: 12) {
            summaryCard("Total", "\(total)", filteredCount == total ? "processes" : "\(filteredCount) shown", .blue)
            summaryCard("Total RSS", ByteCountFormatter.string(fromByteCount: Int64(totalRSSBytes), countStyle: .memory), "fleet", .orange)
            summaryCard("Selected", "\(selection.count)", selection.isEmpty ? "—" : ByteCountFormatter.string(fromByteCount: Int64(selectedRSS), countStyle: .memory), .purple)
            summaryCard("Top CPU", String(format: "%.1f%%", topCPU), "fleet peak", cpuColor(topCPU))
            summaryCard("Filtered", "\(filteredCount)", "of \(total)", .green)
        }
        .padding(10)
        .background(.quaternary.opacity(0.5))
    }

    private func summaryCard(_ title: String, _ value: String, _ sub: String, _ color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(.secondary)
            Text(value).font(.system(.title3, design: .monospaced)).bold().foregroundStyle(color)
            Text(sub).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(8)
        .background(.quaternary)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    // MARK: - Filter bar

    private var filterBar: some View {
        HStack(spacing: 8) {
            Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
            TextField("Filter by name / project / harness / pid", text: $filterText)
                .textFieldStyle(.plain)
            if !filterText.isEmpty {
                Button { filterText = "" } label: {
                    Image(systemName: "xmark.circle.fill").foregroundStyle(.secondary)
                }
                .buttonStyle(.borderless)
            }
            Divider().frame(height: 16)
            Text("Min RSS \(Int(minRSS)) MB").font(.caption).foregroundStyle(.secondary)
            Slider(value: $minRSS, in: 0...4096, step: 64) {
                Text("Min RSS")
            }
            .frame(width: 160)
            Spacer()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(.quaternary.opacity(0.3))
    }

    // MARK: - Bulk action bar

    private var bulkActionBar: some View {
        HStack(spacing: 10) {
            Button {
                Task {
                    var killed = 0
                    for pid in selection {
                        await state.kill(pid: pid)
                        killed += 1
                    }
                    bulkStatus = "Killed \(killed) selected"
                    try? await Task.sleep(nanoseconds: 2_000_000_000)
                    bulkStatus = ""
                }
            } label: {
                Label("Kill selected (\(selection.count))", systemImage: "xmark.circle")
            }
            .disabled(selection.isEmpty)

            Button {
                Task {
                    await state.killAll()
                    bulkStatus = "Kill-all requested"
                    try? await Task.sleep(nanoseconds: 2_000_000_000)
                    bulkStatus = ""
                }
            } label: {
                Label("Kill all", systemImage: "xmark.octagon")
            }
            .tint(.red)

            Spacer()

            Button {
                exportJSON(filtered: filtered)
            } label: {
                Label("Export JSON", systemImage: "square.and.arrow.up")
            }

            Button {
                exportCSV(filtered: filtered)
            } label: {
                Label("Export CSV", systemImage: "tablecells")
            }

            if !bulkStatus.isEmpty {
                Text(bulkStatus)
                    .font(.caption)
                    .foregroundStyle(bulkStatus.lowercased().contains("error") ? .red : .green)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(.quaternary.opacity(0.3))
    }

    // MARK: - Export

    private func exportJSON(filtered: [ProcessSummary]) {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [UTType.json]
        panel.nameFieldStringValue = "sharecli-processes.json"
        panel.canCreateDirectories = true
        guard panel.runModal() == .OK, let url = panel.url else { return }

        struct Out: Codable { let exported_at: UInt64; let count: Int; let processes: [ProcessSummary] }
        let payload = Out(
            exported_at: UInt64(Date().timeIntervalSince1970),
            count: filtered.count,
            processes: filtered
        )
        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        if let data = try? enc.encode(payload) {
            try? data.write(to: url)
        }
    }

    private func exportCSV(filtered: [ProcessSummary]) {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [UTType.commaSeparatedText]
        panel.nameFieldStringValue = "sharecli-processes.csv"
        panel.canCreateDirectories = true
        guard panel.runModal() == .OK, let url = panel.url else { return }

        var lines: [String] = ["pid,name,memory_mb,project,harness,start_time,age_seconds,cpu_percent"]
        let now = UInt64(Date().timeIntervalSince1970)
        for p in filtered {
            let age = p.start_time > 0 ? (now >= p.start_time ? now - p.start_time : 0) : 0
            let cells: [String] = [
                "\(p.pid)",
                csvEscape(p.name),
                "\(p.memory_mb)",
                csvEscape(p.project ?? ""),
                csvEscape(p.harness ?? ""),
                "\(p.start_time)",
                "\(age)",
                String(format: "%.2f", p.cpu_percent)
            ]
            lines.append(cells.joined(separator: ","))
        }
        let body = lines.joined(separator: "\n") + "\n"
        try? body.write(to: url, atomically: true, encoding: .utf8)
    }

    private func csvEscape(_ s: String) -> String {
        if s.contains(",") || s.contains("\"") || s.contains("\n") {
            return "\"" + s.replacingOccurrences(of: "\"", with: "\"\"") + "\""
        }
        return s
    }

    // MARK: - Helpers

    private func formatAge(_ startTime: UInt64) -> String {
        guard startTime > 0 else { return "—" }
        let now = UInt64(Date().timeIntervalSince1970)
        guard now >= startTime else { return "?" }
        let secs = now - startTime
        if secs < 60 { return "\(secs)s" }
        if secs < 3600 { return "\(secs / 60)m" }
        if secs < 86400 { return "\(secs / 3600)h \(secs % 3600 / 60)m" }
        return "\(secs / 86400)d"
    }

    private func rssColor(_ mb: UInt64) -> Color {
        if mb > 1024 { return .red }
        if mb > 512 { return .orange }
        if mb > 128 { return .yellow }
        return .primary
    }

    private func cpuColor(_ pct: Float) -> Color {
        if pct > 90 { return .red }
        if pct > 60 { return .orange }
        if pct > 25 { return .yellow }
        return .secondary
    }

    private func cpuBar(_ pct: Float) -> some View {
        let width = max(0, min(1, pct / 100.0))
        return GeometryReader { geo in
            ZStack(alignment: .leading) {
                RoundedRectangle(cornerRadius: 2)
                    .fill(.quaternary)
                RoundedRectangle(cornerRadius: 2)
                    .fill(cpuColor(pct).opacity(0.85))
                    .frame(width: geo.size.width * CGFloat(width))
            }
        }
        .frame(height: 6)
    }
}

// MARK: - Tree subpage

/// In-memory tree model for the .tree subpage. Built lazily from
/// `state.processes` by `TreeView`; nodes whose `ppid` either is nil or
/// points at a PID not present in the current process set are treated as
/// roots (this also handles "cycle" cases where a parent was missing or
/// out-of-band). To keep the model robust against malformed `ppid`
/// cycles (parent → child → parent), `TreeNode` is built by walking the
/// forest top-down and skipping any parent link that would revisit a
/// node already on the active ancestor chain.
struct TreeNode: Identifiable, Hashable {
    let process: ProcessSummary
    var children: [TreeNode]
    var id: UInt32 { process.pid }
    var depth: Int
}

struct TreeView: View {
    @ObservedObject var state: AppState

    /// Persisted per-node expansion state, keyed by PID. Stored as
    /// comma-separated string of expanded PIDs so we can keep it inside
    /// @AppStorage (which is a String scalar).
    @AppStorage("processes.treeExpanded") private var expandedRaw: String = ""

    @State private var selection: UInt32?

    private var expanded: Set<UInt32> {
        Set(expandedRaw.split(separator: ",").compactMap { UInt32($0) })
    }

    private func setExpanded(_ pid: UInt32, _ on: Bool) {
        var s = expanded
        if on { s.insert(pid) } else { s.remove(pid) }
        expandedRaw = s.sorted().map(String.init).joined(separator: ",")
    }

    private func toggle(_ pid: UInt32) {
        let s = expanded
        setExpanded(pid, !s.contains(pid))
    }

    /// Build the forest. Roots = processes whose ppid is nil, 0, or
    /// points at a PID not in the current set. Cycle protection: a
    /// node is never parented to an ancestor already on its own chain.
    private var roots: [TreeNode] {
        let rows = state.processes
        let byPid: [UInt32: ProcessSummary] = Dictionary(uniqueKeysWithValues: rows.map { ($0.pid, $0) })

        // children: ppid -> [pid] (only valid ppids that exist in the set)
        var kids: [UInt32: [UInt32]] = [:]
        for p in rows {
            guard let pp = p.ppid, byPid[pp] != nil else { continue }
            kids[pp, default: []].append(p.pid)
        }
        // Stable order: sort children by RSS desc within each parent.
        for (k, v) in kids {
            let rss: [UInt32: UInt64] = Dictionary(uniqueKeysWithValues: rows.map { ($0.pid, $0.memory_mb) })
            kids[k] = v.sorted { (rss[$0] ?? 0) > (rss[$1] ?? 0) }
        }

        // Roots: ppid nil/0, or ppid not in byPid.
        let rootPids = rows
            .filter { p in
                guard let pp = p.ppid, pp != 0 else { return true }
                return byPid[pp] == nil
            }
            .map { $0.pid }
            .sorted { (a, b) in
                let ra = byPid[a]?.memory_mb ?? 0
                let rb = byPid[b]?.memory_mb ?? 0
                if ra != rb { return ra > rb }
                return a < b
            }

        func build(_ pid: UInt32, depth: Int, chain: Set<UInt32>) -> TreeNode? {
            guard let p = byPid[pid], !chain.contains(pid) else { return nil }
            let nextChain = chain.union([pid])
            let childPids = kids[pid] ?? []
            let childNodes = childPids.compactMap { build($0, depth: depth + 1, chain: nextChain) }
            return TreeNode(process: p, children: childNodes, depth: depth)
        }

        let topChain: Set<UInt32> = []
        return rootPids.compactMap { build($0, depth: 0, chain: topChain) }
    }

    private var totalRSSBytes: UInt64 {
        state.processes.reduce(UInt64(0)) { $0 + $1.memory_mb * 1024 * 1024 }
    }

    var body: some View {
        VStack(spacing: 0) {
            summaryStrip
            Divider()
            if state.processes.isEmpty {
                EmptyStateView(
                    icon: "list.bullet.indent",
                    title: "No processes to tree",
                    subtitle: "The fleet will render a parent → child tree once processes are registered.",
                    variant: .normal,
                    primaryTitle: "Refresh now",
                    primaryIcon: "arrow.clockwise",
                    primaryAction: { Task { await state.refresh() } }
                )
            } else if roots.isEmpty {
                EmptyStateView(
                    icon: "exclamationmark.triangle",
                    title: "No tree structure",
                    subtitle: "Every process points at a parent that isn't visible — likely a cycle. Showing as a flat list below.",
                    variant: .quiet
                )
                flatFallback
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 2) {
                        ForEach(roots) { node in
                            TreeRowView(
                                node: node,
                                expanded: expanded,
                                selection: $selection,
                                onToggle: { toggle($0) },
                                onSelect: { selection = $0 },
                                totalRSSBytes: totalRSSBytes
                            )
                        }
                    }
                    .padding(10)
                }
            }
        }
    }

    private var summaryStrip: some View {
        let total = state.processes.count
        let orphans = state.processes.filter { p in
            guard let pp = p.ppid, pp != 0 else { return false }
            return !state.processes.contains(where: { $0.pid == pp })
        }.count
        let rootCount = roots.count
        return HStack(spacing: 12) {
            summaryCard("Processes", "\(total)", "fleet", .blue)
            summaryCard("Tree roots", "\(rootCount)", "top-level", .green)
            summaryCard("Orphans", "\(orphans)", "missing ppid", orphans > 0 ? .orange : .secondary)
            summaryCard("Total RSS",
                        ByteCountFormatter.string(fromByteCount: Int64(totalRSSBytes), countStyle: .memory),
                        "fleet", .orange)
        }
        .padding(10)
        .background(.quaternary.opacity(0.5))
    }

    private func summaryCard(_ title: String, _ value: String, _ sub: String, _ color: Color) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(.secondary)
            Text(value).font(.system(.title3, design: .monospaced)).bold().foregroundStyle(color)
            Text(sub).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(8)
        .background(.quaternary)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    /// Last-resort flat list shown when cycle protection eats everything.
    private var flatFallback: some View {
        VStack(alignment: .leading, spacing: 2) {
            ForEach(state.processes.sorted { $0.pid < $1.pid }) { p in
                TreeLeafRow(
                    process: p,
                    isSelected: selection == p.pid,
                    onSelect: { _ in selection = p.pid },
                    totalRSSBytes: totalRSSBytes
                )
            }
        }
        .padding(10)
    }
}

/// Single row in the tree. Renders indent based on `node.depth`, an
/// expand/collapse chevron when there are children, and the process
/// summary (name + badges + RSS).
struct TreeRowView: View {
    let node: TreeNode
    let expanded: Set<UInt32>
    @Binding var selection: UInt32?
    let onToggle: (UInt32) -> Void
    let onSelect: (UInt32) -> Void
    let totalRSSBytes: UInt64

    private var isExpanded: Bool {
        // Default-expand root nodes (depth 0) for the cold-start case
        // if the user hasn't persisted any state yet. Once the user
        // has touched a row, the persisted set drives everything.
        if expanded.isEmpty { return node.depth < 2 }
        return expanded.contains(node.process.pid)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 6) {
                // Indent gutter
                ForEach(0..<node.depth, id: \.self) { _ in
                    Rectangle()
                        .fill(.quaternary)
                        .frame(width: 1, height: 16)
                        .padding(.leading, 7)
                }
                // Expand/collapse chevron
                if node.children.isEmpty {
                    Image(systemName: "circle.fill")
                        .font(.system(size: 4))
                        .foregroundStyle(.quaternary)
                        .frame(width: 16)
                } else {
                    Button { onToggle(node.process.pid) } label: {
                        Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                            .font(.system(size: 10, weight: .semibold))
                            .frame(width: 16, height: 16)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderless)
                }
                TreeLeafRow(
                    process: node.process,
                    isSelected: selection == node.process.pid,
                    onSelect: onSelect,
                    totalRSSBytes: totalRSSBytes
                )
            }
            if isExpanded && !node.children.isEmpty {
                ForEach(node.children) { child in
                    TreeRowView(
                        node: child,
                        expanded: expanded,
                        selection: $selection,
                        onToggle: onToggle,
                        onSelect: onSelect,
                        totalRSSBytes: totalRSSBytes
                    )
                }
            }
        }
    }
}

/// Leaf row content (used by both the tree row and the cycle fallback).
struct TreeLeafRow: View {
    let process: ProcessSummary
    let isSelected: Bool
    let onSelect: (UInt32) -> Void
    let totalRSSBytes: UInt64

    private var rssFraction: Double {
        guard totalRSSBytes > 0 else { return 0 }
        return Double(process.memory_mb * 1024 * 1024) / Double(totalRSSBytes)
    }

    var body: some View {
        HStack(spacing: 8) {
            Text("\(process.pid)")
                .font(.system(.caption, design: .monospaced))
                .frame(width: 50, alignment: .leading)
                .foregroundStyle(.secondary)
            Text(process.name)
                .font(.system(.caption, design: .monospaced))
                .frame(width: 140, alignment: .leading)
                .lineLimit(1)
            if let proj = process.project { Badge(text: proj, color: .blue) }
            if let h = process.harness { Badge(text: h, color: .purple) }
            if let pp = process.ppid {
                Text("ppid \(pp)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 6)
            // Tiny RSS bar
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2).fill(.quaternary)
                    RoundedRectangle(cornerRadius: 2)
                        .fill(rssColor(process.memory_mb).opacity(0.85))
                        .frame(width: max(2, geo.size.width * CGFloat(rssFraction)))
                }
            }
            .frame(width: 60, height: 6)
            Text("\(process.memory_mb) MB")
                .font(.system(.caption2, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(width: 64, alignment: .trailing)
            Text(String(format: "%.1f%%", process.cpu_percent))
                .font(.system(.caption2, design: .monospaced))
                .foregroundStyle(cpuColor(process.cpu_percent))
                .frame(width: 48, alignment: .trailing)
        }
        .padding(.vertical, 3)
        .padding(.horizontal, 6)
        .background(isSelected ? Color.accentColor.opacity(0.18) : Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .contentShape(Rectangle())
        .onTapGesture { onSelect(process.pid) }
    }

    private func rssColor(_ mb: UInt64) -> Color {
        if mb > 1024 { return .red }
        if mb > 512 { return .orange }
        if mb > 128 { return .yellow }
        return .blue
    }

    private func cpuColor(_ pct: Float) -> Color {
        if pct > 90 { return .red }
        if pct > 60 { return .orange }
        if pct > 25 { return .yellow }
        return .secondary
    }
}