import Foundation
import Darwin
import Testing
@testable import ShareCLIGhosttyControl

private struct FakeProvider: SurfaceProvider {
    func listSurfaces() throws -> [SurfaceRecord] {
        [SurfaceRecord(id: "ghostty:1", title: "agent", cwd: "/tmp", process: nil)]
    }

    func send(surfaceID: String, bytes: [UInt8]) throws {}
    func read(surfaceID: String, maxBytes: Int) throws -> [UInt8] { Array("ok".utf8.prefix(maxBytes)) }
    func resize(surfaceID: String, rows: UInt16, cols: UInt16) throws {}
    func capabilities(surfaceID: String) throws -> SurfaceCapabilities {
        SurfaceCapabilities(read: true, write: true, resize: true, layout: false, durablePty: false)
    }
}

@Test func listUsesRustCompatibleSnakeCase() throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":1,"method":"surface.list","params":{}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: dispatcher.dispatch(line)) as? [String: Any])
    let result = try #require(response["result"] as? [[String: Any]])
    #expect(result[0]["id"] as? String == "ghostty:1")
}

@Test func tokenIsRequiredBeforeProviderAccess() throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider(), expectedToken: "secret")
    let line = Data(#"{"jsonrpc":"2.0","id":2,"method":"surface.list","params":{}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32001)
}

@Test func sendRequiresExactlyOnePayload() throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    let line = Data(#"{"jsonrpc":"2.0","id":3,"method":"surface.io.send","params":{"surface_id":"ghostty:1"}}"#.utf8)
    let response = try #require(JSONSerialization.jsonObject(with: dispatcher.dispatch(line)) as? [String: Any])
    let error = try #require(response["error"] as? [String: Any])
    #expect(error["code"] as? Int == -32602)
}

@Test func integerInputsRejectBooleansAndFloats() throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    for literal in ["true", "1.5"] {
        let line = Data("{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"surface.io.read\",\"params\":{\"surface_id\":\"ghostty:1\",\"max_bytes\":\(literal)}}".utf8)
        let response = try #require(JSONSerialization.jsonObject(with: dispatcher.dispatch(line)) as? [String: Any])
        let error = try #require(response["error"] as? [String: Any])
        #expect(error["code"] as? Int == -32602)
    }
}

@Test func byteInputsRejectBooleansAndFloats() throws {
    let dispatcher = ControlDispatcher(provider: FakeProvider())
    for literal in ["true", "1.5"] {
        let line = Data("{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"surface.io.send\",\"params\":{\"surface_id\":\"ghostty:1\",\"bytes\":[\(literal)]}}".utf8)
        let response = try #require(JSONSerialization.jsonObject(with: dispatcher.dispatch(line)) as? [String: Any])
        let error = try #require(response["error"] as? [String: Any])
        #expect(error["code"] as? Int == -32602)
    }
}

@Test func unixServerRoundTripsOneRequestLine() throws {
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
