// SPDX-License-Identifier: MIT OR Apache-2.0
//! Inter-agent inbox: messages and query filters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Delivery priority for an inbox message.
///
/// Ordered low < normal < high. The daemon may use this for ordering and
/// for backpressure decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Low priority — informational, may be batched.
    Low,
    /// Normal priority — default for human-readable chat.
    Normal,
    /// High priority — surfaced prominently; client should consider
    /// interrupt signals.
    High,
}

/// A single inbox message from one session to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxMessage {
    /// Stable, unique message id (UUID v4).
    pub message_id: String,
    /// Sender session id.
    pub from_session: String,
    /// Recipient session id. A special value like `"*"` may be used for
    /// broadcast; interpretation is up to the daemon.
    pub to_session: String,
    /// Short single-line subject.
    pub subject: String,
    /// Full message body. Plain text by convention; the daemon does not
    /// interpret markdown.
    pub body: String,
    /// Delivery priority.
    pub priority: Priority,
    /// When the message was posted.
    pub ts: DateTime<Utc>,
    /// `true` once the recipient has acknowledged/read the message.
    pub read: bool,
}

/// Query filter for fetching inbox messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxQuery {
    /// If `true`, only return unread messages.
    pub unread_only: bool,
    /// Maximum number of messages to return. `0` is treated as "no limit"
    /// by the daemon.
    pub limit: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 12, 12, 0, 0).unwrap()
    }

    #[test]
    fn priority_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(Priority::Low).unwrap(),
            serde_json::json!("low")
        );
        assert_eq!(
            serde_json::to_value(Priority::Normal).unwrap(),
            serde_json::json!("normal")
        );
        assert_eq!(
            serde_json::to_value(Priority::High).unwrap(),
            serde_json::json!("high")
        );
    }

    #[test]
    fn priority_roundtrip() {
        for p in [Priority::Low, Priority::Normal, Priority::High] {
            let s = serde_json::to_string(&p).unwrap();
            let back: Priority = serde_json::from_str(&s).unwrap();
            assert_eq!(back, p);
        }
    }

    #[test]
    fn inbox_message_roundtrip() {
        let original = InboxMessage {
            message_id: "msg-1".into(),
            from_session: "sess-a".into(),
            to_session: "sess-b".into(),
            subject: "heads up".into(),
            body: "I'm about to refactor src/lib.rs".into(),
            priority: Priority::High,
            ts: ts(),
            read: false,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: InboxMessage = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn inbox_message_read_flag_roundtrip() {
        let mut msg = InboxMessage {
            message_id: "msg-2".into(),
            from_session: "s".into(),
            to_session: "t".into(),
            subject: "x".into(),
            body: "y".into(),
            priority: Priority::Normal,
            ts: ts(),
            read: false,
        };
        let s = serde_json::to_string(&msg).unwrap();
        let back: InboxMessage = serde_json::from_str(&s).unwrap();
        assert_eq!(back, msg);
        msg.read = true;
        let s = serde_json::to_string(&msg).unwrap();
        let back: InboxMessage = serde_json::from_str(&s).unwrap();
        assert!(back.read);
    }

    #[test]
    fn inbox_query_roundtrip() {
        let cases = [
            InboxQuery {
                unread_only: true,
                limit: 0,
            },
            InboxQuery {
                unread_only: false,
                limit: 50,
            },
        ];
        for original in cases {
            let s = serde_json::to_string(&original).unwrap();
            let back: InboxQuery = serde_json::from_str(&s).unwrap();
            assert_eq!(back, original);
        }
    }
}
