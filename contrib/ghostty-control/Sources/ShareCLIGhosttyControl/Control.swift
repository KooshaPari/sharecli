import Foundation
import Darwin

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

/// Owner-only newline-delimited Unix-domain listener for a native Ghostty app.
///
/// The listener owns transport and connection lifetime; `SurfaceProvider` owns
/// all app/PTY references. Each request is bounded before it reaches the
/// dispatcher, and each complete request produces exactly one response line.
public final class UnixControlServer: @unchecked Sendable {
    public let path: String

    private static let maxSocketPathBytes = 104
    private let dispatcher: ControlDispatcher
    private let queue: DispatchQueue
    private let lock = NSLock()
    private var listenerFD: Int32 = -1
    private var source: DispatchSourceRead?

    public init(path: String, dispatcher: ControlDispatcher) {
        self.path = path
        self.dispatcher = dispatcher
        self.queue = DispatchQueue(label: "sharecli.ghostty-control", qos: .userInitiated)
    }

    deinit {
        stop()
    }

    public func start() throws {
        lock.lock()
        defer { lock.unlock() }
        guard listenerFD < 0 else { return }
        guard path.utf8.count + 1 <= Self.maxSocketPathBytes else {
            throw ControlError.invalidRequest("control socket path is too long")
        }
        removeExistingSocketIfSafe()

        let fd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { throw socketError("create control socket") }
        do {
            var address = try unixAddress(path)
            let addressLength = socklen_t(MemoryLayout<sockaddr_un>.size)
            let bound = withUnsafePointer(to: &address) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { pointer in
                    Darwin.bind(fd, pointer, addressLength)
                }
            }
            guard bound == 0 else {
                throw socketError("bind control socket")
            }
            guard Darwin.listen(fd, 16) == 0 else { throw socketError("listen control socket") }
            guard Darwin.fcntl(fd, F_SETFL, O_NONBLOCK) == 0 else {
                throw socketError("configure control socket")
            }
            guard Darwin.chmod(path, mode_t(0o600)) == 0 else {
                throw socketError("protect control socket")
            }
            listenerFD = fd
            let source = DispatchSource.makeReadSource(fileDescriptor: fd, queue: queue)
            source.setEventHandler { [weak self] in self?.acceptConnections() }
            source.setCancelHandler { Darwin.close(fd) }
            source.resume()
            self.source = source
        } catch {
            Darwin.close(fd)
            unlink(path)
            throw error
        }
    }

    public func stop() {
        lock.lock()
        let activeSource = source
        source = nil
        listenerFD = -1
        activeSource?.cancel()
        unlink(path)
        lock.unlock()
    }

    private func acceptConnections() {
        while true {
            let fd = Darwin.accept(listenerFD, nil, nil)
            if fd < 0 {
                if errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR { return }
                return
            }
            DispatchQueue.global(qos: .userInitiated).async { [weak self] in
                self?.serveConnection(fd)
            }
        }
    }

    private func serveConnection(_ fd: Int32) {
        defer { Darwin.close(fd) }
        var buffer = Data()
        var chunk = [UInt8](repeating: 0, count: 16 * 1024)
        while true {
            let count = chunk.withUnsafeMutableBytes { rawBuffer in
                Darwin.recv(fd, rawBuffer.baseAddress, rawBuffer.count, 0)
            }
            if count <= 0 { return }
            buffer.append(contentsOf: chunk[0..<count])
            guard buffer.count <= ControlDispatcher.maxRequestBytes else { return }
            while let newline = buffer.firstIndex(of: 0x0a) {
                let line = buffer.prefix(upTo: newline)
                buffer.removeSubrange(...newline)
                var response = dispatcher.dispatch(Data(line))
                response.append(0x0a)
                guard sendAll(fd, response) else { return }
            }
        }
    }

    private func sendAll(_ fd: Int32, _ data: Data) -> Bool {
        data.withUnsafeBytes { rawBuffer in
            guard let base = rawBuffer.baseAddress else { return true }
            var sent = 0
            while sent < data.count {
                let count = Darwin.send(fd, base.advanced(by: sent), data.count - sent, 0)
                if count <= 0 { return false }
                sent += count
            }
            return true
        }
    }

    private func removeExistingSocketIfSafe() {
        var info = stat()
        guard lstat(path, &info) == 0 else { return }
        guard (info.st_mode & S_IFMT) == S_IFSOCK else { return }
        unlink(path)
    }

    private func unixAddress(_ path: String) throws -> sockaddr_un {
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        path.withCString { pointer in
            withUnsafeMutableBytes(of: &address.sun_path) { destination in
                destination.copyBytes(from: UnsafeRawBufferPointer(start: pointer, count: path.utf8.count + 1))
            }
        }
        return address
    }

    private func socketError(_ operation: String) -> ControlError {
        .provider("\(operation): \(String(cString: strerror(errno)))")
    }
}
