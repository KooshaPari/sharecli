/// CompositeHealth.swift — composite fleet + host health metric (T-95).
///
/// Combines the latest `FleetSample` (processes, memory, CPU, pool
/// health) with the latest `HostWatchSample` (1-minute load average)
/// into a single 0–100 score plus a four-band classification. The
/// intent is to give operators one glanceable number that captures
/// "is the fleet healthy right now?" rather than forcing them to read
/// 10+ separate tiles.
///
/// Scoring model (additive, clamped to 0–100):
///   - Pool health flag (binary): 30 pts if `fleet.poolHealthy`
///   - CPU headroom:              up to 25 pts = (1 − cpu/100) × 25
///   - Memory headroom:           up to 25 pts = (1 − used/total) × 25
///   - Load average headroom:     up to 20 pts = (1 − load/8) × 20
///
/// Band thresholds (inclusive lower bound):
///   80+  healthy · 60–79 watch · 30–59 degraded · <30 critical
///
/// The computation is pure data — no I/O, no AppState touch, no
/// Foundation date — so it can be unit-tested directly from Swift
/// and reused inside SwiftUI without coupling to the polling layer.
///
/// T-95 / WBS — additive file. Does not modify AppState or any
/// existing type. Callers compute on demand from `state.fleetHistory`
/// and `state.hostWatchHistory`.

import Foundation

public struct CompositeHealthMetric: Equatable {
    public let score: Int
    public let band: HealthBand
    public let poolHealthy: Bool
    public let breakdown: Breakdown

    public enum HealthBand: String, Equatable {
        case healthy
        case watch
        case degraded
        case critical

        public var displayName: String {
            switch self {
            case .healthy: return "Healthy"
            case .watch: return "Watch"
            case .degraded: return "Degraded"
            case .critical: return "Critical"
            }
        }
    }

    public struct Breakdown: Equatable {
        public let poolPoints: Int
        public let cpuPoints: Int
        public let memoryPoints: Int
        public let loadPoints: Int
        public let totalPoints: Int
        public let cpuPercent: Float
        public let memUsedMB: UInt64
        public let memTotalMB: UInt64
        public let load1m: Double
        public let hasHostSample: Bool
    }

    public init(
        score: Int,
        band: HealthBand,
        poolHealthy: Bool,
        breakdown: Breakdown
    ) {
        self.score = score
        self.band = band
        self.poolHealthy = poolHealthy
        self.breakdown = breakdown
    }
}

public extension CompositeHealthMetric {
    /// Composite score maximum. All four sub-components sum to <=100 by
    /// construction; this clamp is defensive in case a future sub-metric
    /// is added.
    static let maxScore: Int = 100

    /// CPU percent considered fully saturated (0 pts from CPU).
    static let clampCPUPercent: Float = 100

    /// 1-minute load average considered fully saturated (0 pts from load).
    static let clampLoad1m: Double = 8

    /// Compute a composite metric from the latest fleet + host samples.
    /// Returns `nil` only when no fleet sample exists yet (cold start).
    /// A missing host sample contributes a full 20 load points (we
    /// cannot penalise what we have not measured).
    static func compute(fleet: FleetSample?, host: HostWatchSample?) -> CompositeHealthMetric? {
        guard let fleet else { return nil }

        let poolHealthy = fleet.poolHealthy
        let poolPoints = poolHealthy ? 30 : 0

        let cpuFraction = min(1.0, max(0.0, Double(fleet.cpuAvgPercent) / Double(clampCPUPercent)))
        let cpuPoints = Int(round((1.0 - cpuFraction) * 25))

        let memFraction: Double
        if fleet.totalMemoryMB > 0 {
            memFraction = min(1.0, Double(fleet.usedMemoryMB) / Double(fleet.totalMemoryMB))
        } else {
            memFraction = 0
        }
        let memoryPoints = Int(round((1.0 - memFraction) * 25))

        let hasHost = host != nil
        let loadFraction: Double
        if let host {
            loadFraction = min(1.0, max(0.0, host.load_1m / clampLoad1m))
        } else {
            loadFraction = 0
        }
        let loadPoints = Int(round((1.0 - loadFraction) * 20))

        let total = poolPoints + cpuPoints + memoryPoints + loadPoints
        let score = max(0, min(maxScore, total))

        return CompositeHealthMetric(
            score: score,
            band: band(forScore: score),
            poolHealthy: poolHealthy,
            breakdown: Breakdown(
                poolPoints: poolPoints,
                cpuPoints: cpuPoints,
                memoryPoints: memoryPoints,
                loadPoints: loadPoints,
                totalPoints: total,
                cpuPercent: fleet.cpuAvgPercent,
                memUsedMB: fleet.usedMemoryMB,
                memTotalMB: fleet.totalMemoryMB,
                load1m: host?.load_1m ?? 0,
                hasHostSample: hasHost
            )
        )
    }

    /// Derive a band directly from a 0–100 score. Useful for tests and
    /// for callers that want to re-band a stored metric after the fact.
    static func band(forScore score: Int) -> HealthBand {
        switch score {
        case 80...: return .healthy
        case 60..<80: return .watch
        case 30..<60: return .degraded
        default: return .critical
        }
    }
}
