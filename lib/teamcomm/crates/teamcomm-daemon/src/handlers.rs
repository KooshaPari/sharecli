// SPDX-License-Identifier: MIT OR Apache-2.0
//! JSON-RPC method handlers for M0.
//!
//! Each handler takes the shared [`AppState`] and the raw JSON-RPC
//! `params` value, and returns either a successful JSON result payload
//! or a [`TeamcommError`]. The listener wraps the result in the
//! `JsonRpcResponse` envelope; the error is mapped to a
//! `JsonRpcErrorResponse` by the listener.

use std::path::PathBuf;

use chrono::Utc;
use serde_json::{json, Value};
use tracing::{debug, info};

use teamcomm_protocol::rpc::RpcId;
use teamcomm_protocol::{
    AgentStatus, AgentType, InboxMessage, LiveState, Priority, Reservation, ReservationMode,
    Session, SessionSummary,
};

use crate::error::TeamcommError;
use crate::state::{mint_message_id, mint_reservation_id, mint_session_id, AppState};

/// Heartbeat interval the daemon recommends to agents, in seconds.
///
/// Returned in `session.heartbeat` responses so clients can dynamically
/// pick up a new cadence without redeploying.
pub const HEARTBEAT_INTERVAL_SEC: u64 = 30;

/// Session lease length, in seconds. After `LEASE_TTL_SEC` of no
/// heartbeats, the daemon may reap the session. Returned in
/// `session.register` responses.
pub const LEASE_TTL_SEC: u64 = 90;

/// `session.register` — create or refresh a session for the given pid.
///
/// Idempotent on `pid`: if the same pid re-registers, the existing
/// session is reused and its `last_heartbeat` is bumped. This makes
/// at-startup agent restarts safe.
pub async fn handle_session_register(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let pid = payload
        .get("pid")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: pid".into()))?
        as u32;

    let agent_kind_str = payload
        .get("agent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("custom")
        .to_string();
    let agent_type = parse_agent_type(&agent_kind_str);

    let working_dir: PathBuf = payload
        .get("working_dir")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));

    let capabilities: Vec<String> = payload
        .get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let now = Utc::now();

    // Idempotency: re-use the existing session for this pid, if any.
    let session_id = {
        let mut guard = state.write().await;
        if let Some(existing_id) = guard.sessions_by_pid.get(&pid).cloned() {
            if let Some(existing) = guard.sessions.get_mut(&existing_id) {
                existing.last_heartbeat = now;
                debug!(pid, session_id = %existing_id, "re-registered existing session");
                existing_id
            } else {
                // Reverse index is out of sync — recover by re-inserting
                // under a fresh id. Should not normally happen.
                insert_new_session(&mut guard, pid, agent_type, working_dir, capabilities, now)
            }
        } else {
            insert_new_session(&mut guard, pid, agent_type, working_dir, capabilities, now)
        }
    };

    info!(session_id = %session_id, pid, agent = %agent_kind_str, "session registered");

    Ok(json!({
        "session_id": session_id,
        "lease_ttl_sec": LEASE_TTL_SEC,
    }))
}

/// `session.deregister` — remove a session by id.
///
/// Missing session is a no-op success: we return `{"ok": true}` so that
/// at-shutdown cleanup requests are idempotent.
pub async fn handle_session_deregister(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let mut guard = state.write().await;
    let removed = guard.sessions.remove(&session_id);
    if let Some(sess) = &removed {
        guard.sessions_by_pid.remove(&sess.pid);
    }
    drop(guard);

    info!(session_id = %session_id, removed = removed.is_some(), "session deregistered");

    Ok(json!({ "ok": true }))
}

/// `session.heartbeat` — refresh a session's `last_heartbeat` and report
/// the recommended next-heartbeat interval.
pub async fn handle_session_heartbeat(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let now = Utc::now();
    {
        let mut guard = state.write().await;
        let session = guard
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| TeamcommError::NotFound(format!("session {session_id}")))?;
        session.last_heartbeat = now;
        // Status is tracked on `LiveState` in the protocol crate, not on
        // `Session` itself. M0 doesn't receive explicit state pushes, so
        // we leave the agent's previously-reported status untouched.
    }

    debug!(session_id = %session_id, "heartbeat");

    Ok(json!({
        "ok": true,
        "next_heartbeat_sec": HEARTBEAT_INTERVAL_SEC,
    }))
}

/// `session.list` — return all registered sessions.
pub async fn handle_session_list(state: AppState, _params: Value) -> Result<Value, TeamcommError> {
    let guard = state.read().await;
    let sessions: Vec<&Session> = guard.sessions.values().collect();
    Ok(json!(sessions))
}

/// `session.get` — return a single session by id.
pub async fn handle_session_get(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let guard = state.read().await;
    let session = guard
        .sessions
        .get(&session_id)
        .ok_or_else(|| TeamcommError::NotFound(format!("session {session_id}")))?;
    Ok(json!(session))
}

// ===== Reservation handlers =====

/// `reservation.claim` — claim an advisory lock on a path.
pub async fn handle_reservation_claim(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let path = payload
        .get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: path".into()))?;

    let mode_str = payload
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("write");
    let mode = parse_reservation_mode(mode_str)?;

    let ttl_sec = payload
        .get("ttl_sec")
        .and_then(|v| v.as_u64())
        .unwrap_or(600);

    let now = Utc::now();
    let expires = now + chrono::Duration::seconds(ttl_sec as i64);

    let mut guard = state.write().await;

    // Verify session exists.
    if !guard.sessions.contains_key(&session_id) {
        return Err(TeamcommError::NotFound(format!("session {session_id}")));
    }

    // Check for conflicts.
    let conflicts: Vec<Reservation> = guard
        .reservations
        .values()
        .filter(|r| {
            r.path == path
                && r.session_id != session_id
                && r.expires_at > now
                && mode_conflicts(mode, r.mode)
        })
        .cloned()
        .collect();

    let reservation_id = mint_reservation_id();
    let reservation = Reservation {
        reservation_id: reservation_id.clone(),
        session_id: session_id.clone(),
        path: path.clone(),
        mode,
        acquired_at: now,
        expires_at: expires,
    };

    guard
        .reservations
        .insert(reservation_id.clone(), reservation);
    drop(guard);

    info!(session_id = %session_id, reservation_id = %reservation_id, path = %path.display(), "reservation claimed");

    Ok(json!({
        "reservation_id": reservation_id,
        "conflicts": conflicts,
    }))
}

/// `reservation.release` — release a reservation by id.
pub async fn handle_reservation_release(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let reservation_id = payload
        .get("reservation_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            TeamcommError::InvalidParams("missing required field: reservation_id".into())
        })?
        .to_string();

    let mut guard = state.write().await;
    let removed = guard.reservations.remove(&reservation_id);
    drop(guard);

    if removed.is_none() {
        return Err(TeamcommError::NotFound(format!(
            "reservation {reservation_id}"
        )));
    }

    info!(reservation_id = %reservation_id, "reservation released");
    Ok(json!({ "ok": true }))
}

/// `reservation.list` — return all active reservations.
pub async fn handle_reservation_list(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let path_filter = payload
        .get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);

    let guard = state.read().await;
    let now = Utc::now();
    let mut reservations: Vec<&Reservation> = guard
        .reservations
        .values()
        .filter(|r| r.expires_at > now)
        .collect();

    if let Some(prefix) = path_filter {
        let prefix_str = prefix.to_string_lossy();
        reservations.retain(|r| r.path.to_string_lossy().starts_with(&*prefix_str));
    }

    Ok(json!(reservations))
}

// ===== Inbox handlers =====

/// `inbox.post` — post a message to another session.
pub async fn handle_inbox_post(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let from_session = payload
        .get("from_session")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: from_session".into()))?
        .to_string();

    let to_session = payload
        .get("to_session")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: to_session".into()))?
        .to_string();

    let subject = payload
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let priority_str = payload
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");
    let priority = parse_priority(priority_str)?;

    let mut guard = state.write().await;

    // Verify from_session exists.
    if !guard.sessions.contains_key(&from_session) {
        return Err(TeamcommError::NotFound(format!("session {from_session}")));
    }

    let message = InboxMessage {
        message_id: mint_message_id(),
        from_session: from_session.clone(),
        to_session: to_session.clone(),
        subject,
        body,
        priority,
        ts: Utc::now(),
        read: false,
    };

    let msg_id = message.message_id.clone();
    guard
        .inbox
        .entry(to_session.clone())
        .or_default()
        .push(message);
    drop(guard);

    info!(msg_id = %msg_id, from = %from_session, to = %to_session, "inbox message posted");
    Ok(json!({ "message_id": msg_id }))
}

/// `inbox.list` — list messages for a session.
pub async fn handle_inbox_list(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let unread_only = payload
        .get("unread_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let limit = payload.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    let mut guard = state.write().await;
    let messages = guard.inbox.entry(session_id.clone()).or_default();

    // Collect ids to mark as read first, then collect result.
    let result_ids: Vec<String> = messages
        .iter()
        .filter(|m| !unread_only || !m.read)
        .take(if limit > 0 { limit } else { usize::MAX })
        .map(|m| m.message_id.clone())
        .collect();

    for m in messages.iter_mut() {
        if result_ids.contains(&m.message_id) {
            m.read = true;
        }
    }

    let result: Vec<InboxMessage> = messages
        .iter()
        .filter(|m| result_ids.contains(&m.message_id))
        .cloned()
        .collect();
    drop(guard);

    Ok(json!(result))
}

/// `inbox.read` — read a single message by id.
pub async fn handle_inbox_read(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let message_id = payload
        .get("message_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: message_id".into()))?
        .to_string();

    let mut guard = state.write().await;
    for messages in guard.inbox.values_mut() {
        if let Some(m) = messages.iter_mut().find(|m| m.message_id == message_id) {
            m.read = true;
            return Ok(json!(m));
        }
    }
    drop(guard);

    Err(TeamcommError::NotFound(format!("message {message_id}")))
}

// ===== State handlers =====

/// `state.set` — publish live state for a session.
pub async fn handle_state_set(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let focus_file = payload
        .get("focus_file")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);

    let focus_branch = payload
        .get("focus_branch")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let worktree = payload
        .get("worktree")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);

    let status_str = payload
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("idle");
    let status = parse_agent_status(status_str)?;

    let mut guard = state.write().await;
    if !guard.sessions.contains_key(&session_id) {
        return Err(TeamcommError::NotFound(format!("session {session_id}")));
    }

    let live = LiveState {
        session_id: session_id.clone(),
        focus_file,
        focus_branch,
        worktree,
        status,
        last_heartbeat: Utc::now(),
    };
    guard.live_state.insert(session_id.clone(), live);
    drop(guard);

    info!(session_id = %session_id, "live state updated");
    Ok(json!({ "ok": true }))
}

/// `state.get` — get live state for a session.
pub async fn handle_state_get(state: AppState, payload: Value) -> Result<Value, TeamcommError> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TeamcommError::InvalidParams("missing required field: session_id".into()))?
        .to_string();

    let guard = state.read().await;
    let live = guard
        .live_state
        .get(&session_id)
        .ok_or_else(|| TeamcommError::NotFound(format!("live state for session {session_id}")))?;
    Ok(json!(live))
}

// ===== Discovery handlers =====

/// `discover.agents` — query sessions by filters.
pub async fn handle_discover_agents(
    state: AppState,
    payload: Value,
) -> Result<Value, TeamcommError> {
    let path_filter = payload
        .get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);
    let branch_filter = payload
        .get("branch")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let capabilities_filter: Vec<String> = payload
        .get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let guard = state.read().await;
    let mut summaries: Vec<SessionSummary> = Vec::new();

    for (id, session) in guard.sessions.iter() {
        let live = guard.live_state.get(id);
        let status = live.map(|l| l.status).unwrap_or(AgentStatus::Idle);
        let focus_file = live.and_then(|l| l.focus_file.clone());

        // Apply filters.
        if let Some(ref prefix) = path_filter {
            if let Some(ref fp) = focus_file {
                if !fp.to_string_lossy().starts_with(&*prefix.to_string_lossy()) {
                    continue;
                }
            } else {
                continue;
            }
        }
        if let Some(ref b) = branch_filter {
            if live.map(|l| l.focus_branch.as_ref()) != Some(Some(b)) {
                continue;
            }
        }
        if !capabilities_filter.is_empty() {
            let caps: std::collections::HashSet<&String> = session.capabilities.iter().collect();
            if !capabilities_filter.iter().all(|c| caps.contains(c)) {
                continue;
            }
        }

        summaries.push(SessionSummary {
            session_id: id.clone(),
            agent_type: session.agent_type.clone(),
            pid: session.pid,
            status,
            focus_file,
            last_heartbeat: session.last_heartbeat,
        });
    }

    Ok(json!(summaries))
}

// ----- helpers -----

/// Insert a freshly-minted [`Session`] into the in-memory state and
/// return its id. Caller must already hold a write lock.
fn insert_new_session(
    guard: &mut crate::state::AppStateInner,
    pid: u32,
    agent_type: AgentType,
    working_dir: PathBuf,
    capabilities: Vec<String>,
    now: chrono::DateTime<Utc>,
) -> String {
    let session_id = mint_session_id();
    let session = Session {
        session_id: session_id.clone(),
        agent_type,
        pid,
        started_at: now,
        working_dir,
        capabilities,
        last_heartbeat: now,
    };
    guard.sessions.insert(session_id.clone(), session);
    guard.sessions_by_pid.insert(pid, session_id.clone());
    session_id
}

/// Map a free-form agent kind string to the typed [`AgentType`] enum.
fn parse_agent_type(s: &str) -> AgentType {
    match s.to_ascii_lowercase().as_str() {
        "forge" => AgentType::Forge,
        "codex" => AgentType::Codex,
        "claude" => AgentType::Claude,
        "copilot" => AgentType::Copilot,
        other => AgentType::Custom(other.to_string()),
    }
}

fn parse_reservation_mode(s: &str) -> Result<ReservationMode, TeamcommError> {
    match s.to_ascii_lowercase().as_str() {
        "read" => Ok(ReservationMode::Read),
        "write" => Ok(ReservationMode::Write),
        "exclusive" => Ok(ReservationMode::Exclusive),
        other => Err(TeamcommError::InvalidParams(format!(
            "unknown reservation mode: {other}"
        ))),
    }
}

fn parse_priority(s: &str) -> Result<Priority, TeamcommError> {
    match s.to_ascii_lowercase().as_str() {
        "low" => Ok(Priority::Low),
        "normal" => Ok(Priority::Normal),
        "high" => Ok(Priority::High),
        other => Err(TeamcommError::InvalidParams(format!(
            "unknown priority: {other}"
        ))),
    }
}

fn parse_agent_status(s: &str) -> Result<AgentStatus, TeamcommError> {
    match s.to_ascii_lowercase().as_str() {
        "idle" => Ok(AgentStatus::Idle),
        "working" => Ok(AgentStatus::Working),
        "blocked" => Ok(AgentStatus::Blocked),
        "done" => Ok(AgentStatus::Done),
        "stuck" => Ok(AgentStatus::Stuck),
        other => Err(TeamcommError::InvalidParams(format!(
            "unknown agent status: {other}"
        ))),
    }
}

fn mode_conflicts(new: ReservationMode, existing: ReservationMode) -> bool {
    match new {
        ReservationMode::Read => matches!(existing, ReservationMode::Exclusive),
        ReservationMode::Write => matches!(
            existing,
            ReservationMode::Write | ReservationMode::Exclusive
        ),
        ReservationMode::Exclusive => true,
    }
}

/// Convenience for the listener: given an [`RpcId`] and a successful
/// handler result, build a JSON-RPC success envelope.
pub fn success_envelope(id: RpcId, result: Value) -> teamcomm_protocol::rpc::JsonRpcResponse {
    teamcomm_protocol::rpc::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
    }
}

/// Convenience for the listener: given an [`RpcId`] and a
/// [`TeamcommError`], build a JSON-RPC error envelope.
pub fn error_envelope(
    id: RpcId,
    err: TeamcommError,
) -> teamcomm_protocol::rpc::JsonRpcErrorResponse {
    err.into_response(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_state;

    #[tokio::test]
    async fn register_assigns_session_id_and_lease() {
        let state = new_state();
        let result =
            handle_session_register(state.clone(), json!({ "pid": 1001, "agent_type": "forge" }))
                .await
                .unwrap();
        assert!(result["session_id"].as_str().unwrap().starts_with("sess_"));
        assert_eq!(result["lease_ttl_sec"], LEASE_TTL_SEC);
    }

    #[tokio::test]
    async fn register_is_idempotent_on_pid() {
        let state = new_state();
        let r1 = handle_session_register(state.clone(), json!({ "pid": 2002 }))
            .await
            .unwrap();
        let id1 = r1["session_id"].as_str().unwrap().to_string();
        let r2 = handle_session_register(state.clone(), json!({ "pid": 2002 }))
            .await
            .unwrap();
        let id2 = r2["session_id"].as_str().unwrap().to_string();
        assert_eq!(id1, id2, "second register must reuse the first session");
    }

    #[tokio::test]
    async fn register_rejects_missing_pid() {
        let state = new_state();
        let err = handle_session_register(state.clone(), json!({}))
            .await
            .unwrap_err();
        assert!(
            matches!(err, TeamcommError::InvalidParams(_)),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn deregister_returns_ok() {
        let state = new_state();
        let r = handle_session_register(state.clone(), json!({ "pid": 1 }))
            .await
            .unwrap();
        let id = r["session_id"].as_str().unwrap();
        let out = handle_session_deregister(state.clone(), json!({ "session_id": id }))
            .await
            .unwrap();
        assert_eq!(out, json!({ "ok": true }));
    }

    #[tokio::test]
    async fn deregister_unknown_session_is_idempotent_ok() {
        let state = new_state();
        let out = handle_session_deregister(state.clone(), json!({ "session_id": "sess_nope" }))
            .await
            .unwrap();
        assert_eq!(out, json!({ "ok": true }));
    }

    #[tokio::test]
    async fn heartbeat_refreshes_and_returns_next_interval() {
        let state = new_state();
        let r = handle_session_register(state.clone(), json!({ "pid": 1 }))
            .await
            .unwrap();
        let id = r["session_id"].as_str().unwrap();
        let out = handle_session_heartbeat(state.clone(), json!({ "session_id": id }))
            .await
            .unwrap();
        assert_eq!(out["ok"], json!(true));
        assert_eq!(out["next_heartbeat_sec"], json!(HEARTBEAT_INTERVAL_SEC));
    }

    #[tokio::test]
    async fn heartbeat_unknown_session_returns_not_found() {
        let state = new_state();
        let err = handle_session_heartbeat(state.clone(), json!({ "session_id": "sess_nope" }))
            .await
            .unwrap_err();
        assert!(matches!(err, TeamcommError::NotFound(_)), "got {err:?}");
    }

    #[test]
    fn parse_agent_type_known_and_custom() {
        assert_eq!(parse_agent_type("forge"), AgentType::Forge);
        assert_eq!(parse_agent_type("CLAUDE"), AgentType::Claude);
        assert_eq!(
            parse_agent_type("aider"),
            AgentType::Custom("aider".to_string())
        );
    }
}
