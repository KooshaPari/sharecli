import Foundation
import Darwin

/// Owner-only newline-delimited Unix-domain listener for a native Ghostty app.
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

    deinit { stop() }

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
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    Darwin.bind(fd, $0, addressLength)
                }
            }
            guard bound == 0 else { throw socketError("bind control socket") }
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
            guard peerBelongsToCurrentUser(fd) else {
                Darwin.close(fd)
                continue
            }
            Task.detached(priority: .userInitiated) { [weak self] in
                await self?.serveConnection(fd)
            }
        }
    }

    private func peerBelongsToCurrentUser(_ fd: Int32) -> Bool {
        var peerUID: uid_t = 0
        var peerGID: gid_t = 0
        guard getpeereid(fd, &peerUID, &peerGID) == 0 else { return false }
        return peerUID == geteuid()
    }

    private func serveConnection(_ fd: Int32) async {
        let writer = SocketWriter(fd: fd)
        var eventTasks: [Task<Void, Never>] = []
        defer {
            eventTasks.forEach { $0.cancel() }
            writer.close()
        }
        var buffer = Data()
        var chunk = [UInt8](repeating: 0, count: 16 * 1024)
        while true {
            let count = chunk.withUnsafeMutableBytes { Darwin.recv(fd, $0.baseAddress, $0.count, 0) }
            if count <= 0 { return }
            buffer.append(contentsOf: chunk[0..<count])
            guard buffer.count <= ControlDispatcher.maxRequestBytes else { return }
            while let newline = buffer.firstIndex(of: 0x0a) {
                let line = buffer.prefix(upTo: newline)
                buffer.removeSubrange(...newline)
                var response = await dispatcher.dispatch(Data(line))
                if !response.isEmpty {
                    response.append(0x0a)
                    guard writer.send(response) else { return }
                }
                if let subscriptionID = subscriptionID(from: response),
                   let liveEvents = dispatcher.liveEvents,
                   let subscription = await liveEvents.subscription(id: subscriptionID) {
                    let task = Task.detached(priority: .userInitiated) { [writer] in
                        for await event in subscription {
                            guard let data = Self.encodeEvent(event) else { return }
                            var line = data
                            line.append(0x0a)
                            guard writer.send(line) else { return }
                        }
                    }
                    eventTasks.append(task)
                }
            }
        }
    }

    private static func encodeEvent(_ event: LiveIOEvent) -> Data? {
        guard let params = try? JSONSerialization.jsonObject(with: JSONEncoder().encode(event)) else {
            return nil
        }
        return try? JSONSerialization.data(withJSONObject: [
            "jsonrpc": "2.0",
            "method": "surface.io.event",
            "params": params,
        ])
    }

    private func subscriptionID(from response: Data) -> UInt64? {
        guard let object = try? JSONSerialization.jsonObject(with: response) as? [String: Any],
              let result = object["result"] as? [String: Any] else { return nil }
        guard let value = result["subscription_id"] as? NSNumber else { return nil }
        let type = String(cString: value.objCType)
        guard ["i", "s", "l", "q", "I", "S", "L", "Q"].contains(type),
              value.int64Value >= 0 else { return nil }
        return value.uint64Value
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

private final class SocketWriter: @unchecked Sendable {
    private let fd: Int32
    private let lock = NSLock()
    private var closed = false

    init(fd: Int32) {
        self.fd = fd
    }

    func send(_ data: Data) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard !closed else { return false }
        return data.withUnsafeBytes { rawBuffer in
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

    func close() {
        lock.lock()
        guard !closed else {
            lock.unlock()
            return
        }
        closed = true
        Darwin.close(fd)
        lock.unlock()
    }
}
