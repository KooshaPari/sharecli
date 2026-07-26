/// ConfigPage.swift — full configuration editor (PR 7 of dashboard expansion plan).
///
/// Replaces the bare `ConfigEditorView` that had hardcoded `@State` defaults —
/// this version fetches the live config via `config.get` IPC on appear and
/// keeps the form in sync.
///
/// Layout:
///   ┌──────────────────────────────────────────────────────────────┐
///   │ Sectioned form (Runtime / Pool / Monitoring / Spawn / Ports  │
///   │  / Paths / Defaults / Cast / Serve / Health checks)          │
///   │  — every key is a row: label · current value · input         │
///   │  — apply-on-submit (Enter) or apply-on-toggle                │
///   ├──────────────────────────────────────────────────────────────┤
///   │ Live JSON preview (always shows current in-effect config)    │
///   └──────────────────────────────────────────────────────────────┘
///
/// Persistence: the form state re-fetches on appear; nothing cached.

import SwiftUI
import ShareCLICore

struct ConfigPage: View {
    @ObservedObject var state: AppState

    @State private var liveJSON: String = ""
    @State private var parsedTree: [String: AnyCodable] = [:]
    @State private var applyStatus: String = ""
    @State private var loading: Bool = false
    @State private var defaultHarness: String = "claude"
    @State private var poolEnabled: Bool = true
    @State private var maxPerType: String = "5"
    @State private var idleTimeoutSecs: String = "300"
    @State private var maxMemoryMB: String = "4096"
    @State private var maxProcesses: String = "100"
    @State private var healthCheckInterval: String = "30"
    @State private var highMemThreshold: String = "4096"
    @State private var ipcPort: String = "7820"
    @State private var binRoot: String = ""

    var body: some View {
        HStack(spacing: 0) {
            formPanel
            .frame(minWidth: 460, idealWidth: 520)
            jsonPreviewPanel
            .frame(minWidth: 360, idealWidth: 420)
        }
        .frame(minWidth: 820, minHeight: 480)
        .task {
            await loadConfig()
        }
    }

    // MARK: - Form panel

    private var formPanel: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                HStack {
                    Text("Configuration")
                        .font(.largeTitle).bold()
                    Spacer()
                    Button {
                        Task { await loadConfig() }
                    } label: {
                        HStack(spacing: 4) {
                            if loading { ProgressView().scaleEffect(0.5).frame(width: 12, height: 12) }
                            Image(systemName: "arrow.clockwise")
                            Text("Refresh")
                        }
                    }
                    .help("Re-fetch the live config from the sidecar")
                }

                statusRow

                section("Runtime") {
                    row("max_memory_mb", binding: $maxMemoryMB, key: "runtime.max_memory_mb", asInt: true, hint: "Memory cap (MB) for managed processes")
                    row("max_processes", binding: $maxProcesses, key: "runtime.max_processes", asInt: true, hint: "Hard cap on concurrent processes")
                }

                section("Process Pool") {
                    Toggle("Enabled", isOn: $poolEnabled)
                        .onChange(of: poolEnabled) { _, v in apply("pool.enabled", value: .bool(v)) }
                    row("max_per_type", binding: $maxPerType, key: "pool.max_per_type", asInt: true, hint: "Max instances per harness type")
                    row("idle_timeout_secs", binding: $idleTimeoutSecs, key: "pool.idle_timeout_secs", asInt: true, hint: "Seconds before idle pool socket is reaped")
                }

                section("Monitoring") {
                    row("health_check_interval_secs", binding: $healthCheckInterval, key: "monitoring.health_check_interval_secs", asInt: true, hint: "Background poller cadence")
                    row("high_memory_threshold_mb", binding: $highMemThreshold, key: "monitoring.high_memory_threshold_mb", asInt: true, hint: "Above this, monitor emits a warning")
                }

                section("Spawn") {
                    HStack {
                        Text("default_harness")
                            .font(.system(.body, design: .monospaced))
                            .frame(width: 240, alignment: .leading)
                        Picker("", selection: $defaultHarness) {
                            ForEach(["claude", "forge", "node", "bun"], id: \.self) { Text($0) }
                        }
                        .labelsHidden()
                        .frame(width: 140)
                        .onChange(of: defaultHarness) { _, v in apply("spawn.default_harness", value: .string(v)) }
                    }
                }

                section("Ports") {
                    row("ipc_port", binding: $ipcPort, key: "port.ipc_port", asInt: true, hint: "Unix socket path (set via env SHARECLI_IPC_SOCK at runtime)")
                }

                section("Paths") {
                    row("bin_root", binding: $binRoot, key: "paths.bin_root", hint: "Directory sharecli looks for harness binaries")
                }

                section("Defaults (per-harness)") {
                    Text("Hardcoded fallbacks for each harness type. These are loaded when no per-harness config.toml override exists.")
                        .font(.caption2).foregroundStyle(.secondary)
                    defaultsPanel
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

    // MARK: - Defaults panel

    private var defaultsPanel: some View {
        let defaults = parsedTree["defaults"]?.objectValue ?? [:]
        let entries = defaults.sorted { $0.key < $1.key }
        return VStack(alignment: .leading, spacing: 6) {
            if entries.isEmpty {
                Text("No defaults configured.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(entries, id: \.key) { (key, value) in
                    let display = prettyJSON(value)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(key)
                            .font(.system(.caption, design: .monospaced).bold())
                        Text(display)
                            .font(.system(.caption2, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                            .padding(6)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(.quaternary)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                    }
                }
            }
        }
    }

    // MARK: - JSON preview panel

    private var jsonPreviewPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Live config")
                    .font(.headline)
                Spacer()
                Button {
                    Task {
                        if let data = await state.getConfig() {
                            liveJSON = String(data: data, encoding: .utf8) ?? ""
                        }
                    }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .buttonStyle(.borderless)
                .help("Re-fetch")
                Button {
                    let pb = NSPasteboard.general
                    pb.clearContents()
                    pb.setString(liveJSON, forType: .string)
                } label: {
                    Image(systemName: "doc.on.doc")
                }
                .buttonStyle(.borderless)
                .help("Copy config to clipboard")
            }
            .padding(.horizontal, 12)
            .padding(.top, 12)

            ScrollView {
                Text(liveJSON.isEmpty ? "(no config yet)" : liveJSON)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(10)
            }
            .background(.quaternary.opacity(0.5))
        }
        .background(.background)
    }

    // MARK: - Helpers

    private var statusRow: some View {
        HStack(spacing: 8) {
            if loading {
                ProgressView().scaleEffect(0.5).frame(width: 12, height: 12)
            } else {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
            }
            Text("In-effect config is live-edited (Enter to apply, toggles apply instantly).")
                .font(.caption).foregroundStyle(.secondary)
            Spacer()
        }
    }

    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundStyle(.secondary)
            Divider()
            content()
        }
    }

    private func row(_ label: String, binding: Binding<String>, key: String, asInt: Bool = false, hint: String = "") -> some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack {
                Text(label)
                    .font(.system(.body, design: .monospaced))
                    .frame(width: 240, alignment: .leading)
                TextField("", text: binding)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 180)
                    .onSubmit {
                        if asInt, let i = Int(binding.wrappedValue) {
                            apply(key, value: .int(i))
                        } else {
                            apply(key, value: .string(binding.wrappedValue))
                        }
                    }
                if !hint.isEmpty {
                    Text(hint)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    private func loadConfig() async {
        loading = true
        defer { loading = false }
        guard let data = await state.getConfig() else {
            liveJSON = "(no config — sidecar offline)"
            parsedTree = [:]
            return
        }
        liveJSON = String(data: data, encoding: .utf8) ?? ""
        if let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            parsedTree = AnyCodable.from(jsonObject: obj)
        }
        // Hydrate form fields from the live tree.
        if let runtime = parsedTree["runtime"]?.objectValue {
            if let v = runtime["max_memory_mb"]?.intValue { maxMemoryMB = "\(v)" }
            if let v = runtime["max_processes"]?.intValue { maxProcesses = "\(v)" }
        }
        if let pool = parsedTree["pool"]?.objectValue {
            if let v = pool["enabled"]?.boolValue { poolEnabled = v }
            if let v = pool["max_per_type"]?.intValue { maxPerType = "\(v)" }
            if let v = pool["idle_timeout_secs"]?.intValue { idleTimeoutSecs = "\(v)" }
        }
        if let mon = parsedTree["monitoring"]?.objectValue {
            if let v = mon["health_check_interval_secs"]?.intValue { healthCheckInterval = "\(v)" }
            if let v = mon["high_memory_threshold_mb"]?.intValue { highMemThreshold = "\(v)" }
        }
        if let spawn = parsedTree["spawn"]?.objectValue,
           let v = spawn["default_harness"]?.stringValue {
            defaultHarness = v
        }
        if let port = parsedTree["port"]?.objectValue,
           let v = port["ipc_port"]?.intValue {
            ipcPort = "\(v)"
        }
        if let paths = parsedTree["paths"]?.objectValue,
           let v = paths["bin_root"]?.stringValue {
            binRoot = v
        }
    }

    private func apply(_ key: String, value: AnyCodable) {
        Task {
            await state.setConfig(key: key, value: value)
            applyStatus = "Applied: \(key)"
            await loadConfig()
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            applyStatus = ""
        }
    }

    private func prettyJSON(_ value: AnyCodable) -> String {
        do {
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            let data = try encoder.encode(value)
            return String(data: data, encoding: .utf8) ?? ""
        } catch {
            return String(describing: value)
        }
    }
}

// MARK: - AnyCodable convenience accessors

extension AnyCodable {
    var stringValue: String? {
        if case let .string(s) = self { return s } else { return nil }
    }
    var intValue: Int? {
        switch self {
        case .int(let i): return i
        case .uint(let u): return Int(u)
        case .double(let d): return Int(d)
        default: return nil
        }
    }
    var boolValue: Bool? {
        if case let .bool(b) = self { return b } else { return nil }
    }
    var objectValue: [String: AnyCodable]? {
        if case let .object(o) = self { return o } else { return nil }
    }
    var arrayValue: [AnyCodable]? {
        if case let .array(a) = self { return a } else { return nil }
    }

    /// Build an `AnyCodable` tree from a JSON-deserialized `[String: Any]`.
    static func from(jsonObject: [String: Any]) -> [String: AnyCodable] {
        var out: [String: AnyCodable] = [:]
        for (k, v) in jsonObject {
            out[k] = AnyCodable.from(any: v)
        }
        return out
    }

    static func from(any: Any) -> AnyCodable {
        if let s = any as? String { return .string(s) }
        if let i = any as? Int { return .int(i) }
        if let u = any as? UInt32 { return .uint(u) }
        if let d = any as? Double { return .double(d) }
        if let b = any as? Bool { return .bool(b) }
        if let arr = any as? [Any] { return .array(arr.map { from(any: $0) }) }
        if let obj = any as? [String: Any] { return .object(from(jsonObject: obj)) }
        return .null
    }
}