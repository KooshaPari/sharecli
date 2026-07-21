// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm-daemon` — long-running multi-agent coordinator.
//!
//! M0 scope:
//! - Unix-socket listener with newline-delimited JSON-RPC 2.0 framing.
//! - In-memory state: sessions, reservations stub, live state, inbox.
//! - Handlers: `session.register`, `session.deregister`, `session.heartbeat`.
//! - `clap` CLI: `start [--foreground]`, `stop`, `status`.
//! - PID-file management with "refuse if a daemon is already running"
//!   semantics and stale-pid recovery.
//!
//! M1 will replace [`state::AppStateInner`] with a SQLite-backed store
//! and add the rest of the method catalogue (reservations, inbox,
//! discovery, hook events).
//!
//! ## Module map
//!
//! - [`config`]   — paths, timeouts, log level.
//! - [`error`]    — daemon error type and JSON-RPC code mapping.
//! - [`state`]    — in-memory shared state and id minters.
//! - [`pid`]      — PID file read/write/remove and "is it running" probe.
//! - [`handlers`] — JSON-RPC method bodies.
//! - [`listener`] — UnixListener accept loop, signal handling, dispatch.

pub mod config;
pub mod error;
pub mod db;
pub mod handlers;
pub mod listener;
pub mod pid;
pub mod state;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::watch;

pub use config::{default_paths, DaemonConfig};
pub use error::TeamcommError;
pub use state::{new_state, AppState, AppStateInner};

/// Long-lived daemon handle. Wraps the resolved configuration, the
/// shared in-memory state, and the shutdown broadcast channel.
///
/// Use [`Daemon::new`] to construct, then either:
/// - call [`Daemon::run_foreground`] for the in-process main loop, or
/// - call [`Daemon::shutdown_handle`] to get a [`DaemonHandle`] that
///   can be sent across tasks / threads to trigger graceful shutdown
///   (e.g. from a Ctrl-C handler in `main`).
pub struct Daemon {
    config: DaemonConfig,
    state: AppState,
    /// Held so we can hand out [`DaemonHandle`] clones; never actually
    /// `.send()`-ed by the struct itself (the listener drives the
    /// shutdown watch via its own receiver).
    #[allow(dead_code)]
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
}

impl Daemon {
    /// Build a new [`Daemon`] with a fresh, empty [`AppState`].
    pub fn new(config: DaemonConfig) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            config,
            state: new_state(),
            shutdown_tx,
            shutdown_rx,
        }
    }

    /// Build a new [`Daemon`] reusing a caller-supplied [`AppState`].
    /// Useful for tests that want to share a state across multiple
    /// listeners.
    pub fn with_state(config: DaemonConfig, state: AppState) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            config,
            state,
            shutdown_tx,
            shutdown_rx,
        }
    }

    /// Read-only access to the daemon configuration.
    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    /// Cloneable reference to the shared in-memory state.
    pub fn state(&self) -> AppState {
        Arc::clone(&self.state)
    }

    /// Get a [`DaemonHandle`] that can be used to trigger graceful
    /// shutdown from any task or thread.
    pub fn shutdown_handle(&self) -> DaemonHandle {
        DaemonHandle {
            tx: self.shutdown_tx.clone(),
        }
    }

    /// Run the listener loop in the current task. Returns when the
    /// shutdown signal is observed and all in-flight connections are
    /// drained.
    pub async fn run_foreground(self) -> Result<()> {
        listener::run(self.config, self.state, self.shutdown_rx).await
    }
}

/// Cheap, cloneable shutdown trigger.
///
/// Clone freely; all clones refer to the same underlying broadcast
/// channel. Calling [`DaemonHandle::shutdown`] is idempotent.
#[derive(Clone)]
pub struct DaemonHandle {
    tx: watch::Sender<bool>,
}

impl DaemonHandle {
    /// Trigger graceful shutdown. Safe to call multiple times.
    pub fn shutdown(&self) {
        let _ = self.tx.send(true);
    }

    /// Borrow the current shutdown state.
    pub fn is_shutting_down(&self) -> bool {
        *self.tx.borrow()
    }
}
