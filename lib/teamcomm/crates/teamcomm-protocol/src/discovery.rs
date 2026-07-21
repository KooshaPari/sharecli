// SPDX-License-Identifier: MIT OR Apache-2.0
//! Discovery query/result types for "who else is working on what".

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::session::SessionSummary;

/// Filter for `discover_sessions` requests.
///
/// All fields are optional and combine with AND semantics. An empty
/// [`DiscoveryQuery`] (all defaults) returns every known session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    /// Match sessions whose `focus_file` equals or is under this path.
    pub path: Option<PathBuf>,
    /// Match sessions whose `focus_branch` equals this branch name.
    pub branch: Option<String>,
    /// Match sessions whose `working_dir` is in this repo root.
    pub repo: Option<PathBuf>,
    /// Match sessions that declare *all* of these capability tags.
    /// Empty means "no capability filter".
    pub capabilities: Vec<String>,
}

/// Result of a `discover_sessions` request: a list of matching
/// [`SessionSummary`] entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryResult {
    /// Matching sessions, in daemon-defined order.
    pub sessions: Vec<SessionSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{AgentType, SessionSummary};
    use crate::state::AgentStatus;
    use chrono::{DateTime, TimeZone, Utc};

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 12, 13, 0, 0).unwrap()
    }

    fn summary(id: &str, focus: Option<&str>) -> SessionSummary {
        SessionSummary {
            session_id: id.into(),
            agent_type: AgentType::Forge,
            pid: 1,
            status: AgentStatus::Working,
            focus_file: focus.map(PathBuf::from),
            last_heartbeat: ts(),
        }
    }

    #[test]
    fn empty_query_default_roundtrip() {
        let original = DiscoveryQuery::default();
        let s = serde_json::to_string(&original).unwrap();
        // An all-default query must serialize to `{}`-like data, not be
        // skipped (so receivers always see a stable shape).
        assert!(s.contains("\"path\":null"), "got: {s}");
        assert!(s.contains("\"branch\":null"), "got: {s}");
        assert!(s.contains("\"repo\":null"), "got: {s}");
        assert!(s.contains("\"capabilities\":[]"), "got: {s}");
        let back: DiscoveryQuery = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn populated_query_roundtrip() {
        let original = DiscoveryQuery {
            path: Some(PathBuf::from("/repo/src/lib.rs")),
            branch: Some("main".into()),
            repo: Some(PathBuf::from("/repo")),
            capabilities: vec!["rust".into(), "git:write".into()],
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: DiscoveryQuery = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn empty_result_default_roundtrip() {
        let original = DiscoveryResult::default();
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("\"sessions\":[]"), "got: {s}");
        let back: DiscoveryResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn populated_result_roundtrip() {
        let original = DiscoveryResult {
            sessions: vec![
                summary("s1", Some("/repo/src/lib.rs")),
                summary("s2", None),
            ],
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: DiscoveryResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
        assert_eq!(back.sessions.len(), 2);
        assert_eq!(back.sessions[0].session_id, "s1");
        assert_eq!(back.sessions[0].focus_file, Some(PathBuf::from("/repo/src/lib.rs")));
        assert!(back.sessions[1].focus_file.is_none());
    }
}
