// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typed events emitted into the daemon hook stream.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The kind of [`HookEvent`] being broadcast.
///
/// Subscribers (other agents, the CLI, the MCP surface) can filter on the
/// variant they care about. The `payload` carries variant-specific data as
/// a free-form `serde_json::Value` so the protocol stays flexible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEventType {
    /// A new session registered with the daemon.
    SessionStarted,
    /// A session ended (cleanly or via timeout).
    SessionEnded,
    /// An agent read a file.
    FileRead,
    /// An agent wrote (overwrote) a file.
    FileWritten,
    /// An agent edited (in-place) a file.
    FileEdited,
    /// An agent announced a new plan / TODO.
    PlanAnnounced,
    /// An agent changed which file it's focused on.
    FocusChanged,
    /// A periodic heartbeat from a session.
    Heartbeat,
    /// A reservation was successfully claimed.
    ReservationClaimed,
    /// A reservation was released (or expired).
    ReservationReleased,
    /// A new inbox message was posted.
    InboxMessagePosted,
}

/// A single event in the daemon's hook stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEvent {
    /// Session that produced this event.
    pub session_id: String,
    /// Variant of the event.
    pub event_type: HookEventType,
    /// Free-form, variant-specific payload. Producers SHOULD include a
    /// `"path"` field for file-related events and a `"message_id"` field
    /// for [`HookEventType::InboxMessagePosted`].
    pub payload: Value,
    /// When the event was produced.
    pub ts: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 12, 11, 0, 0).unwrap()
    }

    #[test]
    fn hook_event_type_serializes_as_snake_case() {
        let cases = [
            (HookEventType::SessionStarted, "session_started"),
            (HookEventType::SessionEnded, "session_ended"),
            (HookEventType::FileRead, "file_read"),
            (HookEventType::FileWritten, "file_written"),
            (HookEventType::FileEdited, "file_edited"),
            (HookEventType::PlanAnnounced, "plan_announced"),
            (HookEventType::FocusChanged, "focus_changed"),
            (HookEventType::Heartbeat, "heartbeat"),
            (HookEventType::ReservationClaimed, "reservation_claimed"),
            (HookEventType::ReservationReleased, "reservation_released"),
            (HookEventType::InboxMessagePosted, "inbox_message_posted"),
        ];
        for (variant, expected) in cases {
            let v = serde_json::to_value(variant).unwrap();
            assert_eq!(v, serde_json::json!(expected), "variant {variant:?}");
        }
    }

    #[test]
    fn hook_event_type_roundtrip() {
        for variant in [
            HookEventType::SessionStarted,
            HookEventType::SessionEnded,
            HookEventType::FileRead,
            HookEventType::FileWritten,
            HookEventType::FileEdited,
            HookEventType::PlanAnnounced,
            HookEventType::FocusChanged,
            HookEventType::Heartbeat,
            HookEventType::ReservationClaimed,
            HookEventType::ReservationReleased,
            HookEventType::InboxMessagePosted,
        ] {
            let s = serde_json::to_string(&variant).unwrap();
            let back: HookEventType = serde_json::from_str(&s).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn hook_event_roundtrip() {
        let original = HookEvent {
            session_id: "sess-1".into(),
            event_type: HookEventType::FileWritten,
            payload: json!({
                "path": "/repo/src/lib.rs",
                "bytes": 4096,
            }),
            ts: ts(),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: HookEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
        // Payload must be preserved verbatim.
        assert_eq!(back.payload["path"], "/repo/src/lib.rs");
        assert_eq!(back.payload["bytes"], 4096);
    }

    #[test]
    fn hook_event_supports_empty_payload() {
        let original = HookEvent {
            session_id: "sess-1".into(),
            event_type: HookEventType::Heartbeat,
            payload: Value::Null,
            ts: ts(),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: HookEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }
}
