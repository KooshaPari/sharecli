/// DashboardView.swift — full NSWindow dashboard (process table + config editor).
///
/// Navigation sidebar:
///   Processes  → live process table with per-row kill + filter by project/harness
///   Config     → spawn_policy + pool + monitoring config editor with live apply
///   Health     → memory + process count charts

import SwiftUI
import ShareCLICore

struct DashboardView: View {
    @ObservedObject var state: AppState
    @State private var selection: Section = .processes

    enum Section: String, CaseIterable, Identifiable {
        var id: String { rawValue }
        case processes = "Processes"
        case config = "Config"
        case health = "Health"
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
                case .processes: ProcessTableView(state: state)
                case .config: ConfigEditorView(state: state)
                case .health: HealthView(state: state)
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
        case .config: return "gearshape"
        case .health: return "heart.fill"
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

struct ConfigEditorView: View {
    @ObservedObject var state: AppState

    // Pool settings
    @State private var poolEnabled: Bool = true
    @State private var maxPerType: String = "5"
    @State private var idleTimeoutSecs: String = "300"

    // Runtime settings
    @State private var maxMemoryMB: String = "4096"
    @State private var maxProcesses: String = "100"

    // Monitoring settings
    @State private var healthCheckInterval: String = "30"
    @State private var highMemThreshold: String = "4096"

    // Spawn settings
    @State private var defaultHarness: String = "claude"

    @State private var applyStatus: String = ""

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                Text("Configuration")
                    .font(.largeTitle)
                    .bold()
                    .padding(.bottom, 4)

                configSection("Runtime") {
                    row("max_memory_mb", binding: $maxMemoryMB,
                        key: "runtime.max_memory_mb", asInt: true)
                    row("max_processes", binding: $maxProcesses,
                        key: "runtime.max_processes", asInt: true)
                }

                configSection("Process Pool") {
                    Toggle("Enabled", isOn: $poolEnabled)
                        .onChange(of: poolEnabled) { v in apply("pool.enabled", value: .bool(v)) }
                    row("max_per_type", binding: $maxPerType,
                        key: "pool.max_per_type", asInt: true)
                    row("idle_timeout_secs", binding: $idleTimeoutSecs,
                        key: "pool.idle_timeout_secs", asInt: true)
                }

                configSection("Monitoring") {
                    row("health_check_interval_secs", binding: $healthCheckInterval,
                        key: "monitoring.health_check_interval_secs", asInt: true)
                    row("high_memory_threshold_mb", binding: $highMemThreshold,
                        key: "monitoring.high_memory_threshold_mb", asInt: true)
                }

                configSection("Spawn") {
                    HStack {
                        Text("default_harness")
                            .font(.system(.body, design: .monospaced))
                            .frame(width: 200, alignment: .leading)
                        Picker("", selection: $defaultHarness) {
                            ForEach(["claude", "forge", "node", "bun"], id: \.self) { Text($0) }
                        }
                        .labelsHidden()
                        .frame(width: 120)
                        .onChange(of: defaultHarness) { v in apply("spawn.default_harness", value: .string(v)) }
                    }
                }

                if !applyStatus.isEmpty {
                    Text(applyStatus)
                        .font(.caption)
                        .foregroundStyle(applyStatus.hasPrefix("Error") ? .red : .green)
                }
            }
            .padding(24)
        }
    }

    private func configSection<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundStyle(.secondary)
            Divider()
            content()
        }
    }

    private func row(_ label: String, binding: Binding<String>, key: String, asInt: Bool) -> some View {
        HStack {
            Text(label)
                .font(.system(.body, design: .monospaced))
                .frame(width: 240, alignment: .leading)
            TextField("", text: binding)
                .textFieldStyle(.roundedBorder)
                .frame(width: 120)
                .onSubmit {
                    if asInt, let i = Int(binding.wrappedValue) {
                        apply(key, value: .int(i))
                    } else {
                        apply(key, value: .string(binding.wrappedValue))
                    }
                }
        }
    }

    private func apply(_ key: String, value: AnyCodable) {
        Task {
            await state.setConfig(key: key, value: value)
            applyStatus = "Applied: \(key)"
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            applyStatus = ""
        }
    }
}

// MARK: - Health View

struct HealthView: View {
    @ObservedObject var state: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                Text("Health")
                    .font(.largeTitle)
                    .bold()

                if let h = state.health {
                    HStack(spacing: 24) {
                        metricCard(
                            title: "Managed Processes",
                            value: "\(h.managed_processes)",
                            icon: "cpu",
                            color: .blue
                        )
                        metricCard(
                            title: "Used Memory",
                            value: "\(h.used_memory_mb) MB",
                            icon: "memorychip",
                            color: .orange
                        )
                        metricCard(
                            title: "Total Memory",
                            value: "\(h.total_memory_mb) MB",
                            icon: "externaldrive",
                            color: .gray
                        )
                        metricCard(
                            title: "Status",
                            value: h.healthy ? "Healthy" : "Warning",
                            icon: h.healthy ? "checkmark.seal.fill" : "exclamationmark.triangle.fill",
                            color: h.healthy ? .green : .orange
                        )
                    }

                    // Memory bar
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Memory Utilization")
                            .font(.headline)
                        GeometryReader { geo in
                            ZStack(alignment: .leading) {
                                RoundedRectangle(cornerRadius: 6)
                                    .fill(.quaternary)
                                RoundedRectangle(cornerRadius: 6)
                                    .fill(h.used_memory_mb > h.total_memory_mb / 2 ? Color.orange : .blue)
                                    .frame(width: geo.size.width * CGFloat(h.used_memory_mb) / CGFloat(max(h.total_memory_mb, 1)))
                            }
                        }
                        .frame(height: 16)
                        Text("\(h.used_memory_mb) MB / \(h.total_memory_mb) MB")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.top, 8)
                } else {
                    Text(state.isConnected ? "Loading health data…" : "Not connected to sharecli-ipc")
                        .foregroundStyle(.secondary)
                }
            }
            .padding(24)
        }
    }

    private func metricCard(title: String, value: String, icon: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Image(systemName: icon).foregroundStyle(color)
                Text(title).font(.caption).foregroundStyle(.secondary)
            }
            Text(value)
                .font(.system(.title2, design: .monospaced))
                .bold()
        }
        .padding(14)
        .frame(minWidth: 140, alignment: .leading)
        .background(.quaternary)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}
