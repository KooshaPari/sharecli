import Foundation
import Testing
@testable import ShareCLIGhosttyControl

@Test func liveEventsCarryWireEnvelopeAndMonotonicSequence() async throws {
    let hub = LiveIOEventHub()
    let subscription = try await hub.subscribe(
        surfaceID: "ghostty:1",
        fromSequence: nil,
        maxChunkBytes: 1024,
        queueCapacity: 8
    )

    let sequence = try await hub.publish(
        surfaceID: "ghostty:1",
        kind: .output,
        bytes: Array("hello".utf8),
        timestamp: "1970-01-01T00:00:42Z"
    )
    #expect(sequence == 1)

    var iterator = subscription.makeAsyncIterator()
    let event = try #require(await iterator.next())
    #expect(event.subscriptionID == subscription.id)
    #expect(event.surfaceID == "ghostty:1")
    #expect(event.seq == sequence)
    #expect(event.kind == .output)
    #expect(event.timestamp == "1970-01-01T00:00:42Z")
    #expect(event.eventBytesBase64 == Data("hello".utf8).base64EncodedString())
    #expect(event.dropped == nil)
    #expect(event.resyncRequired == nil)

    let encoded = try JSONSerialization.jsonObject(with: JSONEncoder().encode(event)) as? [String: Any]
    #expect(encoded?["subscription_id"] as? UInt64 == subscription.id)
    #expect(encoded?["event_bytes_base64"] as? String == Data("hello".utf8).base64EncodedString())
}

@Test func liveEventQueueIsBoundedAndReportsDroppedEvents() async throws {
    let hub = LiveIOEventHub()
    let subscription = try await hub.subscribe(
        surfaceID: nil,
        fromSequence: nil,
        maxChunkBytes: 1024,
        queueCapacity: 2
    )

    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [1], timestamp: "1970-01-01T00:00:01Z")
    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [2], timestamp: "1970-01-01T00:00:02Z")
    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [3], timestamp: "1970-01-01T00:00:03Z")

    var iterator = subscription.makeAsyncIterator()
    let first = try #require(await iterator.next())
    let second = try #require(await iterator.next())
    #expect(first.seq == 2)
    #expect(second.seq == 3)

    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [4], timestamp: "1970-01-01T00:00:04Z")
    let fourth = try #require(await iterator.next())
    #expect(fourth.seq == 4)
    #expect(fourth.dropped == 1)
    #expect(fourth.resyncRequired == true)
}

@Test func liveEventSubscriptionFiltersSurfaceAndStartingSequence() async throws {
    let hub = LiveIOEventHub()
    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [1], timestamp: "1970-01-01T00:00:01Z")
    _ = try await hub.publish(surfaceID: "ghostty:2", kind: .output, bytes: [2], timestamp: "1970-01-01T00:00:02Z")

    let subscription = try await hub.subscribe(
        surfaceID: "ghostty:1",
        fromSequence: 2,
        maxChunkBytes: 1024,
        queueCapacity: 8
    )
    _ = try await hub.publish(surfaceID: "ghostty:1", kind: .output, bytes: [3], timestamp: "1970-01-01T00:00:03Z")
    _ = try await hub.publish(surfaceID: "ghostty:2", kind: .output, bytes: [4], timestamp: "1970-01-01T00:00:04Z")

    var iterator = subscription.makeAsyncIterator()
    let event = try #require(await iterator.next())
    #expect(event.surfaceID == "ghostty:1")
    #expect(event.seq == 3)
    #expect(event.eventBytesBase64 == Data([3]).base64EncodedString())
}

@Test func liveEventUnsubscribeFinishesAsyncSequence() async throws {
    let hub = LiveIOEventHub()
    let subscription = try await hub.subscribe(
        surfaceID: nil,
        fromSequence: nil,
        maxChunkBytes: 1024,
        queueCapacity: 8
    )
    var iterator = subscription.makeAsyncIterator()
    await hub.unsubscribe(subscription)
    #expect(await iterator.next() == nil)
}

@Test func liveEventLimitsAreValidated() async throws {
    let hub = LiveIOEventHub()
    do {
        _ = try await hub.subscribe(surfaceID: nil, fromSequence: nil, maxChunkBytes: 0, queueCapacity: 1)
        Issue.record("zero max_chunk_bytes must be rejected")
    } catch LiveIOError.invalidChunkBytes { }

    do {
        _ = try await hub.subscribe(surfaceID: nil, fromSequence: nil, maxChunkBytes: 65_537, queueCapacity: 1)
        Issue.record("max_chunk_bytes above the wire limit must be rejected")
    } catch LiveIOError.invalidChunkBytes { }

    do {
        _ = try await hub.subscribe(surfaceID: nil, fromSequence: nil, maxChunkBytes: 1, queueCapacity: 257)
        Issue.record("queue capacity above the wire limit must be rejected")
    } catch LiveIOError.invalidQueueCapacity { }
}
