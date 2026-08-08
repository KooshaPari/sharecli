import Foundation
import Darwin
import Testing
@testable import ShareCLIGhosttyControl

private struct FakeProvider: SurfaceProvider {
    func listSurfaces() async throws -> [SurfaceRecord] {
        [SurfaceRecord(id: "ghostty:1", title: "agent", cwd: "/tmp", process: nil)]
    }

    func send(surfaceID: String, bytes: [UInt8]) async throws {}
    func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8] { Array("ok".utf8.prefix(maxBytes)) }
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws {}
    func capabilities(surfaceID: String) async throws -> SurfaceCapabilities {
        SurfaceCapabilities(read: true, write: true, resize: true, layout: false, durablePty: false)
    }
}

@MainActor
private final class MainActorProvider: SurfaceProvider {
    func listSurfaces() async throws -> [SurfaceRecord] {
        [SurfaceRecord(id: "ghostty:main", title: "main", cwd: "/tmp", process: nil)]
    }

    func send(surfaceID: String, bytes: [UInt8]) async throws {}
    func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8] { [] }
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws {}
    func capabilities(surfaceID: String) async throws -> SurfaceCapabilities {
        SurfaceCapabilities(read: true, write: true, resize: true, layout: true, durablePty: true)
    }
}

private struct OversizedReadProvider: SurfaceProvider {
    func listSurfaces() async throws -> [SurfaceRecord] { [] }
    func send(surfaceID: String, bytes: [UInt8]) async throws {}
    func read(surfaceID: String, maxBytes: Int) async throws -> [UInt8] {
        [UInt8](repeating: 0, count: maxBytes + 1)
    }
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) async throws {}
    func capabilities(surfaceID: String) async throws -> SurfaceCapabilities {
        SurfaceCapabilities(read: true, write: true, resize: true, layout: false, durablePty: false)
    }
}

@Test func listUsesRustCompatibleSnakeCase() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":1,"method":"surface.list","params":{}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let result = try #require(response["result"] as? [[String: Any]])
    #expect(result[0]["id"] as? String == "ghostty:1")
}

@Test func tokenIsRequiredBeforeProviderAccess() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider(), expectedToken: "secret")
    let line = Data(#"{"jsonrpc":"2.0","id":2,"method":"surface.list","params":{}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32001)
}

@Test func mainActorProviderCanServeThroughDispatcher() async throws {
    let dispatcher = ControlDispatcher(provider: MainActorProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":6,"method":"surface.list","params":{}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let result = try #require(response["result"] as? [[String: Any]])
    #expect(result[0]["id"] as? String == "ghostty:main")
}

@Test func nonObjectParamsAreRejected() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":7,"method":"surface.list","params":[]}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32602)
}

@Test func sendRejectsOversizedPayload() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let text = String(repeating: "x", count: ControlDispatcher.maxSendBytes + 1)
    let object: [String: Any] = [
        "jsonrpc": "2.0",
        "id": 8,
        "method": "surface.io.send",
        "params": ["surface_id": "ghostty:1", "text": text],
    ]
    let line = try JSONSerialization.data(withJSONObject: object)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32602)
}

@Test func oversizedProviderReadIsRejected() async throws {
    let dispatcher = ControlDispatcher(provider: OversizedReadProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":10,"method":"surface.io.read","params":{"surface_id":"ghostty:1","max_bytes":8}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32000)
}

@Test func sendRequiresExactlyOnePayload() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":3,"method":"surface.io.send","params":{"surface_id":"ghostty:1"}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32602)
}

@Test func integerInputsRejectBooleansAndFloats() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    for literal in ["true", "1.5"] {
        let line = Data("{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"surface.io.read\",\"params\":{\"surface_id\":\"ghostty:1\",\"max_bytes\":\(literal)}}".utf8)
        let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
        let error = try #require(response["error"] as? [String: Any])
        #expect(error["code"] as? Int == -32602)
    }
}

@Test func byteInputsRejectBooleansAndFloats() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    for literal in ["true", "1.5"] {
        let line = Data("{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"surface.io.send\",\"params\":{\"surface_id\":\"ghostty:1\",\"bytes\":[\(literal)]}}".utf8)
        let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
        let error = try #require(response["error"] as? [String: Any])
        #expect(error["code"] as? Int == -32602)
    }
}

@Test func liveSubscriptionDispatchReturnsBoundedAck() async throws {
    let hub = LiveIOEventHub()
    let dispatcher = ControlDispatcher(provider: FakeProvider(), liveEvents: hub)
    let line = Data(#"{"jsonrpc":"2.0","id":11,"method":"surface.io.subscribe","params":{"surface_id":"ghostty:1","max_chunk_bytes":1024,"queue_capacity":4}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: await dispatcher.dispatch(line)) as? [String: Any])
    let result = try #require(response["result"] as? [String: Any])
    #expect(result["subscription_id"] as? UInt64 == 1)
    #expect(result["next_seq"] as? UInt64 == 1)
    #expect((result["capabilities"] as? [String: Any])?["queue_capacity"] as? Int == 4)
}

@Test func notificationsProduceNoResponseBytes() async throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","method":"surface.io.send","params":{"surface_id":"ghostty:1","text":"hello"}}"#.utf8)
    #expect(await dispatcher.dispatch(line).isEmpty)
}

@Test func unixServerRoundTripsOneRequestLine() async throws {
    let path = "/tmp/sharecli-control-\(UUID().uuidString).sock"
    let server = UnixControlServer(path: path, dispatcher: ControlDispatcher(provider: FakeProvider()))
    try server.start()
    defer { server.stop() }

    let fd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
    #expect(fd >= 0)
    defer { Darwin.close(fd) }
    var address = sockaddr_un()
    address.sun_family = sa_family_t(AF_UNIX)
    path.withCString { pointer in
        withUnsafeMutableBytes(of: &address.sun_path) { destination in
            destination.copyBytes(from: UnsafeRawBufferPointer(start: pointer, count: path.utf8.count + 1))
        }
    }
    let addressLength = socklen_t(MemoryLayout<sockaddr_un>.size)
    let connected = withUnsafePointer(to: &address) { pointer in
        pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) {
            Darwin.connect(fd, $0, addressLength)
        }
    }
    #expect(connected == 0)

    let request = Data("{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"surface.list\",\"params\":{}}\n".utf8)
    request.withUnsafeBytes { buffer in
        _ = Darwin.send(fd, buffer.baseAddress, buffer.count, 0)
    }
    var responseBuffer = [UInt8](repeating: 0, count: 4096)
    let count = responseBuffer.withUnsafeMutableBytes { buffer in
        Darwin.recv(fd, buffer.baseAddress, buffer.count, 0)
    }
    #expect(count > 0)
    let responseData = Data(responseBuffer.prefix(max(0, count)))
    let response = try #require(JSONSerialization.jsonObject(with: responseData) as? [String: Any])
    let result = try #require(response["result"] as? [[String: Any]])
    #expect(result[0]["id"] as? String == "ghostty:1")
}
