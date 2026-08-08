import Foundation

/// Process metadata supplied by the Ghostty app-side adapter.
public struct ProcessEvidence: Codable, Equatable, Sendable {
    public let pid: UInt32?
    public let tty: String?
    public let cwd: String
    public let argv: [String]
    public let startedAt: String?

    public init(pid: UInt32?, tty: String?, cwd: String, argv: [String], startedAt: String? = nil) {
        self.pid = pid
        self.tty = tty
        self.cwd = cwd
        self.argv = argv
        self.startedAt = startedAt
    }

    enum CodingKeys: String, CodingKey {
        case pid, tty, cwd, argv
        case startedAt = "started_at"
    }
}

/// Stable identity and process evidence for one Ghostty split/pane surface.
public struct SurfaceRecord: Codable, Equatable, Sendable {
    public let id: String
    public let terminal: String
    public let title: String?
    public let cwd: String
    public let process: ProcessEvidence?

    public init(id: String, terminal: String = "ghostty", title: String?, cwd: String, process: ProcessEvidence?) {
        self.id = id
        self.terminal = terminal
        self.title = title
        self.cwd = cwd
        self.process = process
    }
}

/// I/O and durability capabilities reported for one surface.
public struct SurfaceCapabilities: Codable, Equatable, Sendable {
    public let read: Bool
    public let write: Bool
    public let resize: Bool
    public let layout: Bool
    public let durablePty: Bool

    public init(read: Bool, write: Bool, resize: Bool, layout: Bool, durablePty: Bool) {
        self.read = read
        self.write = write
        self.resize = resize
        self.layout = layout
        self.durablePty = durablePty
    }

    enum CodingKeys: String, CodingKey {
        case read, write, resize, layout
        case durablePty = "durable_pty"
    }
}

/// Provider implemented by the Ghostty app-side binding.
///
/// The provider owns all app/PTY references. The dispatcher only validates
/// requests and transports typed values; it never executes shell text.
public protocol SurfaceProvider: Sendable {
    func listSurfaces() async throws -> [SurfaceRecord]
    func send(surfaceID: String, bytes: [UInt8]) async throws
    func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8]
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws
    func capabilities(surfaceID: String) async throws -> SurfaceCapabilities
}

public enum ControlError: Error, Equatable, Sendable {
    case invalidRequest(String)
    case invalidParams(String)
    case methodNotFound(String)
    case unauthorized
    case provider(String)
    case requestTooLarge
    case liveIO(String)
}

extension ControlError {
    var code: Int {
        switch self {
        case .invalidRequest: return -32600
        case .invalidParams: return -32602
        case .methodNotFound: return -32601
        case .unauthorized: return -32001
        case .provider: return -32000
        case .liveIO: return -32000
        case .requestTooLarge: return -32600
        }
    }

    var message: String {
        switch self {
        case let .invalidRequest(message), let .invalidParams(message), let .methodNotFound(message), let .provider(message), let .liveIO(message):
            return message
        case .unauthorized:
            return "invalid control token"
        case .requestTooLarge:
            return "request exceeds 1 MiB limit"
        }
    }
}

/// Newline-delimited JSON-RPC dispatcher used by a native Ghostty socket.
public struct ControlDispatcher: Sendable {
    public static let maxRequestBytes = 1024 * 1024
    public static let maxSendBytes = 64 * 1024
    public static let maxReadBytes = 1024 * 1024

    private let provider: any SurfaceProvider
    private let expectedToken: String?
    public let liveEvents: LiveIOEventHub?

    public init(
        provider: any SurfaceProvider,
        expectedToken: String? = nil,
        liveEvents: LiveIOEventHub? = nil
    ) {
        self.provider = provider
        self.expectedToken = expectedToken
        self.liveEvents = liveEvents
    }

    /// Dispatch one complete JSON request and return one JSON response line.
    public func dispatch(_ line: Data) async -> Data {
        do {
            guard line.count <= Self.maxRequestBytes else { throw ControlError.requestTooLarge }
            guard let object = try JSONSerialization.jsonObject(with: line) as? [String: Any] else {
                throw ControlError.invalidRequest("request must be a JSON object")
            }
            let id = object["id"] ?? NSNull()
            guard object["jsonrpc"] as? String == "2.0" else {
                throw ControlError.invalidRequest("jsonrpc must be \"2.0\"")
            }
            if let expectedToken {
                guard object["token"] as? String == expectedToken else { throw ControlError.unauthorized }
            }
            guard let method = object["method"] as? String, !method.isEmpty else {
                throw ControlError.invalidRequest("method is required")
            }
            let isNotification = object["id"] == nil
            let params: [String: Any]
            if let rawParams = object["params"] {
                guard let objectParams = rawParams as? [String: Any] else {
                    throw ControlError.invalidParams("params must be a JSON object")
                }
                params = objectParams
            } else {
                params = [:]
            }
            let result = try await dispatch(method: method, params: params)
            if isNotification { return Data() }
            return encode(["jsonrpc": "2.0", "id": id, "result": result])
        } catch let error as ControlError {
            if !requestHasID(line) { return Data() }
            return encode(["jsonrpc": "2.0", "id": requestID(from: line), "error": ["code": error.code, "message": error.message]])
        } catch let error as LiveIOError {
            if !requestHasID(line) { return Data() }
            return encode(["jsonrpc": "2.0", "id": requestID(from: line), "error": ["code": -32602, "message": String(describing: error)]])
        } catch {
            if !requestHasID(line) { return Data() }
            return encode(["jsonrpc": "2.0", "id": requestID(from: line), "error": ["code": -32700, "message": "parse error: \(error)"]])
        }
    }

    private func dispatch(method: String, params: [String: Any]) async throws -> Any {
        switch method {
        case "surface.list":
            return try jsonObject(await provider.listSurfaces())
        case "surface.io.send":
            let surfaceID = try stringParam(params, "surface_id")
            let text = params["text"] as? String
            let bytes = params["bytes"] as? [Any]
            guard (text != nil) != (bytes != nil) else {
                throw ControlError.invalidParams("exactly one of params.text or params.bytes is required")
            }
            let payload = try text.map { Array($0.utf8) } ?? bytesToUInt8(bytes!)
            guard payload.count <= Self.maxSendBytes else {
                throw ControlError.invalidParams("payload must not exceed 65536 bytes")
            }
            try await provider.send(surfaceID: surfaceID, bytes: payload)
            return NSNull()
        case "surface.io.read":
            let surfaceID = try stringParam(params, "surface_id")
            let maxBytes = try intParam(params, "max_bytes")
            guard maxBytes >= 0 && maxBytes <= Self.maxReadBytes else {
                throw ControlError.invalidParams("max_bytes must be between 0 and 1048576")
            }
            let bytes = try await provider.read(surfaceID: surfaceID, maxBytes: maxBytes)
            guard bytes.count <= maxBytes else {
                throw ControlError.provider("surface provider returned more bytes than requested")
            }
            return ["bytes": bytes]
        case "surface.io.resize":
            let surfaceID = try stringParam(params, "surface_id")
            let rows = try uint16Param(params, "rows")
            let cols = try uint16Param(params, "cols")
            guard rows > 0 && cols > 0 else {
                throw ControlError.invalidParams("rows and cols must be greater than zero")
            }
            try await provider.resize(surfaceID: surfaceID, rows: rows, cols: cols)
            return NSNull()
        case "surface.io.capabilities":
            return try jsonObject(await provider.capabilities(surfaceID: stringParam(params, "surface_id")))
        case "surface.io.subscribe":
            guard let liveEvents else { throw ControlError.liveIO("live surface events unavailable") }
            let surfaceID = params["surface_id"] as? String
            let fromSequence = try optionalUInt64Param(params, "from_seq")
            let maxChunkBytes = try intParamOrDefault(params, "max_chunk_bytes", default: LiveIOEventHub.maxChunkBytes)
            let queueCapacity = try intParamOrDefault(params, "queue_capacity", default: 64)
            let subscription = try await liveEvents.subscribe(
                surfaceID: surfaceID,
                fromSequence: fromSequence,
                maxChunkBytes: maxChunkBytes,
                queueCapacity: queueCapacity
            )
            let nextSequence = await liveEvents.nextSequenceNumber()
            return [
                "subscription_id": subscription.id,
                "next_seq": max(nextSequence, fromSequence ?? 0),
                "capabilities": [
                    "max_chunk_bytes": maxChunkBytes,
                    "queue_capacity": queueCapacity,
                    "replay": false,
                ],
            ]
        case "surface.io.unsubscribe":
            guard let liveEvents else { throw ControlError.liveIO("live surface events unavailable") }
            let subscriptionID = try uint64Param(params, "subscription_id")
            return ["unsubscribed": await liveEvents.unsubscribe(subscriptionID: subscriptionID)]
        default:
            throw ControlError.methodNotFound(method)
        }
    }

    private func stringParam(_ params: [String: Any], _ name: String) throws -> String {
        guard let value = params[name] as? String, !value.isEmpty else {
            throw ControlError.invalidParams("params.\(name) is required")
        }
        return value
    }

    private func intParam(_ params: [String: Any], _ name: String) throws -> Int {
        guard let value = strictInteger(params[name]) else {
            throw ControlError.invalidParams("params.\(name) must be an integer")
        }
        return value.intValue
    }

    private func uint16Param(_ params: [String: Any], _ name: String) throws -> UInt16 {
        let value = try intParam(params, name)
        guard value >= 0 && value <= Int(UInt16.max) else { throw ControlError.invalidParams("params.\(name) is out of range") }
        return UInt16(value)
    }

    private func uint64Param(_ params: [String: Any], _ name: String) throws -> UInt64 {
        let value = try intParam(params, name)
        guard value >= 0 else { throw ControlError.invalidParams("params.\(name) is out of range") }
        return UInt64(value)
    }

    private func optionalUInt64Param(_ params: [String: Any], _ name: String) throws -> UInt64? {
        guard params[name] != nil else { return nil }
        return try uint64Param(params, name)
    }

    private func intParamOrDefault(_ params: [String: Any], _ name: String, default value: Int) throws -> Int {
        guard params[name] != nil else { return value }
        return try intParam(params, name)
    }

    private func bytesToUInt8(_ values: [Any]) throws -> [UInt8] {
        try values.map { value in
            guard let number = strictInteger(value), number.intValue >= 0 && number.intValue <= 255 else {
                throw ControlError.invalidParams("params.bytes must contain integers from 0 to 255")
            }
            return UInt8(number.intValue)
        }
    }

    private func strictInteger(_ value: Any?) -> NSNumber? {
        guard let number = value as? NSNumber else { return nil }
        let type = String(cString: number.objCType)
        guard ["i", "s", "l", "q", "I", "S", "L", "Q"].contains(type) else { return nil }
        return number
    }

    private func jsonObject<T: Encodable>(_ value: T) throws -> Any {
        let data = try JSONEncoder().encode(value)
        return try JSONSerialization.jsonObject(with: data)
    }

    private func encode(_ object: [String: Any]) -> Data {
        (try? JSONSerialization.data(withJSONObject: object, options: [])) ?? Data("{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"encoding failure\"}}".utf8)
    }

    private func requestID(from line: Data) -> Any {
        ((try? JSONSerialization.jsonObject(with: line) as? [String: Any])?["id"]) ?? NSNull()
    }

    private func requestHasID(_ line: Data) -> Bool {
        guard let object = try? JSONSerialization.jsonObject(with: line) as? [String: Any] else {
            return false
        }
        return object["id"] != nil
    }
}
