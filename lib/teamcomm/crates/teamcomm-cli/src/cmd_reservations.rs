// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm reservations` — file/path reservation commands.
//!
//! These are M0 placeholders: the daemon's M0 only supports
//! session.register/deregister/heartbeat. If the reservation methods
//! return `-32601 Method not found` (or the daemon isn't running at all),
//! the CLI prints a friendly placeholder line and exits 0.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::connect;
use crate::output;
use crate::rpc;

use super::{ModeArg, ReservationsCmd, ReservationsSub};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: ReservationsCmd) -> anyhow::Result<()> {
    match cmd.sub {
        ReservationsSub::Ls { path, socket } => ls(path, socket).await,
        ReservationsSub::Claim {
            path,
            session,
            mode,
            ttl_sec,
            socket,
        } => claim(path, session, mode, ttl_sec, socket).await,
        ReservationsSub::Release {
            reservation_id,
            socket,
        } => release(reservation_id, socket).await,
    }
}

async fn ls(path: Option<PathBuf>, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({ "path": path });
    placeholder_or("reservation.list", &socket, params, |v| {
        output::print_reservation_list(v);
    })
    .await
}

async fn claim(
    path: PathBuf,
    session: String,
    mode: ModeArg,
    ttl_sec: u64,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let mode_str = match mode {
        ModeArg::Read => "read",
        ModeArg::Write => "write",
        ModeArg::Exclusive => "exclusive",
    };
    let params = json!({
        "session": session,
        "path": path,
        "mode": mode_str,
        "ttl_sec": ttl_sec,
    });
    placeholder_or("reservation.claim", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}

async fn release(reservation_id: String, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({ "reservation_id": reservation_id });
    placeholder_or("reservation.release", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}

/// M0-placeholder pattern: try the RPC, on any error print a clear message
/// and return Ok. The `on_success` closure is responsible for printing the
/// successful result. Shared with the other M0 placeholder subcommand
/// groups (inbox, state, discover) via `pub(super)` visibility.
pub(super) async fn placeholder_or(
    method: &str,
    socket: &Path,
    params: serde_json::Value,
    on_success: impl FnOnce(&serde_json::Value),
) -> anyhow::Result<()> {
    match rpc::call_into(&Some(socket.to_path_buf()), method, params).await {
        Ok(Ok(value)) => {
            on_success(&value);
            Ok(())
        }
        Ok(Err(rpc::RpcCallError::MethodNotFound { message })) => {
            println!("{}", output::m0_placeholder(method));
            eprintln!("hint: {message}");
            Ok(())
        }
        Ok(Err(rpc::RpcCallError::Transport(reason))) => {
            println!("{}", output::m0_placeholder(method));
            eprintln!(
                "hint: daemon is not reachable at {} ({reason})",
                socket.display()
            );
            Ok(())
        }
        Ok(Err(e)) => {
            println!("{}", output::m0_placeholder(method));
            eprintln!("hint: {}", e);
            Ok(())
        }
        Err(e) => {
            println!("{}", output::m0_placeholder(method));
            eprintln!("hint: {e}");
            Ok(())
        }
    }
}
