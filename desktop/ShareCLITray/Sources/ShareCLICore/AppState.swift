/// AppState.swift — Observable state for the tray popover + main window.
///
/// Polls the IPC server every 3 s for live data.

import Foundation
import Combine

@MainActor
public final class AppState: ObservableObject {
    @Published public var processes: [ProcessSummary] = []
    @Published public var health: HealthSnapshot?
    @Published public var lastError: String?
    @Published public var isConnected: Bool = false

    private let client = IPCClient()
    private var pollTask: Task<Void, Never>?

    public init() {}

    public func startPolling() {
        pollTask?.cancel()
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh()
                try? await Task.sleep(nanoseconds: 3_000_000_000)
            }
        }
    }

    public func stopPolling() {
        pollTask?.cancel()
        pollTask = nil
    }

    public func refresh() async {
        do {
            async let procs = client.listProcesses()
            async let snap = client.health()
            let (p, h) = try await (procs, snap)
            processes = p
            health = h
            isConnected = true
            lastError = nil
        } catch {
            isConnected = false
            lastError = error.localizedDescription
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

    // MARK: - Config

    public func setConfig(key: String, value: AnyCodable) async {
        do {
            try await client.setConfig(key: key, value: value)
        } catch {
            lastError = "config.set \(key): \(error.localizedDescription)"
        }
    }
}
