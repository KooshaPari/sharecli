import Foundation

public enum LiveIOError: Error, Equatable, Sendable {
    case invalidChunkBytes
    case invalidQueueCapacity
    case closed
}

public enum LiveIOEventKind: String, Codable, Sendable {
    case output
    case resize
    case exit
    case title
    case cwd
    case dropped
}

public struct LiveIOEvent: Codable, Equatable, Sendable {
    public let subscriptionID: UInt64
    public let surfaceID: String
    public let seq: UInt64
    public let kind: LiveIOEventKind
    /// RFC3339 timestamp supplied by the Ghostty app, when available.
    ///
    /// The Rust/root client contract uses an optional string here so native and
    /// non-native transports decode the same event envelope.
    public let timestamp: String?
    public let eventBytesBase64: String?
    public let dropped: Int?
    public let resyncRequired: Bool?

    public init(
        subscriptionID: UInt64,
        surfaceID: String,
        seq: UInt64,
        kind: LiveIOEventKind,
        timestamp: String? = nil,
        eventBytesBase64: String? = nil,
        dropped: Int? = nil,
        resyncRequired: Bool? = nil
    ) {
        self.subscriptionID = subscriptionID
        self.surfaceID = surfaceID
        self.seq = seq
        self.kind = kind
        self.timestamp = timestamp
        self.eventBytesBase64 = eventBytesBase64
        self.dropped = dropped
        self.resyncRequired = resyncRequired
    }

    enum CodingKeys: String, CodingKey {
        case subscriptionID = "subscription_id"
        case surfaceID = "surface_id"
        case seq, kind, timestamp
        case eventBytesBase64 = "event_bytes_base64"
        case dropped
        case resyncRequired = "resync_required"
    }
}

public struct LiveIOSubscription: AsyncSequence, Sendable {
    public typealias Element = LiveIOEvent
    public typealias AsyncIterator = AsyncStream<LiveIOEvent>.Iterator

    public let id: UInt64
    let stream: AsyncStream<LiveIOEvent>

    public func makeAsyncIterator() -> AsyncIterator {
        stream.makeAsyncIterator()
    }
}

/// Actor-isolated live event fanout with bounded per-subscriber buffering.
public actor LiveIOEventHub {
    public static let maxChunkBytes = 64 * 1024
    public static let maxQueueCapacity = 256

    private struct State {
        let surfaceID: String?
        let fromSequence: UInt64
        let maxChunkBytes: Int
        let queueCapacity: Int
        let continuation: AsyncStream<LiveIOEvent>.Continuation
        let stream: AsyncStream<LiveIOEvent>
        var dropped: Int = 0
    }

    private var nextSubscriptionID: UInt64 = 0
    private var nextSequence: UInt64 = 0
    private var subscriptions: [UInt64: State] = [:]

    public init() {}

    public func subscribe(
        surfaceID: String?,
        fromSequence: UInt64?,
        maxChunkBytes: Int = LiveIOEventHub.maxChunkBytes,
        queueCapacity: Int = 64
    ) throws -> LiveIOSubscription {
        guard (1...Self.maxChunkBytes).contains(maxChunkBytes) else {
            throw LiveIOError.invalidChunkBytes
        }
        guard (1...Self.maxQueueCapacity).contains(queueCapacity) else {
            throw LiveIOError.invalidQueueCapacity
        }
        nextSubscriptionID &+= 1
        let id = nextSubscriptionID
        let startingSequence = fromSequence ?? nextSequence &+ 1
        var continuation: AsyncStream<LiveIOEvent>.Continuation!
        let stream = AsyncStream<LiveIOEvent>(bufferingPolicy: .bufferingNewest(queueCapacity)) {
            continuation = $0
        }
        subscriptions[id] = State(
            surfaceID: surfaceID,
            fromSequence: startingSequence,
            maxChunkBytes: maxChunkBytes,
            queueCapacity: queueCapacity,
            continuation: continuation,
            stream: stream
        )
        return LiveIOSubscription(id: id, stream: stream)
    }

    /// The next sequence number that a newly-created subscription would see.
    public func nextSequenceNumber() -> UInt64 {
        nextSequence &+ 1
    }

    public func subscription(id: UInt64) -> LiveIOSubscription? {
        guard let state = subscriptions[id] else { return nil }
        return LiveIOSubscription(id: id, stream: state.stream)
    }

    @discardableResult
    public func publish(
        surfaceID: String,
        kind: LiveIOEventKind,
        bytes: [UInt8],
        timestamp: String? = nil
    ) throws -> UInt64 {
        let limits = subscriptions.values
            .filter { $0.surfaceID == nil || $0.surfaceID == surfaceID }
            .map(\.maxChunkBytes)
        let chunkSize = limits.min() ?? Self.maxChunkBytes
        let chunks: [[UInt8]] = bytes.isEmpty ? [[]] : stride(from: 0, to: bytes.count, by: chunkSize).map {
            Array(bytes[$0 ..< min($0 + chunkSize, bytes.count)])
        }

        var sequence = nextSequence
        for chunk in chunks {
            sequence &+= 1
            nextSequence = sequence
            let encoded = chunk.isEmpty ? nil : Data(chunk).base64EncodedString()
            let matchingIDs = subscriptions.compactMap { id, state in
                (state.surfaceID == nil || state.surfaceID == surfaceID) && sequence >= state.fromSequence
                    ? id
                    : nil
            }
            for id in matchingIDs {
                guard var state = subscriptions[id] else { continue }
                let event = LiveIOEvent(
                    subscriptionID: id,
                    surfaceID: surfaceID,
                    seq: sequence,
                    kind: kind,
                    timestamp: timestamp,
                    eventBytesBase64: encoded,
                    dropped: state.dropped > 0 ? state.dropped : nil,
                    resyncRequired: state.dropped > 0 ? true : nil
                )
                let result = state.continuation.yield(event)
                if case .dropped = result {
                    state.dropped += 1
                } else {
                    state.dropped = 0
                }
                subscriptions[id] = state
            }
        }
        return sequence
    }

    @discardableResult
    public func unsubscribe(subscriptionID: UInt64) -> Bool {
        guard let state = subscriptions.removeValue(forKey: subscriptionID) else { return false }
        state.continuation.finish()
        return true
    }

    @discardableResult
    public func unsubscribe(_ subscription: LiveIOSubscription) -> Bool {
        unsubscribe(subscriptionID: subscription.id)
    }
}
