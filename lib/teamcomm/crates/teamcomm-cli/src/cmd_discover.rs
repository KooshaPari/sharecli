// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm discover` — discovery queries (who is working on what).
//!
//! M0 placeholder: full support lands in M1–M3.

use std::path::PathBuf;

use serde_json::json;

use crate::cmd_reservations::placeholder_or;
use crate::connect;
use crate::output;

use super::{DiscoverCmd, DiscoverSub};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: DiscoverCmd) -> anyhow::Result<()> {
    match cmd.sub {
        DiscoverSub::Agents {
            path,
            branch,
            capability,
            socket,
        } => agents(path, branch, capability, socket).await,
    }
}

async fn agents(
    path: Option<PathBuf>,
    branch: Option<String>,
    capability: Vec<String>,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({
        "path": path,
        "branch": branch,
        "capabilities": capability,
    });
    placeholder_or("discover.agents", &socket, params, |v| {
        // Discovery results are `Vec<SessionSummary>` envelopes.
        output::print_session_list(v);
    })
    .await
}
