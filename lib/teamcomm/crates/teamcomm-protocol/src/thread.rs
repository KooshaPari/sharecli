// SPDX-License-Identifier: MIT OR Apache-2.0
//! First-class conversation threads (M1).
//!
//! A [`Thread`] is a named, multi-participant conversation surface. It
//! has its own id, an optional short topic, a list of participating
//! sessions, and an `archived` flag. Messages in the inbox can carry an
//! optional `thread_id` (see [`crate::InboxMessage::thread_id`]) so that
//! consumers can fetch every message in a thread in one call.
//!
//! The wire shape is stable: snake_case enums, ISO-8601 timestamps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Lifecycle status of a thread.
///
/// A thread is [`ThreadStatus::Active`] from creation until it is
/// explicitly archived. Archived threads are still readable (`get_details`
/// and the underlying message list remain accessible) but are filtered
/// out of the default `thread.list` result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    /// Open thread, appears in default `thread.list` results.
    Active,
    /// Archived thread, does not appear in default `thread.list`
    /// results unless explicitly requested.
    Archived,
}

impl Default for ThreadStatus {
    fn default() -> Self {
        ThreadStatus::Active
    }
}

impl ThreadStatus {
    /// Canonical wire string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreadStatus::Active => "active",
            ThreadStatus::Archived => "archived",
        }
    }

    /// Inverse of [`Self::as_str`]. Case-insensitive on the way in.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "active" => Some(ThreadStatus::Active),
            "archived" => Some(ThreadStatus::Archived),
            _ => None,
        }
    }
}

impl std::fmt::Display for ThreadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ThreadStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("unknown thread status: {s}"))
    }
}

/// A conversation thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thread {
    /// Stable, unique thread id (UUID v4, prefixed `thr_`).
    pub thread_id: String,
    /// Short, human-readable title. Required.
    pub title: String,
    /// Optional longer description / topic. Free-form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Session id of the agent that created the thread.
    pub created_by: String,
    /// When the thread was created.
    pub created_at: DateTime<Utc>,
    /// Lifecycle status. Defaults to [`ThreadStatus::Active`].
    #[serde(default)]
    pub status: ThreadStatus,
}

/// Payload for `thread.create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateThreadRequest {
    /// Session id of the creator. The daemon adds this session to the
    /// thread's participant list as part of creation.
    pub created_by: String,
    /// Short title for the thread.
    pub title: String,
    /// Optional longer topic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

/// Detailed view of a thread, including its participant list and the
/// number of messages tagged with its `thread_id`.
///
/// Returned by `thread.get_details`. The full message list is *not*
/// included in this struct to keep the response small; callers fetch
/// the actual messages via the inbox (filtered by `thread_id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadDetails {
    /// The thread itself.
    pub thread: Thread,
    /// All sessions currently participating in the thread, in the order
    /// they joined. The creator is always first.
    pub participants: Vec<String>,
    /// Number of inbox messages tagged with this `thread_id`. Computed
    /// at the time of the `get_details` call.
    pub message_count: u32,
}

/// Query filter for `thread.list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadListQuery {
    /// If `true`, include archived threads. Default: `false`.
    #[serde(default)]
    pub include_archived: bool,
    /// Optional substring filter on `title` (case-insensitive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_contains: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 14, 9, 0, 0).unwrap()
    }

    #[test]
    fn thread_status_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(ThreadStatus::Active).unwrap(),
            serde_json::json!("active")
        );
        assert_eq!(
            serde_json::to_value(ThreadStatus::Archived).unwrap(),
            serde_json::json!("archived")
        );
    }

    #[test]
    fn thread_status_roundtrip() {
        for s in [ThreadStatus::Active, ThreadStatus::Archived] {
            let j = serde_json::to_string(&s).unwrap();
            let back: ThreadStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn thread_status_default_is_active() {
        assert_eq!(ThreadStatus::default(), ThreadStatus::Active);
    }

    #[test]
    fn thread_status_parse_is_case_insensitive() {
        assert_eq!(ThreadStatus::parse("ACTIVE"), Some(ThreadStatus::Active));
        assert_eq!(ThreadStatus::parse("Archived"), Some(ThreadStatus::Archived));
        assert_eq!(ThreadStatus::parse("nope"), None);
    }

    #[test]
    fn thread_roundtrip_minimal() {
        let original = Thread {
            thread_id: "thr_abc".into(),
            title: "M1 persistence design".into(),
            topic: None,
            created_by: "sess-1".into(),
            created_at: ts(),
            status: ThreadStatus::Active,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: Thread = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn thread_roundtrip_with_topic() {
        let original = Thread {
            thread_id: "thr_xyz".into(),
            title: "auth refactor".into(),
            topic: Some("How to migrate from session-cookies to JWTs".into()),
            created_by: "sess-forge".into(),
            created_at: ts(),
            status: ThreadStatus::Active,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: Thread = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn thread_omits_none_topic() {
        let t = Thread {
            thread_id: "thr_q".into(),
            title: "x".into(),
            topic: None,
            created_by: "s".into(),
            created_at: ts(),
            status: ThreadStatus::Active,
        };
        let s = serde_json::to_string(&t).unwrap();
        assert!(!s.contains("\"topic\""), "got: {s}");
    }

    #[test]
    fn thread_legacy_payload_defaults_status() {
        // An M0-style payload (no `status` field) should still
        // deserialize, defaulting to Active.
        let legacy = serde_json::json!({
            "thread_id": "thr_legacy",
            "title": "x",
            "created_by": "s",
            "created_at": "2026-06-14T09:00:00Z"
        });
        let t: Thread = serde_json::from_value(legacy).unwrap();
        assert_eq!(t.status, ThreadStatus::Active);
        assert!(t.topic.is_none());
    }

    #[test]
    fn create_thread_request_roundtrip() {
        let original = CreateThreadRequest {
            created_by: "sess-1".into(),
            title: "M1 persistence design".into(),
            topic: Some("choosing SQLite vs sled".into()),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: CreateThreadRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn thread_details_roundtrip() {
        let thread = Thread {
            thread_id: "thr_d".into(),
            title: "x".into(),
            topic: Some("t".into()),
            created_by: "s1".into(),
            created_at: ts(),
            status: ThreadStatus::Active,
        };
        let original = ThreadDetails {
            thread,
            participants: vec!["s1".into(), "s2".into(), "s3".into()],
            message_count: 7,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: ThreadDetails = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn thread_list_query_default_roundtrip() {
        let original = ThreadListQuery::default();
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("\"include_archived\":false"), "got: {s}");
        assert!(!s.contains("title_contains"), "got: {s}");
        let back: ThreadListQuery = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn thread_list_query_populated_roundtrip() {
        let original = ThreadListQuery {
            include_archived: true,
            title_contains: Some("m1".into()),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: ThreadListQuery = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }
}
