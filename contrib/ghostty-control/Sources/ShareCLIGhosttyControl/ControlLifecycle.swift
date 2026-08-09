import Foundation

/// State of the app-owned control listener.
public enum ControlLifecycleState: Equatable, Sendable {
    case stopped
    case running(socketPath: String)
    case failed(message: String)
}

/// Main-actor owner for the Ghostty app integration lifecycle.
///
/// Instantiate this beside the native app delegate. Call `start(provider:)`
/// only after Ghostty has created its app/surface registry, and call `stop()`
/// before that registry is torn down. The listener is intentionally kept
/// alive with `UnavailableSurfaceProvider` when the app wants a degraded
/// control endpoint during startup/teardown.
@MainActor
public final class ControlLifecycle {
    public let socketPath: String
    public let expectedToken: String?
    public let liveEvents: LiveIOEventHub?
    public private(set) var state: ControlLifecycleState = .stopped

    private var server: UnixControlServer?

    public init(
        socketPath: String,
        expectedToken: String? = nil,
        liveEvents: LiveIOEventHub? = nil
    ) {
        self.socketPath = socketPath
        self.expectedToken = expectedToken
        self.liveEvents = liveEvents
    }

    /// Start the listener once Ghostty's native surface tree is ready.
    public func start(provider: any SurfaceProvider) {
        guard server == nil else { return }
        let dispatcher = ControlDispatcher(
            provider: provider,
            expectedToken: expectedToken,
            liveEvents: liveEvents
        )
        let candidate = UnixControlServer(path: socketPath, dispatcher: dispatcher)
        do {
            try candidate.start()
            server = candidate
            state = .running(socketPath: socketPath)
        } catch {
            state = .failed(message: String(describing: error))
        }
    }

    /// Start a degraded endpoint before Ghostty is ready or after its provider
    /// has been invalidated. This is useful for health/status tooling without
    /// pretending that surface I/O is available.
    public func startUnavailable(reason: String = "native Ghostty surface provider unavailable") {
        start(provider: UnavailableSurfaceProvider(reason: reason))
    }

    /// Stop the listener before Ghostty surface/PTY teardown.
    public func stop() {
        server?.stop()
        server = nil
        state = .stopped
    }

    deinit {
        server?.stop()
    }
}
