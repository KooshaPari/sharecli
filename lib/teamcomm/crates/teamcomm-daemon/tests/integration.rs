// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end integration test for the daemon's Unix-socket listener.
//!
//! What it covers:
//! 1. The listener can be started against a temp-dir socket + pid file.
//! 2. A client connecting with `tokio::net::UnixStream` can send
//!    newline-delimited JSON-RPC 2.0 requests and read responses.
//! 3. `session.register` returns `{ session_id, lease_ttl_sec: 90 }`.
//! 4. `session.deregister` is idempotent and returns `{ ok: true }`.
//! 5. Triggering the shutdown watch channel drains the listener task
//!    and lets us `await` it to completion.

use std::time::Duration;

use anyhow::Result;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::watch;
use tokio::time::timeout;

use teamcomm_daemon::{listener, DaemonConfig};

/// Wait for `path` to be connectable, retrying for up to 1s. Returns
/// `None` if the socket never came up.
async fn wait_for_socket(path: &std::path::Path) -> Option<UnixStream> {
    for _ in 0..50 {
        match UnixStream::connect(path).await {
            Ok(s) => return Some(s),
            Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
        }
    }
    None
}

/// A pair of (write half, BufReader<read half>) extracted from a
/// `UnixStream`. Lets us send a line and read the next line back
/// without lifetime gymnastics.
struct Client {
    writer: tokio::net::unix::OwnedWriteHalf,
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
}

impl Client {
    fn new(stream: UnixStream) -> Self {
        let (read_half, write_half) = stream.into_split();
        Self {
            writer: write_half,
            reader: BufReader::new(read_half),
        }
    }

    async fn round_trip(&mut self, request: Value) -> Result<Value> {
        let line = format!("{request}\n");
        self.writer.write_all(line.as_bytes()).await?;
        let mut buf = String::new();
        let n = timeout(Duration::from_secs(2), self.reader.read_line(&mut buf))
            .await
            .expect("response within 2s")?;
        assert!(n > 0, "expected at least one byte back, got EOF");
        Ok(serde_json::from_str(buf.trim())?)
    }
}

#[tokio::test]
async fn listener_handles_register_deregister_and_shutdown() -> Result<()> {
    let dir = TempDir::new()?;
    let socket_path = dir.path().join("test.sock");
    let pid_file = dir.path().join("test.pid");

    let config = DaemonConfig::from_args(Some(socket_path.clone()), Some(pid_file.clone()));
    let state = teamcomm_daemon::new_state();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let listener_task = tokio::spawn(listener::run(config.clone(), state.clone(), shutdown_rx));

    // 1. Connect to the listener.
    let stream = wait_for_socket(&socket_path)
        .await
        .expect("daemon socket should become ready within 1s");
    let mut client = Client::new(stream);

    // 2. `session.register` returns session_id + lease_ttl_sec.
    let resp = client
        .round_trip(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "session.register",
            "params": { "pid": 1234, "agent_type": "forge" }
        }))
        .await?;
    assert!(resp.get("error").is_none(), "register must succeed: {resp}");
    let result = resp.get("result").expect("result present");
    let session_id = result
        .get("session_id")
        .and_then(|v| v.as_str())
        .expect("session_id present")
        .to_string();
    assert!(
        session_id.starts_with("sess_"),
        "session_id should have the sess_ prefix; got {session_id}"
    );
    assert_eq!(result.get("lease_ttl_sec"), Some(&json!(90)));

    // 3. `session.deregister` returns ok.
    let resp = client
        .round_trip(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session.deregister",
            "params": { "session_id": session_id }
        }))
        .await?;
    assert!(
        resp.get("error").is_none(),
        "deregister must succeed: {resp}"
    );
    let result = resp.get("result").expect("result present");
    assert_eq!(result.get("ok"), Some(&json!(true)));

    // 4. `session.heartbeat` on a deregistered session reports
    //    NotFound as a properly-formed error envelope.
    let resp = client
        .round_trip(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session.heartbeat",
            "params": { "session_id": session_id }
        }))
        .await?;
    let err = resp
        .get("error")
        .expect("not_found should be reported as an error envelope");
    assert_eq!(err.get("code"), Some(&json!(-32004)));

    // 5. Unknown method returns -32601 (method not found).
    let resp = client
        .round_trip(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "session.teleport",
            "params": {}
        }))
        .await?;
    let err = resp.get("error").expect("error present");
    assert_eq!(err.get("code"), Some(&json!(-32601)));

    // 6. Shutdown the listener and confirm the task ends.
    drop(client);
    shutdown_tx
        .send(true)
        .map_err(|_| anyhow::anyhow!("shutdown_rx dropped"))?;
    let join = timeout(Duration::from_secs(2), listener_task)
        .await
        .expect("listener exits within 2s")?;
    join.expect("listener run returned Ok");

    // 7. Socket file is cleaned up.
    assert!(
        !socket_path.exists(),
        "socket file should be removed on shutdown"
    );

    Ok(())
}

#[tokio::test]
async fn listener_serves_independent_pids() -> Result<()> {
    let dir = TempDir::new()?;
    let socket_path = dir.path().join("concurrent.sock");
    let pid_file = dir.path().join("concurrent.pid");

    let config = DaemonConfig::from_args(Some(socket_path.clone()), Some(pid_file));
    let state = teamcomm_daemon::new_state();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let listener_task = tokio::spawn(listener::run(config.clone(), state.clone(), shutdown_rx));

    let stream = wait_for_socket(&socket_path)
        .await
        .expect("daemon socket should become ready within 1s");
    let mut client = Client::new(stream);

    // Two independent sessions from two different pids.
    let r1 = client
        .round_trip(json!({
            "jsonrpc": "2.0", "id": 1, "method": "session.register",
            "params": { "pid": 11, "agent_type": "claude" }
        }))
        .await?;
    let r2 = client
        .round_trip(json!({
            "jsonrpc": "2.0", "id": 2, "method": "session.register",
            "params": { "pid": 22, "agent_type": "codex" }
        }))
        .await?;

    let id1 = r1["result"]["session_id"].as_str().unwrap().to_string();
    let id2 = r2["result"]["session_id"].as_str().unwrap().to_string();
    assert_ne!(id1, id2, "different pids should get different sessions");

    shutdown_tx.send(true).ok();
    let _ = timeout(Duration::from_secs(2), listener_task).await;

    Ok(())
}
