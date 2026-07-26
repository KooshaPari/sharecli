/// AppState.swift — Observable state for the tray popover + main window.
///
/// Polls the IPC server on `TrayPoll.intervalSeconds` cadence for live data via a single
/// `monitoring.report` snapshot (AC-007.48 / AC-007.72): gate/host_watch + process inventory
/// + embedded pool/status in one round-trip.
///
/// Also maintains two rolling in-memory windows used by the dashboard:
///   - `hostWatchHistory` — last `hostWatchHistoryCap` samples of `HostResourceWatchJson`
///     (PR 6 of the dashboard expansion: Host watch sparklines)
///   - `gateDecisionHistory` — last `gateDecisionHistoryCap` gate decisions, each tagged
///     with the wall-clock time it was observed (PR 6 of the dashboard expansion: thermal
///     gate panel "last 20 gate decisions" log)

import Foundation
import Combine

/// One sample of host resource watch telemetry. Captured on every successful
/// `monitoring.report` poll and pushed onto `AppState.hostWatchHistory`.
///
/// Field semantics match `HostResourceWatchJson` (the wire shape from the
/// sidecar's `monitoring.report` envelope).
public struct HostWatchSample: Identifiable, Hashable {
    public var id: Date { timestamp }
    public let timestamp: Date
    public let fd_count: UInt64
    public let net_rx_bytes: UInt64
    public let net_tx_bytes: UInt64
    public let mem_rss_bytes: UInt64
    public let load_1m: Double

    public init(
        timestamp: Date,
        fd_count: UInt64,
        net_rx_bytes: UInt64,
        net_tx_bytes: UInt64,
        mem_rss_bytes: UInt64,
        load_1m: Double
    ) {
        self.timestamp = timestamp
        self.fd_count = fd_count
        self.net_rx_bytes = net_rx_bytes
        self.net_tx_bytes = net_tx_bytes
        self.mem_rss_bytes = mem_rss_bytes
        self.load_1m = load_1m
    }

    public init(timestamp: Date, host: HostResourceWatchJson) {
        self.init(
            timestamp: timestamp,
            fd_count: host.fd_count,
            net_rx_bytes: host.net_rx_bytes,
            net_tx_bytes: host.net_tx_bytes,
            mem_rss_bytes: host.mem_rss_bytes,
            load_1m: host.load_1m
        )
    }
}

/// One observed gate decision. Captured on every successful poll when the
/// gate decision string is non-empty (it can be "UNAVAILABLE" when the
/// sidecar is offline — those are recorded too).
public struct GateDecisionSample: Identifiable, Hashable {
    public var id: Date { timestamp }
    public let timestamp: Date
    public let thermalPressure: String
    public let detectedAgents: Int
    public let agentTotalRssBytes: UInt64
    public let agentContention: String
    public let gateDecision: String

    public init(timestamp: Date, gate: GateStatusSnapshot) {
        self.timestamp = timestamp
        self.thermalPressure = gate.thermal_pressure
        self.detectedAgents = gate.detected_agents
        self.agentTotalRssBytes = gate.agent_total_rss_bytes
        self.agentContention = gate.agent_contention
        self.gateDecision = gate.gate_decision
    }
}

@MainActor
public final class AppState: ObservableObject {
    /// Cap for the host watch rolling window. The spec calls for a 60s × 1s
    /// window (60 entries); TrayPoll polls every 3s so the actual window is
    /// ~3 minutes of samples. The cap keeps memory bounded either way.
    public static let hostWatchHistoryCap = 60

    /// Cap for the gate decision rolling window (last 20 decisions, per the
    /// Health page Thermal gate panel).
    public static let gateDecisionHistoryCap = 20

    @Published public var processes: [ProcessSummary] = []
    @Published public var health: HealthSnapshot?
    @Published public var poolStatus: PoolSnapshot?
    @Published public var statusSnapshot: StatusSnapshot?
    @Published public var poolEffectiveness: PoolEffectivenessSnapshot?
    @Published public var lastError: String?
    @Published public var isConnected: Bool = false

    /// Rolling window of host resource watch samples (most recent last).
    @Published public var hostWatchHistory: [HostWatchSample] = []

    /// Rolling window of gate decision observations (most recent last).
    @Published public var gateDecisionHistory: [GateDecisionSample] = []

    /// Rolling window of pool effectiveness samples (most recent last).
    /// Cap mirrors host_watch history; effectiveness samples are cheap to keep.
    public static let poolEffectivenessHistoryCap = 60
    @Published public var poolEffectivenessHistory: [PoolEffectivenessSnapshot] = []

    /// Per-agent RSS history ring buffer (PR 5 of dashboard expansion plan).
    /// Each value is an RSS sample in bytes, taken from successive
    /// `monitoring.report` polls. Capped at `agentRSSHistoryCap` per PID.
    /// Older entries are evicted FIFO.
    public static let agentRSSHistoryCap = 60
    @Published public var agentRSSHistory: [UInt32: [UInt64]] = [:]

    /// Cmdline cache (PR 5 of dashboard expansion plan). Keyed by PID so
    /// navigating between agents on the Agents page doesn't re-fetch
    /// the same cmdline. TTL-free for now — the entry stays until the
    /// PID disappears from the agents list (then we evict).
    @Published public var cmdlineCache: [UInt32: ProcessCmdline] = [:]

    private let client = IPCClient.defaultClient()
    private var pollTask: Task<Void, Never>?

    public init() {}

    public func startPolling() {
        pollTask?.cancel()
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                try? await Task.sleep(nanoseconds: TrayPoll.intervalNanoseconds)
            }
        }
    }

    public func stopPolling() {
        pollTask?.cancel()
        pollTask = nil
    }

    public func refresh() async {
        do {
            let report = try await client.monitoringReport()
            let effectiveness = (try? await client.poolEffectiveness())
            let now = Date()
            processes = report.asProcessSummaries()
            health = report.asHealthSnapshot()
            poolStatus = report.pool
            statusSnapshot = report.status
            poolEffectiveness = effectiveness
            isConnected = true
            lastError = nil

            // Rolling-window maintenance for the dashboard's PR 6 panels.
            // We capture the timestamp at the moment the snapshot lands so the
            // sparkline x-axis is wall-clock-correct (not the sidecar's
            // `report.timestamp`, which is the moment the sidecar produced the
            // snapshot and can lag the poll by a few hundred ms).
            hostWatchHistory.append(HostWatchSample(timestamp: now, host: report.host_watch))
            if hostWatchHistory.count > Self.hostWatchHistoryCap {
                hostWatchHistory.removeFirst(hostWatchHistory.count - Self.hostWatchHistoryCap)
            }
            gateDecisionHistory.append(GateDecisionSample(timestamp: now, gate: report.gate))
            if gateDecisionHistory.count > Self.gateDecisionHistoryCap {
                gateDecisionHistory.removeFirst(gateDecisionHistory.count - Self.gateDecisionHistoryCap)
            }
            // Pool effectiveness rolling window — captured separately because
            // effectiveness is its own IPC round-trip. Cheap; same cap as
            // host watch.
            if let eff = effectiveness {
                poolEffectivenessHistory.append(eff)
                if poolEffectivenessHistory.count > Self.poolEffectivenessHistoryCap {
                    poolEffectivenessHistory.removeFirst(
                        poolEffectivenessHistory.count - Self.poolEffectivenessHistoryCap
                    )
                }
            }

            // Per-agent RSS history ring buffer (PR 5). Each agent's RSS
            // is appended to its own per-PID array. New PIDs get a fresh
            // array; PIDs that have left the agents list have their entry
            // evicted from the agentRSSHistory map (and their cmdline
            // cache entry too).
            let livePIDs = Set(report.status.agents.map { $0.pid })
            for agent in report.status.agents {
                var series = agentRSSHistory[agent.pid] ?? []
                series.append(agent.mem_rss_bytes)
                if series.count > Self.agentRSSHistoryCap {
                    series.removeFirst(series.count - Self.agentRSSHistoryCap)
                }
                agentRSSHistory[agent.pid] = series
            }
            let stalePIDs = Set(agentRSSHistory.keys).subtracting(livePIDs)
            for pid in stalePIDs {
                agentRSSHistory.removeValue(forKey: pid)
                cmdlineCache.removeValue(forKey: pid)
            }

            NotificationCenter.default.post(
                name: .sharecliHealthChanged,
                object: health
            )
        } catch {
            isConnected = false
            lastError = error.localizedDescription
            poolStatus = nil
            statusSnapshot = nil
            // On IPC failure we keep the rolling windows so the last-known
            // sparklines remain visible. Clearing them would flash empty.
            NotificationCenter.default.post(
                name: .sharecliHealthChanged,
                object: nil
            )
        }
    }

    public func kill(pid: UInt32) async {
        do {
            try await client.kill(pid: pid)
            await refresh()
        } catch {
            lastError = "kill \(pid): \(error.localizedDescription)"
        }
    }

    public func killAll() async {
        do {
            try await client.killAll()
            await refresh()
        } catch {
            lastError = "kill_all: \(error.localizedDescription)"
        }
    }

    /// Fetch the cmdline for `pid` if not already cached. Returns the
    /// cached value on subsequent calls. Returns `nil` if the sidecar
    /// couldn't read the cmdline (process gone, non-Linux platform).
    public func fetchCmdlineIfNeeded(pid: UInt32) async -> ProcessCmdline? {
        if let cached = cmdlineCache[pid] { return cached }
        do {
            let snap = try await client.fetchCmdline(pid: pid)
            if let snap = snap {
                cmdlineCache[pid] = snap
            }
            return snap
        } catch {
            lastError = "process.cmdline \(pid): \(error.localizedDescription)"
            return nil
        }
    }

    // MARK: - Config

    public func setConfig(key: String, value: AnyCodable) async {
        do {
            try await client.setConfig(key: key, value: value)
        } catch {
            lastError = "config.set \(key): \(error.localizedDescription)"
        }
    }

    /// Fetch the live config from the sidecar (PR 7 of dashboard expansion plan).
    /// Returns the raw JSON `Data` — the page parses it for the form + JSON preview.
    public func getConfig() async -> Data? {
        do {
            return try await client.getConfig()
        } catch {
            lastError = "config.get: \(error.localizedDescription)"
            return nil
        }
    }
}

public extension Notification.Name {
    /// Posted on the main thread whenever AppState.refresh() completes
    /// (carries a HealthSnapshot? as object — nil when IPC disconnected).
    static let sharecliHealthChanged = Notification.Name("sharecliHealthChanged")
}