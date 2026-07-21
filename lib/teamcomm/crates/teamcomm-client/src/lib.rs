// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm-client` — embeddable async client for talking to the teamcomm daemon.
//!
//! Connects over a Unix-domain socket and dispatches JSON-RPC 2.0 requests.
//! All methods return `anyhow::Result<serde_json::Value>` so the caller can
//! extract the concrete types they need.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Default socket path used when none is provided.
///
/// Uses `$XDG_RUNTIME_DIR/teamcomm/daemon.sock` or falls back to
/// `/tmp/teamcomm/daemon.sock`.
pub fn default_socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("teamcomm/daemon.sock")
}

/// Async client handle to the teamcomm daemon.
#[derive(Debug)]
pub struct Client {
    stream: Option<UnixStream>,
}

impl Client {
    /// Connect to the daemon at the given socket path.
    pub async fn connect(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let stream = UnixStream::connect(path.as_ref()).await.with_context(|| {
            format!("failed to connect to daemon at {}", path.as_ref().display())
        })?;
        Ok(Self {
            stream: Some(stream),
        })
    }

    /// Connect to the daemon at the default socket path.
    pub async fn connect_default() -> Result<Self> {
        Self::connect(default_socket_path()).await
    }

    /// Send a JSON-RPC 2.0 request and return the raw result payload.
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let mut payload = serde_json::to_string(&request)?;
        payload.push('\n');

        let stream = self
            .stream
            .take()
            .ok_or_else(|| anyhow::anyhow!("client stream is not available"))?;
        let (read_half, mut write_half) = stream.into_split();
        write_half
            .write_all(payload.as_bytes())
            .await
            .context("failed to write request")?;

        let mut reader = BufReader::new(read_half);
        let mut line = String::new();
        let n = reader
            .read_line(&mut line)
            .await
            .context("failed to read response")?;
        if n == 0 {
            anyhow::bail!("daemon closed connection before sending response");
        }

        let read_half = reader.into_inner();
        self.stream = Some(read_half.reunite(write_half)?);

        let resp: Value = serde_json::from_str(&line).context("failed to parse response")?;
        if let Some(err) = resp.get("error") {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603);
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error")
                .to_string();
            anyhow::bail!("daemon error {code}: {message}");
        }
        Ok(resp.get("result").cloned().unwrap_or(json!(null)))
    }

    // ===== Session methods =====

    /// Register a new session.
    pub async fn session_register(
        &mut self,
        pid: u32,
        agent_type: &str,
        working_dir: Option<&str>,
        capabilities: Vec<String>,
    ) -> Result<Value> {
        let mut params = json!({
            "pid": pid,
            "agent_type": agent_type,
            "capabilities": capabilities,
        });
        if let Some(dir) = working_dir {
            params["working_dir"] = json!(dir);
        }
        self.call("session.register", params).await
    }

    /// Deregister a session.
    pub async fn session_deregister(&mut self, session_id: &str) -> Result<Value> {
        self.call("session.deregister", json!({ "session_id": session_id }))
            .await
    }

    /// Send a heartbeat.
    pub async fn session_heartbeat(&mut self, session_id: &str) -> Result<Value> {
        self.call("session.heartbeat", json!({ "session_id": session_id }))
            .await
    }

    /// List all sessions.
    pub async fn session_list(&mut self) -> Result<Value> {
        self.call("session.list", json!({})).await
    }

    /// Get a single session.
    pub async fn session_get(&mut self, session_id: &str) -> Result<Value> {
        self.call("session.get", json!({ "session_id": session_id }))
            .await
    }

    // ===== Reservation methods =====

    /// Claim a reservation.
    pub async fn reservation_claim(
        &mut self,
        session_id: &str,
        path: &str,
        mode: &str,
        ttl_sec: u64,
    ) -> Result<Value> {
        self.call(
            "reservation.claim",
            json!({
                "session_id": session_id,
                "path": path,
                "mode": mode,
                "ttl_sec": ttl_sec,
            }),
        )
        .await
    }

    /// Release a reservation.
    pub async fn reservation_release(&mut self, reservation_id: &str) -> Result<Value> {
        self.call(
            "reservation.release",
            json!({ "reservation_id": reservation_id }),
        )
        .await
    }

    /// List reservations.
    pub async fn reservation_list(&mut self, path: Option<&str>) -> Result<Value> {
        let mut params = json!({});
        if let Some(p) = path {
            params["path"] = json!(p);
        }
        self.call("reservation.list", params).await
    }

    // ===== Inbox methods =====

    /// Post a message.
    pub async fn inbox_post(
        &mut self,
        from_session: &str,
        to_session: &str,
        subject: &str,
        body: &str,
        priority: &str,
    ) -> Result<Value> {
        self.call(
            "inbox.post",
            json!({
                "from_session": from_session,
                "to_session": to_session,
                "subject": subject,
                "body": body,
                "priority": priority,
            }),
        )
        .await
    }

    /// List messages for a session.
    pub async fn inbox_list(
        &mut self,
        session_id: &str,
        unread_only: bool,
        limit: u32,
    ) -> Result<Value> {
        self.call(
            "inbox.list",
            json!({
                "session_id": session_id,
                "unread_only": unread_only,
                "limit": limit,
            }),
        )
        .await
    }

    /// Read a single message.
    pub async fn inbox_read(&mut self, message_id: &str) -> Result<Value> {
        self.call("inbox.read", json!({ "message_id": message_id }))
            .await
    }

    // ===== State methods =====

    /// Set live state.
    pub async fn state_set(
        &mut self,
        session_id: &str,
        focus_file: Option<&str>,
        focus_branch: Option<&str>,
        worktree: Option<&str>,
        status: &str,
    ) -> Result<Value> {
        let mut params = json!({
            "session_id": session_id,
            "status": status,
        });
        if let Some(f) = focus_file {
            params["focus_file"] = json!(f);
        }
        if let Some(b) = focus_branch {
            params["focus_branch"] = json!(b);
        }
        if let Some(w) = worktree {
            params["worktree"] = json!(w);
        }
        self.call("state.set", params).await
    }

    /// Get live state.
    pub async fn state_get(&mut self, session_id: &str) -> Result<Value> {
        self.call("state.get", json!({ "session_id": session_id }))
            .await
    }

    // ===== Discovery methods =====

    /// Discover agents.
    pub async fn discover_agents(
        &mut self,
        path: Option<&str>,
        branch: Option<&str>,
        capabilities: Vec<String>,
    ) -> Result<Value> {
        let mut params = json!({
            "capabilities": capabilities,
        });
        if let Some(p) = path {
            params["path"] = json!(p);
        }
        if let Some(b) = branch {
            params["branch"] = json!(b);
        }
        self.call("discover.agents", params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_socket_path_contains_teamcomm() {
        let p = default_socket_path();
        assert!(p.to_string_lossy().contains("teamcomm"));
        assert!(p.ends_with("daemon.sock"));
    }
}
