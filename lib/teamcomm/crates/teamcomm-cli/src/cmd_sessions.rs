// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm sessions` — list and inspect registered agent sessions.
//!
//! `sessions list` → `session.list` RPC, optionally in `--watch` mode that
//! re-polls every second.
//! `sessions show <id>` → `session.get` RPC for a single record.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::json;

use crate::connect;
use crate::output;
use crate::rpc;

use super::{SessionsCmd, SessionsSub};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: SessionsCmd) -> anyhow::Result<()> {
    match cmd.sub {
        SessionsSub::List { watch, socket } => list(watch, socket).await,
        SessionsSub::Show { session_id, socket } => show(session_id, socket).await,
    }
}

async fn list(watch: bool, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let method = "session.list";

    if !watch {
        return dispatch(method, &socket, json!({})).await;
    }

    // Watch mode: redraw the table on every poll using ANSI clear-screen.
    loop {
        print!("\x1B[2J\x1B[H");
        let _ = dispatch(method, &socket, json!({})).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn show(session_id: String, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    dispatch("session.get", &socket, json!({ "session_id": session_id })).await
}

/// Inner dispatch used by both `list` and `show`. On any error we propagate
/// it via `Err(...)` — sessions are not M0 placeholders, so the user
/// should see the actual failure (e.g. "daemon not running" or a
/// structured daemon error).
async fn dispatch(method: &str, socket: &Path, params: serde_json::Value) -> anyhow::Result<()> {
    match rpc::call_into(&Some(socket.to_path_buf()), method, params).await {
        Ok(Ok(value)) => {
            if method == "session.list" {
                output::print_session_list(&value);
            } else {
                output::print_json(&value);
            }
            Ok(())
        }
        Ok(Err(rpc::RpcCallError::MethodNotFound { message })) => {
            // Sessions are not formally M0 placeholders, but the daemon's
            // M0 only includes session.register/deregister/heartbeat. If
            // `session.list`/`session.get` aren't there yet, surface a
            // friendly error (exit non-zero) so the user knows.
            Err(anyhow::anyhow!(
                "daemon does not implement `{method}` yet ({message})"
            ))
        }
        Ok(Err(e)) => Err(anyhow::anyhow!(e.to_string())),
        Err(e) => Err(e),
    }
}
