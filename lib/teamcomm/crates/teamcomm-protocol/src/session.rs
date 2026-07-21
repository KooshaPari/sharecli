// SPDX-License-Identifier: MIT OR Apache-2.0
//! Agent session types: identity, registration payload, and lightweight summary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// What kind of coding agent is running a session.
///
/// `Custom(String)` lets the protocol grow without a coordinated change here
/// — a new agent just uses `custom` plus a stable string identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Forge (this codebase's primary agent).
    Forge,
    /// Codex.
    Codex,
    /// Claude Code.
    Claude,
    /// GitHub Copilot.
    Copilot,
    /// Anything not in the known set; the wrapped string is a stable
    /// implementation-defined identifier (e.g. `"aider"`, `"cursor"`).
    Custom(String),
}

/// A live, registered agent session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Stable, unique session identifier (typically a UUID v4).
    pub session_id: String,
    /// Which agent is running the session.
    pub agent_type: AgentType,
    /// OS-level process id of the agent at registration time.
    pub pid: u32,
    /// When the session was registered with the daemon.
    pub started_at: DateTime<Utc>,
    /// Working directory the agent declared at registration.
    pub working_dir: PathBuf,
    /// Free-form capability tags (e.g. `["rust", "git:write", "network"]`).
    pub capabilities: Vec<String>,
    /// Last heartbeat observed for this session. Updated on every
    /// `heartbeat` / `state` push from the agent.
    pub last_heartbeat: DateTime<Utc>,
}

/// Payload sent by an agent to register itself with the daemon.
///
/// The daemon assigns `session_id` and stamps `started_at`; the client only
/// supplies the rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRegistration {
    /// Which agent is registering.
    pub agent_type: AgentType,
    /// Process id of the registering agent.
    pub pid: u32,
    /// Working directory the agent will operate in.
    pub working_dir: PathBuf,
    /// Free-form capability tags.
    pub capabilities: Vec<String>,
}

/// Lightweight summary of a session, suitable for `list_sessions` results
/// and discovery responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session identifier.
    pub session_id: String,
    /// Agent type.
    pub agent_type: AgentType,
    /// OS process id.
    pub pid: u32,
    /// Current [`crate::state::AgentStatus`].
    pub status: crate::state::AgentStatus,
    /// The file the agent is currently focused on, if any.
    pub focus_file: Option<PathBuf>,
    /// Last heartbeat time.
    pub last_heartbeat: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AgentStatus;
    use chrono::TimeZone;

    fn sample_started_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
    }

    #[test]
    fn agent_type_serializes_as_snake_case() {
        // Unit variants serialize as their bare snake_case string.
        assert_eq!(
            serde_json::to_value(AgentType::Forge).unwrap(),
            serde_json::json!("forge")
        );
        assert_eq!(
            serde_json::to_value(AgentType::Codex).unwrap(),
            serde_json::json!("codex")
        );
        assert_eq!(
            serde_json::to_value(AgentType::Claude).unwrap(),
            serde_json::json!("claude")
        );
        assert_eq!(
            serde_json::to_value(AgentType::Copilot).unwrap(),
            serde_json::json!("copilot")
        );
        // `Custom(String)` is a newtype variant, so the wrapped string becomes
        // the *value* of a struct-shaped object: `{"custom": "aider"}`.
        // That wire shape is verified in `agent_type_custom_carries_inner_string`.
    }

    #[test]
    fn agent_type_custom_carries_inner_string() {
        let original = AgentType::Custom("aider".into());
        // Newtype variant → struct-shaped object: `{"custom": "aider"}`.
        assert_eq!(
            serde_json::to_value(&original).unwrap(),
            serde_json::json!({"custom": "aider"})
        );
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("aider"), "missing inner string in: {s}");
        let back: AgentType = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn session_roundtrip() {
        let started = sample_started_at();
        let heartbeat = Utc.with_ymd_and_hms(2026, 1, 2, 3, 5, 0).unwrap();
        let original = Session {
            session_id: "sess-1".into(),
            agent_type: AgentType::Forge,
            pid: 4242,
            started_at: started,
            working_dir: PathBuf::from("/tmp/proj"),
            capabilities: vec!["rust".into(), "git:write".into()],
            last_heartbeat: heartbeat,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: Session = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn session_registration_roundtrip() {
        let original = SessionRegistration {
            agent_type: AgentType::Claude,
            pid: 99,
            working_dir: PathBuf::from("/home/me/repo"),
            capabilities: vec!["read".into()],
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: SessionRegistration = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn session_summary_roundtrip() {
        let original = SessionSummary {
            session_id: "sess-2".into(),
            agent_type: AgentType::Codex,
            pid: 7,
            status: AgentStatus::Working,
            focus_file: Some(PathBuf::from("/repo/src/lib.rs")),
            last_heartbeat: sample_started_at(),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: SessionSummary = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn session_summary_with_no_focus_serializes() {
        let original = SessionSummary {
            session_id: "sess-3".into(),
            agent_type: AgentType::Copilot,
            pid: 1,
            status: AgentStatus::Idle,
            focus_file: None,
            last_heartbeat: sample_started_at(),
        };
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("\"focus_file\":null"), "got: {s}");
        let back: SessionSummary = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }
}
