import Foundation

/// A single app-owned surface binding.
///
/// The Ghostty fork should construct bindings with closures that capture only
/// weak/actor-safe references to its SurfaceView and termio objects. ShareCLI
/// never stores a raw Ghostty C pointer or invokes shell text through this
/// type. Every operation remains asynchronous so a MainActor-bound adapter can
/// hop to the app actor for a short operation.
public struct SurfaceBinding: Sendable {
    public let record: SurfaceRecord

    private let sendOperation: @Sendable ([UInt8]) async throws -> Void
    private let readOperation: @Sendable (Int) async throws -> [UInt8]
    private let resizeOperation: @Sendable (UInt16, UInt16) async throws -> Void
    private let capabilitiesOperation: @Sendable () async throws -> SurfaceCapabilities

    public init(
        record: SurfaceRecord,
        send: @escaping @Sendable ([UInt8]) async throws -> Void,
        read: @escaping @Sendable (Int) async throws -> [UInt8],
        resize: @escaping @Sendable (UInt16, UInt16) async throws -> Void,
        capabilities: @escaping @Sendable () async throws -> SurfaceCapabilities
    ) {
        self.record = record
        self.sendOperation = send
        self.readOperation = read
        self.resizeOperation = resize
        self.capabilitiesOperation = capabilities
    }

    fileprivate func send(_ bytes: [UInt8]) async throws {
        try await sendOperation(bytes)
    }

    fileprivate func read(maxBytes: Int) async throws -> [UInt8] {
        try await readOperation(maxBytes)
    }

    fileprivate func resize(rows: UInt16, cols: UInt16) async throws {
        try await resizeOperation(rows, cols)
    }

    fileprivate func capabilities() async throws -> SurfaceCapabilities {
        try await capabilitiesOperation()
    }
}

/// Explicit degraded provider used while Ghostty's native surface tree is
/// unavailable (for example, before app readiness or after teardown).
///
/// Keeping the listener alive with an empty, read-only provider lets clients
/// distinguish "control plane is up, surfaces unavailable" from a dead socket.
public struct UnavailableSurfaceProvider: SurfaceProvider, Sendable {
    public let reason: String

    public init(reason: String = "native Ghostty surface provider unavailable") {
        self.reason = reason
    }

    public func listSurfaces() async throws -> [SurfaceRecord] { [] }

    public func send(surfaceID: String, bytes: [UInt8]) async throws {
        throw ControlError.provider(reason)
    }

    public func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8] {
        throw ControlError.provider(reason)
    }

    public func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws {
        throw ControlError.provider(reason)
    }

    public func capabilities(surfaceID: String) async throws -> SurfaceCapabilities {
        SurfaceCapabilities(read: false, write: false, resize: false, layout: false, durablePty: false)
    }
}

/// Actor-isolated registry that adapts Ghostty's live surface tree to the
/// ShareCLI `SurfaceProvider` contract.
///
/// Registration/removal is serialized and list results are deterministic. A
/// binding disappearing during a request produces an explicit provider error;
/// it never falls back to AppleScript, process scraping, or command execution.
public actor SurfaceProviderRegistry: SurfaceProvider {
    private var bindings: [String: SurfaceBinding] = [:]

    public init() {}

    public func register(_ binding: SurfaceBinding) throws {
        guard !binding.record.id.isEmpty else {
            throw ControlError.invalidParams("surface binding id must not be empty")
        }
        guard bindings[binding.record.id] == nil else {
            throw ControlError.provider("surface binding already registered: \(binding.record.id)")
        }
        bindings[binding.record.id] = binding
    }

    @discardableResult
    public func replace(_ binding: SurfaceBinding) throws -> SurfaceBinding? {
        guard !binding.record.id.isEmpty else {
            throw ControlError.invalidParams("surface binding id must not be empty")
        }
        return bindings.updateValue(binding, forKey: binding.record.id)
    }

    @discardableResult
    public func unregister(surfaceID: String) -> Bool {
        bindings.removeValue(forKey: surfaceID) != nil
    }

    public func listSurfaces() async throws -> [SurfaceRecord] {
        bindings.values.map(\.record).sorted { $0.id < $1.id }
    }

    public func send(surfaceID: String, bytes: [UInt8]) async throws {
        try await binding(for: surfaceID).send(bytes)
    }

    public func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8] {
        try await binding(for: surfaceID).read(maxBytes: maxBytes)
    }

    public func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws {
        try await binding(for: surfaceID).resize(rows: rows, cols: cols)
    }

    public func capabilities(surfaceID: String) async throws -> SurfaceCapabilities {
        try await binding(for: surfaceID).capabilities()
    }

    private func binding(for surfaceID: String) throws -> SurfaceBinding {
        guard let binding = bindings[surfaceID] else {
            throw ControlError.provider("surface unavailable: \(surfaceID)")
        }
        return binding
    }
}
