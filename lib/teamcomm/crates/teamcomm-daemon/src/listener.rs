// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unix-socket listener and per-connection JSON-RPC dispatch loop.
//!
//! Wire format: one JSON-RPC 2.0 request per line on each connection,
//! one JSON-RPC 2.0 response (success or error) per line back. Empty
//! lines are ignored. EOF on the client side terminates the per-
//! connection task.
//!
//! The listener binds a `tokio::net::UnixListener` at
//! [`DaemonConfig::socket_path`], creates parent directories as needed,
//! and removes any stale socket file. It then races `accept()` against
//! a `tokio::sync::watch` shutdown signal — when the signal flips to
//! `true` the listener stops accepting, joins the per-connection
//! tasks, and removes its socket file.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{watch, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use teamcomm_protocol::rpc::{JsonRpcRequest, RpcId};

use crate::config::DaemonConfig;
use crate::error::TeamcommError;
use crate::handlers;
use crate::state::AppState;

/// Maximum length (in bytes) we are willing to read for a single
/// request line. Generous in M0; M1 will tighten this once we have a
/// payload-size policy.
const MAX_LINE_BYTES: usize = 1024 * 1024; // 1 MiB

/// How long an idle connection may sit before we close it.
const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Run the listener loop to completion. Returns once the shutdown
/// signal is observed and all in-flight connection tasks have joined.
pub async fn run(
    config: DaemonConfig,
    state: AppState,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let listener = bind_socket(&config.socket_path)
        .with_context(|| format!("failed to bind socket at {}", config.socket_path.display()))?;
    info!(socket = %config.socket_path.display(), "daemon listening");

    // Spawn a signal-handler task that flips a separate shutdown watch
    // when SIGINT or SIGTERM arrives. We use a *separate* watch channel
    // for signal-driven shutdown because the caller's `shutdown` is
    // already a `Receiver` (signals need a `Sender` to write to).
    // The accept loop below observes both via two `select!` arms.
    let (signal_tx, mut signal_rx) = watch::channel(false);
    tokio::spawn(async move {
        install_signal_handlers(signal_tx).await;
    });

    // Track in-flight connection tasks so we can join them on shutdown.
    let inflight: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    info!("external shutdown signal observed; draining connections");
                    break;
                }
            }
            _ = signal_rx.changed() => {
                if *signal_rx.borrow() {
                    info!("OS signal shutdown observed; draining connections");
                    break;
                }
            }
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _addr)) => {
                        let state = state.clone();
                        let conn_task = tokio::spawn(handle_connection(stream, state));
                        inflight.lock().await.push(conn_task);
                    }
                    Err(e) => {
                        // A transient accept error: log and keep going. If
                        // it's fatal the next accept() will report the
                        // same root cause.
                        warn!(error = %e, "accept error");
                    }
                }
            }
        }
    }

    // Drop the listener so the FD is released and the socket file can
    // be removed cleanly.
    drop(listener);

    // Abort in-flight connection tasks so they stop blocking on
    // `read_line` and exit promptly. We don't need to wait for them
    // to finish — the unix socket is already closed by `drop(listener)`
    // above, so the aborted tasks will return immediately.
    {
        let mut guard = inflight.lock().await;
        for task in guard.drain(..) {
            task.abort();
        }
    }

    remove_socket_file(&config.socket_path);

    Ok(())
}

/// Bind a [`UnixListener`] at `path`, creating the parent directory as
/// needed and removing any stale socket file.
fn bind_socket(path: &Path) -> Result<UnixListener> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create socket parent {}", parent.display()))?;
        }
    }
    if path.exists() {
        // Stale socket from a previous run. The pid-file check in
        // `pid::write_pid_file` guarantees we are not racing a live
        // daemon.
        fs::remove_file(path)
            .with_context(|| format!("failed to remove stale socket {}", path.display()))?;
    }
    let listener = UnixListener::bind(path)
        .with_context(|| format!("failed to bind unix listener at {}", path.display()))?;
    Ok(listener)
}

/// Remove a socket file, tolerating "already gone".
fn remove_socket_file(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => info!(path = %path.display(), "removed socket file"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!(path = %path.display(), error = %e, "failed to remove socket file"),
    }
}

/// Block until SIGINT or SIGTERM, then flip `tx` to `true` to broadcast
/// shutdown.
async fn install_signal_handlers(tx: watch::Sender<bool>) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigint = match signal(SignalKind::interrupt()) {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "failed to install SIGINT handler");
                return;
            }
        };
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "failed to install SIGTERM handler");
                return;
            }
        };
        tokio::select! {
            _ = sigint.recv()  => info!("SIGINT received"),
            _ = sigterm.recv() => info!("SIGTERM received"),
        }
    }
    #[cfg(not(unix))]
    {
        // Non-Unix fallback (mostly a no-op in practice — the daemon's
        // primary targets are macOS and Linux).
        let _ = tokio::signal::ctrl_c().await;
        info!("ctrl_c received");
    }
    let _ = tx.send(true);
}

/// Per-connection loop: read newline-delimited JSON-RPC requests and
/// dispatch them, writing one response line per request.
async fn handle_connection(stream: UnixStream, state: AppState) {
    let peer = stream.peer_addr().ok();
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        let read = match timeout(IDLE_TIMEOUT, reader.read_line(&mut line)).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                debug!(peer = ?peer, error = %e, "connection read error; closing");
                return;
            }
            Err(_) => {
                debug!(peer = ?peer, "idle timeout; closing connection");
                return;
            }
        };
        if read == 0 {
            debug!(peer = ?peer, "client closed connection");
            return;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.len() > MAX_LINE_BYTES {
            warn!(peer = ?peer, len = trimmed.len(), "rejecting oversize line");
            let env = error_envelope(
                RpcId::Number(0),
                TeamcommError::InvalidParams(format!(
                    "line exceeds MAX_LINE_BYTES ({MAX_LINE_BYTES})"
                )),
            );
            if write_envelope(&mut write_half, &env).await.is_err() {
                return;
            }
            continue;
        }

        // Parse the request envelope.
        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                warn!(peer = ?peer, error = %e, "malformed JSON-RPC request");
                let env = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32700,
                        "message": "parse error",
                        "data": e.to_string(),
                    }
                });
                if write_envelope(&mut write_half, &env).await.is_err() {
                    return;
                }
                continue;
            }
        };

        // Notifications: process but do not reply. M0 only logs them.
        let Some(rpc_id) = request.id.clone() else {
            debug!(method = %request.method, "received notification; no reply");
            continue;
        };

        let envelope = match dispatch(&request.method, request.params, state.clone()).await {
            Ok(result) => success_envelope(rpc_id, result),
            Err(err) => error_envelope(rpc_id, err),
        };

        if let Err(e) = write_envelope(&mut write_half, &envelope).await {
            debug!(peer = ?peer, error = %e, "failed to write response; closing");
            return;
        }
    }
}

/// Dispatch a single method name to its M0 handler. Any method that is
/// not in the M0 surface returns `MethodNotFound`.
async fn dispatch(method: &str, params: Value, state: AppState) -> Result<Value, TeamcommError> {
    match method {
        "session.register" => handlers::handle_session_register(state, params).await,
        "session.deregister" => handlers::handle_session_deregister(state, params).await,
        "session.heartbeat" => handlers::handle_session_heartbeat(state, params).await,
        "session.list" => handlers::handle_session_list(state, params).await,
        "session.get" => handlers::handle_session_get(state, params).await,
        "reservation.claim" => handlers::handle_reservation_claim(state, params).await,
        "reservation.release" => handlers::handle_reservation_release(state, params).await,
        "reservation.list" => handlers::handle_reservation_list(state, params).await,
        "inbox.post" => handlers::handle_inbox_post(state, params).await,
        "inbox.list" => handlers::handle_inbox_list(state, params).await,
        "inbox.read" => handlers::handle_inbox_read(state, params).await,
        "state.set" => handlers::handle_state_set(state, params).await,
        "state.get" => handlers::handle_state_get(state, params).await,
        "discover.agents" => handlers::handle_discover_agents(state, params).await,
        other => Err(TeamcommError::MethodNotFound(other.to_string())),
    }
}

fn success_envelope(id: RpcId, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": rpc_id_to_value(&id),
        "result": result,
    })
}

fn error_envelope(id: RpcId, err: TeamcommError) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": rpc_id_to_value(&id),
        "error": {
            "code": err.rpc_code(),
            "message": err.to_string(),
        }
    })
}

fn rpc_id_to_value(id: &RpcId) -> Value {
    match id {
        RpcId::String(s) => Value::String(s.clone()),
        RpcId::Number(n) => Value::Number(serde_json::Number::from(*n)),
    }
}

/// Serialise `envelope` to one line and flush.
async fn write_envelope(writer: &mut (impl AsyncWriteExt + Unpin), envelope: &Value) -> Result<()> {
    let line = serde_json::to_string(envelope).context("failed to serialise JSON-RPC envelope")?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_state;

    #[tokio::test]
    async fn bind_socket_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c/daemon.sock");
        let _listener = bind_socket(&nested).expect("bind");
        assert!(nested.exists(), "socket file should exist after bind");
    }

    #[tokio::test]
    async fn bind_socket_replaces_stale_file() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("daemon.sock");
        // Create a stale regular file at the socket path; bind should
        // overwrite it.
        fs::write(&sock, b"not a socket").unwrap();
        let _l2 = bind_socket(&sock).expect("bind should overwrite stale regular file");
    }

    #[tokio::test]
    async fn run_starts_and_stops_on_shutdown_signal() {
        use tokio::io::AsyncWriteExt as _;
        use tokio::net::UnixStream;
        let dir = tempfile::tempdir().unwrap();
        let cfg = DaemonConfig::from_args(
            Some(dir.path().join("daemon.sock")),
            Some(dir.path().join("daemon.pid")),
        );
        let state = new_state();
        let (tx, rx) = watch::channel(false);

        let listener_task = tokio::spawn(run(cfg.clone(), state.clone(), rx));

        // Wait for the listener to be ready by trying to connect; if
        // the socket isn't up yet, retry a few times.
        let mut stream = None;
        for _ in 0..50 {
            match UnixStream::connect(&cfg.socket_path).await {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
            }
        }
        let mut stream = stream.expect("daemon socket should be ready within 1s");

        // Send a register request.
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "session.register",
            "params": { "pid": 9999, "agent_type": "forge" }
        });
        let line = format!("{req}\n");
        stream.write_all(line.as_bytes()).await.unwrap();

        // Read the response line.
        let mut reader = BufReader::new(stream);
        let mut resp = String::new();
        let n = timeout(Duration::from_secs(2), reader.read_line(&mut resp))
            .await
            .expect("response within 2s")
            .expect("read ok");
        assert!(n > 0, "got a response line");
        let resp_json: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
        assert!(resp_json["result"]["session_id"]
            .as_str()
            .unwrap()
            .starts_with("sess_"));
        assert_eq!(resp_json["result"]["lease_ttl_sec"], 90);

        // Trigger shutdown.
        tx.send(true).unwrap();
        let join = timeout(Duration::from_secs(2), listener_task)
            .await
            .expect("listener exits within 2s")
            .expect("listener task did not panic");
        join.expect("listener run returned Ok");
    }
}
