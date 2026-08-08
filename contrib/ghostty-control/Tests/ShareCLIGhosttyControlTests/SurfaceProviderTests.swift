import Foundation
import Testing
@testable import ShareCLIGhosttyControl

private func binding(id: String, title: String? = nil) -> SurfaceBinding {
    SurfaceBinding(
        record: SurfaceRecord(
            id: id,
            title: title,
            cwd: "/tmp",
            process: ProcessEvidence(pid: nil, tty: nil, cwd: "/tmp", argv: [], startedAt: nil)
        ),
        send: { _ in },
        read: { maxBytes in Array("ok".utf8.prefix(maxBytes)) },
        resize: { _, _ in },
        capabilities: {
            SurfaceCapabilities(read: true, write: true, resize: true, layout: false, durablePty: false)
        }
    )
}

@Test func registryIsDeterministicAndRoutesOperations() async throws {
    let registry = SurfaceProviderRegistry()
    try await registry.register(binding(id: "ghostty:2"))
    try await registry.register(binding(id: "ghostty:1"))

    let surfaces = try await registry.listSurfaces()
    #expect(surfaces.map(\.id) == ["ghostty:1", "ghostty:2"])
    #expect(try await registry.read(surfaceID: "ghostty:1", maxBytes: 2) == Array("ok".utf8))
    #expect(try await registry.capabilities(surfaceID: "ghostty:1").write)
    #expect(await registry.unregister(surfaceID: "ghostty:2"))
    #expect((try? await registry.listSurfaces().count) == 1)
}

@Test func registryRejectsDuplicateAndUnknownSurface() async throws {
    let registry = SurfaceProviderRegistry()
    try await registry.register(binding(id: "ghostty:1"))

    do {
        try await registry.register(binding(id: "ghostty:1"))
        Issue.record("duplicate surface ids must be rejected")
    } catch let error as ControlError {
        #expect(error == .provider("surface binding already registered: ghostty:1"))
    }

    do {
        _ = try await registry.read(surfaceID: "ghostty:missing", maxBytes: 1)
        Issue.record("unknown surfaces must fail explicitly")
    } catch let error as ControlError {
        #expect(error == .provider("surface unavailable: ghostty:missing"))
    }
}

@Test func unavailableProviderReportsDegradedCapabilities() async throws {
    let provider = UnavailableSurfaceProvider(reason: "before Ghostty ready")
    #expect(try await provider.listSurfaces().isEmpty)
    let capabilities = try await provider.capabilities(surfaceID: "ghostty:missing")
    #expect(!capabilities.read)
    #expect(!capabilities.write)
    #expect(!capabilities.resize)
    #expect(!capabilities.durablePty)

    do {
        _ = try await provider.read(surfaceID: "ghostty:missing", maxBytes: 1)
        Issue.record("unavailable provider must not emulate reads")
    } catch let error as ControlError {
        #expect(error == .provider("before Ghostty ready"))
    }
}

@MainActor
@Test func lifecycleStartsAndStopsUnavailableEndpoint() async throws {
    let path = "/tmp/sharecli-lifecycle-\(UUID().uuidString).sock"
    let lifecycle = ControlLifecycle(socketPath: path)
    lifecycle.startUnavailable(reason: "startup")
    #expect(lifecycle.state == .running(socketPath: path))
    #expect(FileManager.default.fileExists(atPath: path))
    lifecycle.stop()
    #expect(lifecycle.state == .stopped)
    #expect(!FileManager.default.fileExists(atPath: path))
}

@MainActor
@Test func lifecycleSurfacesBindFailureWithoutThrowing() async throws {
    let path = "/tmp/sharecli-lifecycle-\(UUID().uuidString).sock"
    let lifecycle = ControlLifecycle(socketPath: path)
    FileManager.default.createFile(atPath: path, contents: Data("not a socket".utf8))
    defer { try? FileManager.default.removeItem(atPath: path) }

    lifecycle.startUnavailable()
    if case .failed = lifecycle.state {
        #expect(true)
    } else {
        Issue.record("listener failure should be observable in lifecycle state")
    }
}
