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
    func listSurfaces() throws -> [SurfaceRecord]
    func send(surfaceID: String, bytes: [UInt8]) throws
    func read(surfaceID: String, maxBytes: Int) throws -> [UInt8]
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) throws
    func capabilities(surfaceID: String) throws -> SurfaceCapabilities
}

public enum ControlError: Error, Equatable, Sendable {
    case invalidRequest(String)
    case invalidParams(String)
    case methodNotFound(String)
    case unauthorized
    case provider(String)
    case requestTooLarge
}

extension ControlError {
    var code: Int {
        switch self {
        case .invalidRequest: return -32600
        case .invalidParams: return -32602
        case .methodNotFound: return -32601
        case .unauthorized: return -32001
        case .provider: return -32000
        case .requestTooLarge: return -32600
        }
    }

    var message: String {
        switch self {
        case let .invalidRequest(message), let .invalidParams(message), let .methodNotFound(message), let .provider(message):
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
    public static let maxReadBytes = 1024 * 1024

    private let provider: any SurfaceProvider
    private let expectedToken: String?

    public init(provider: any SurfaceProvider, expectedToken: String? = nil) {
        self.provider = provider
        self.expectedToken = expectedToken
    }

    /// Dispatch one complete JSON request and return one JSON response line.
    public func dispatch(_ line: Data) -> Data {
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
            let params = object["params"] as? [String: Any] ?? [:]
            let result = try dispatch(method: method, params: params)
            return encode(["jsonrpc": "2.0", "id": id, "result": result])
        } catch let error as ControlError {
            return encode(["jsonrpc": "2.0", "id": requestID(from: line), "error": ["code": error.code, "message": error.message]])
        } catch {
            return encode(["jsonrpc": "2.0", "id": requestID(from: line), "error": ["code": -32700, "message": "parse error: \(error)"]])
        }
    }

    private func dispatch(method: String, params: [String: Any]) throws -> Any {
        switch method {
        case "surface.list":
            return try jsonObject(provider.listSurfaces())
        case "surface.io.send":
            let surfaceID = try stringParam(params, "surface_id")
            let text = params["text"] as? String
            let bytes = params["bytes"] as? [Any]
            guard (text != nil) != (bytes != nil) else {
                throw ControlError.invalidParams("exactly one of params.text or params.bytes is required")
            }
            let payload = try text.map { Array($0.utf8) } ?? bytesToUInt8(bytes!)
            try provider.send(surfaceID: surfaceID, bytes: payload)
            return NSNull()
        case "surface.io.read":
            let surfaceID = try stringParam(params, "surface_id")
            let maxBytes = try intParam(params, "max_bytes")
            guard maxBytes >= 0 && maxBytes <= Self.maxReadBytes else {
                throw ControlError.invalidParams("max_bytes must be between 0 and 1048576")
            }
            return ["bytes": try provider.read(surfaceID: surfaceID, maxBytes: maxBytes)]
        case "surface.io.resize":
            let surfaceID = try stringParam(params, "surface_id")
            let rows = try uint16Param(params, "rows")
            let cols = try uint16Param(params, "cols")
            guard rows > 0 && cols > 0 else {
                throw ControlError.invalidParams("rows and cols must be greater than zero")
            }
            try provider.resize(surfaceID: surfaceID, rows: rows, cols: cols)
            return NSNull()
        case "surface.io.capabilities":
            return try jsonObject(provider.capabilities(surfaceID: stringParam(params, "surface_id")))
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
}
