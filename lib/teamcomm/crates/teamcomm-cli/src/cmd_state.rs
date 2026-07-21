// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm state` — live agent state (focus file, branch, status) commands.
//!
//! M0 placeholder: full support lands in M1–M3.

use std::path::PathBuf;

use serde_json::json;

use crate::cmd_reservations::placeholder_or;
use crate::connect;
use crate::output;

use super::{StateCmd, StateSub};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: StateCmd) -> anyhow::Result<()> {
    match cmd.sub {
        StateSub::Show { session, socket } => show(session, socket).await,
        StateSub::SetFocus {
            file,
            session,
            socket,
        } => set_focus(file, session, socket).await,
        StateSub::SetStatus {
            status,
            session,
            socket,
        } => set_status(status, session, socket).await,
    }
}

async fn show(session: Option<String>, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({ "session": session });
    placeholder_or("state.get", &socket, params, |v| {
        output::print_state(v);
    })
    .await
}

async fn set_focus(
    file: PathBuf,
    session: Option<String>,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({
        "file": file,
        "session": session,
    });
    placeholder_or("state.set_focus", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}

async fn set_status(
    status: String,
    session: Option<String>,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({
        "status": status,
        "session": session,
    });
    placeholder_or("state.set_status", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}
