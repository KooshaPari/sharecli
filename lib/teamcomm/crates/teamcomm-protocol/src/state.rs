// SPDX-License-Identifier: MIT OR Apache-2.0
//! Live agent state broadcast: what file/branch/worktree the agent is on and
//! what it's doing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Current working status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// The agent is registered but not actively working on a task.
    Idle,
    /// The agent is actively working (e.g. writing code, running tests).
    Working,
    /// The agent is blocked — waiting on a human, another agent, or a
    /// reservation to be released.
    Blocked,
    /// The agent has finished its current task and is awaiting next input.
    Done,
    /// The agent is internally stuck (e.g. repeated failure, lost context,
    /// hung subtool) and needs intervention or restart. Distinct from
    /// `Blocked` (external dependency) — `Stuck` means the agent itself
    /// is unresponsive or in a degenerate loop.
    Stuck,
}

/// Snapshot of an agent's current focus and status.
///
/// Pushed by the agent on every meaningful state change and on heartbeat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveState {
    /// Session this state belongs to.
    pub session_id: String,
    /// Path of the file the agent is currently focused on, if any.
    pub focus_file: Option<PathBuf>,
    /// Branch the agent is working on, if any.
    pub focus_branch: Option<String>,
    /// Worktree directory the agent is operating in, if any.
    pub worktree: Option<PathBuf>,
    /// Current status.
    pub status: AgentStatus,
    /// When this state snapshot was produced.
    pub last_heartbeat: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 12, 9, 30, 0).unwrap()
    }

    #[test]
    fn agent_status_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(AgentStatus::Idle).unwrap(),
            serde_json::json!("idle")
        );
        assert_eq!(
            serde_json::to_value(AgentStatus::Working).unwrap(),
            serde_json::json!("working")
        );
        assert_eq!(
            serde_json::to_value(AgentStatus::Blocked).unwrap(),
            serde_json::json!("blocked")
        );
        assert_eq!(
            serde_json::to_value(AgentStatus::Done).unwrap(),
            serde_json::json!("done")
        );
    }

    #[test]
    fn agent_status_roundtrip() {
        for s in [
            AgentStatus::Idle,
            AgentStatus::Working,
            AgentStatus::Blocked,
            AgentStatus::Done,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let back: AgentStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn live_state_roundtrip() {
        let original = LiveState {
            session_id: "sess-1".into(),
            focus_file: Some(PathBuf::from("/repo/src/lib.rs")),
            focus_branch: Some("feat/x".into()),
            worktree: Some(PathBuf::from("/repo.worktrees/x")),
            status: AgentStatus::Working,
            last_heartbeat: ts(),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: LiveState = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn live_state_with_all_optional_none() {
        let original = LiveState {
            session_id: "sess-2".into(),
            focus_file: None,
            focus_branch: None,
            worktree: None,
            status: AgentStatus::Idle,
            last_heartbeat: ts(),
        };
        let s = serde_json::to_string(&original).unwrap();
        // All three optional fields must serialize as `null`, not be omitted.
        assert!(s.contains("\"focus_file\":null"), "got: {s}");
        assert!(s.contains("\"focus_branch\":null"), "got: {s}");
        assert!(s.contains("\"worktree\":null"), "got: {s}");
        let back: LiveState = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }
}
