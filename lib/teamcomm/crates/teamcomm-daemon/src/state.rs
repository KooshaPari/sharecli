// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-memory daemon state for M0.
//!
//! M1 will replace this module's storage with a SQLite-backed
//! implementation; the public type alias [`AppState`] is what handlers
//! and the listener should depend on.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use teamcomm_protocol::{InboxMessage, LiveState, Reservation, Session};

/// In-memory state held behind a single async RwLock.
///
/// The fields are public for the M0 listener/handlers; once SQLite lands
/// we will narrow the visibility and expose query/mutate methods
/// instead.
#[derive(Debug, Default)]
pub struct AppStateInner {
    /// All live sessions, keyed by session id (the `Session::session_id`
    /// string).
    pub sessions: HashMap<String, Session>,

    /// Reverse index: process id -> session id. Lets `deregister` and
    /// `heartbeat` look up a session when only the pid is known (e.g.
    /// on agent exit detection).
    pub sessions_by_pid: HashMap<u32, String>,

    /// Active reservations, keyed by reservation id.
    pub reservations: HashMap<String, Reservation>,

    /// Latest live state per session.
    pub live_state: HashMap<String, LiveState>,

    /// In-memory inbox, keyed by recipient session id.
    pub inbox: HashMap<String, Vec<InboxMessage>>,
}

/// Shared, cheaply-clonable handle to the in-memory state.
pub type AppState = Arc<RwLock<AppStateInner>>;

/// Construct a fresh, empty [`AppState`].
pub fn new_state() -> AppState {
    Arc::new(RwLock::new(AppStateInner::default()))
}

/// Mint a new session id of the form `"sess_<uuid>"`.
///
/// Mirrors the convention used by the protocol crate's own test fixtures
/// so client and daemon agree on the wire format.
pub fn mint_session_id() -> String {
    format!("sess_{}", Uuid::new_v4().simple())
}

/// Mint a new reservation id of the form `"resv_<uuid>"`.
pub fn mint_reservation_id() -> String {
    format!("resv_{}", Uuid::new_v4().simple())
}

/// Mint a new message id of the form `"msg_<uuid>"`.
pub fn mint_message_id() -> String {
    format!("msg_{}", Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_is_empty() {
        let s = new_state();
        let guard = s.blocking_read();
        assert!(guard.sessions.is_empty());
        assert!(guard.sessions_by_pid.is_empty());
        assert!(guard.reservations.is_empty());
        assert!(guard.live_state.is_empty());
        assert!(guard.inbox.is_empty());
    }

    #[test]
    fn session_id_format() {
        let id = mint_session_id();
        assert!(id.starts_with("sess_"), "got {id}");
        let stripped = id.trim_start_matches("sess_");
        assert_eq!(stripped.len(), 32, "uuid simple is 32 hex chars");
        assert!(stripped.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn reservation_id_format() {
        let id = mint_reservation_id();
        assert!(id.starts_with("resv_"), "got {id}");
    }

    #[test]
    fn app_state_is_cheaply_clonable() {
        let s = new_state();
        let s2 = Arc::clone(&s);
        assert!(Arc::ptr_eq(&s, &s2));
    }
}
