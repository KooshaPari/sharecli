// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm inbox` — inter-agent messaging commands.
//!
//! M0 placeholder: full support lands in M1–M3.

use std::path::PathBuf;

use serde_json::json;

use crate::cmd_reservations::placeholder_or;
use crate::connect;
use crate::output;

use super::{InboxCmd, InboxSub, PriorityArg};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: InboxCmd) -> anyhow::Result<()> {
    match cmd.sub {
        InboxSub::List {
            session,
            unread,
            limit,
            socket,
        } => list(session, unread, limit, socket).await,
        InboxSub::Read { message_id, socket } => read(message_id, socket).await,
        InboxSub::Post {
            from_session,
            to_session,
            subject,
            body,
            priority,
            socket,
        } => post(from_session, to_session, subject, body, priority, socket).await,
    }
}

async fn list(
    session: String,
    unread: bool,
    limit: u32,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({
        "session": session,
        "unread_only": unread,
        "limit": limit,
    });
    placeholder_or("inbox.list", &socket, params, |v| {
        output::print_inbox_list(v);
    })
    .await
}

async fn read(message_id: String, socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let params = json!({ "message_id": message_id });
    placeholder_or("inbox.read", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}

async fn post(
    from_session: String,
    to_session: String,
    subject: String,
    body: String,
    priority: PriorityArg,
    socket: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let priority_str = match priority {
        PriorityArg::Low => "low",
        PriorityArg::Normal => "normal",
        PriorityArg::High => "high",
    };
    let params = json!({
        "from_session": from_session,
        "to_session": to_session,
        "subject": subject,
        "body": body,
        "priority": priority_str,
    });
    placeholder_or("inbox.post", &socket, params, |v| {
        output::print_json(v);
    })
    .await
}
