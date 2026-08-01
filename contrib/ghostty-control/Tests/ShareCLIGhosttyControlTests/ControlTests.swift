import Foundation
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
