/// IPCClient.swift — Unix socket NDJSON-RPC client for the sharecli IPC server.
///
/// All calls are async; the caller awaits on a background Task.
/// Thread-safety: each call creates its own socket connection (stateless from
/// the Swift side — the Rust server handles concurrent connections).

import Foundation
import Network

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

public struct IPCRequest: Encodable {
    public let id: Int
    public let method: String
    public let params: [String: AnyCodable]

    public init(id: Int, method: String, params: [String: AnyCodable] = [:]) {
        self.id = id
        self.method = method
        self.params = params
    }
}

public struct IPCResponse<T: Decodable>: Decodable {
    public let id: Int
    public let result: T?
    public let error: String?
}

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

public struct ProcessSummary: Identifiable, Decodable, Hashable {
    public var id: UInt32 { pid }
    public let pid: UInt32
    public let name: String
    public let cmd: [String]
    public let memory_mb: UInt64
    public let project: String?
    public let harness: String?
    public let start_time: UInt64
}

public struct HealthSnapshot: Decodable {
    public let managed_processes: Int
    public let used_memory_mb: UInt64
    public let total_memory_mb: UInt64
    public let healthy: Bool
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

public actor IPCClient {
    private let socketPath: String
    private var nextId: Int = 1

    public init(socketPath: String) {
        self.socketPath = socketPath
    }

    public convenience init() {
        let env = ProcessInfo.processInfo.environment["SHARECLI_IPC_SOCK"]
        let defaultPath = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/sharecli/ipc.sock")
            .path
        self.init(socketPath: env ?? defaultPath)
    }

    private func nextRequestId() -> Int {
        defer { nextId += 1 }
        return nextId
    }

    // MARK: - Public API

    public func listProcesses() async throws -> [ProcessSummary] {
        let resp: IPCResponse<[ProcessSummary]> = try await call(
            method: "process.list", params: [:]
        )
        return resp.result ?? []
    }

    public func kill(pid: UInt32) async throws {
        let _: IPCResponse<Bool> = try await call(
            method: "process.kill", params: ["pid": .uint(pid)]
        )
    }

    public func killAll() async throws {
        let _: IPCResponse<Bool> = try await call(
            method: "process.kill_all", params: [:]
        )
    }

    public func health() async throws -> HealthSnapshot {
        let resp: IPCResponse<HealthSnapshot> = try await call(
            method: "health.status", params: [:]
        )
        guard let snap = resp.result else {
            throw IPCError.nilResult("health.status")
        }
        return snap
    }

    public func getConfig() async throws -> Data {
        let resp: IPCResponse<AnyCodable> = try await call(
            method: "config.get", params: [:]
        )
        guard let raw = resp.result else { throw IPCError.nilResult("config.get") }
        return try JSONEncoder().encode(raw)
    }

    public func setConfig(key: String, value: AnyCodable) async throws {
        let _: IPCResponse<Bool> = try await call(
            method: "config.set", params: ["key": .string(key), "value": value]
        )
    }

    // MARK: - Transport

    private func call<T: Decodable>(
        method: String,
        params: [String: AnyCodable]
    ) async throws -> IPCResponse<T> {
        let id = nextRequestId()
        let req = IPCRequest(id: id, method: method, params: params)
        var payload = try JSONEncoder().encode(req)
        payload.append(contentsOf: [UInt8(ascii: "\n")])

        let sock = socketPath
        return try await withCheckedThrowingContinuation { continuation in
            DispatchQueue.global(qos: .utility).async {
                do {
                    let fd = try Self.openUnixSocket(path: sock)
                    defer { Darwin.close(fd) }

                    // Write request
                    try payload.withUnsafeBytes { buf in
                        var written = 0
                        while written < buf.count {
                            let n = Darwin.write(fd, buf.baseAddress!.advanced(by: written), buf.count - written)
                            guard n > 0 else { throw IPCError.writeFailed }
                            written += n
                        }
                    }

                    // Read until newline
                    var response = Data()
                    var byte = UInt8(0)
                    while true {
                        let n = Darwin.read(fd, &byte, 1)
                        guard n > 0 else { throw IPCError.readFailed }
                        if byte == UInt8(ascii: "\n") { break }
                        response.append(byte)
                    }

                    let decoded = try JSONDecoder().decode(IPCResponse<T>.self, from: response)
                    continuation.resume(returning: decoded)
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }

    private static func openUnixSocket(path: String) throws -> Int32 {
        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { throw IPCError.socketCreate }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
            path.withCString { cstr in
                let dest = UnsafeMutableRawPointer(ptr).assumingMemoryBound(to: CChar.self)
                strncpy(dest, cstr, MemoryLayout.size(ofValue: addr.sun_path) - 1)
            }
        }

        let connectResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sap in
                connect(fd, sap, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }

        guard connectResult == 0 else {
            Darwin.close(fd)
            throw IPCError.connectFailed(path)
        }
        return fd
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

public enum IPCError: LocalizedError {
    case socketCreate
    case connectFailed(String)
    case writeFailed
    case readFailed
    case nilResult(String)

    public var errorDescription: String? {
        switch self {
        case .socketCreate: return "Failed to create Unix socket"
        case .connectFailed(let p): return "Could not connect to sharecli-ipc at \(p)"
        case .writeFailed: return "Socket write failed"
        case .readFailed: return "Socket read failed"
        case .nilResult(let m): return "Nil result from \(m)"
        }
    }
}

// ---------------------------------------------------------------------------
// AnyCodable — lightweight type-erased JSON value
// ---------------------------------------------------------------------------

public enum AnyCodable: Codable {
    case string(String)
    case int(Int)
    case uint(UInt32)
    case double(Double)
    case bool(Bool)
    case array([AnyCodable])
    case object([String: AnyCodable])
    case null

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let v = try? c.decode(String.self) { self = .string(v) }
        else if let v = try? c.decode(Int.self) { self = .int(v) }
        else if let v = try? c.decode(Double.self) { self = .double(v) }
        else if let v = try? c.decode(Bool.self) { self = .bool(v) }
        else if let v = try? c.decode([AnyCodable].self) { self = .array(v) }
        else if let v = try? c.decode([String: AnyCodable].self) { self = .object(v) }
        else { self = .null }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .string(let v): try c.encode(v)
        case .int(let v): try c.encode(v)
        case .uint(let v): try c.encode(v)
        case .double(let v): try c.encode(v)
        case .bool(let v): try c.encode(v)
        case .array(let v): try c.encode(v)
        case .object(let v): try c.encode(v)
        case .null: try c.encodeNil()
        }
    }
}
