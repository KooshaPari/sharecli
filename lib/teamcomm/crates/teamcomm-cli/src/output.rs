// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pretty-printing helpers used by every subcommand group.
//!
//! We intentionally don't bring in a strong-typed view of every daemon
//! payload: the CLI is the boundary that materializes the JSON the daemon
//! returned into a human-readable shape. Each `print_*` helper accepts a
//! `serde_json::Value` and either renders it as a `comfy-table` (for
//! tabular lists) or pretty JSON (for single objects and the JSON output
//! mode some users prefer).

use comfy_table::Table;
use serde_json::Value;

/// Render a two-dimensional table of rows with the given header labels.
pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    println!("{table}");
}

/// Render a `serde_json::Value` as pretty-printed JSON on stdout.
pub fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("error: failed to render value as JSON: {e}"),
    }
}

/// Best-effort tabular render of a `Vec<SessionSummary>`.
///
/// `value` is expected to be either a `Vec<_>` of session summary objects
/// or a `{"sessions": [...]}` envelope. Unknown shapes fall back to JSON.
pub fn print_session_list(value: &Value) {
    match extract_array(value) {
        Some(items) => {
            let mut rows = Vec::with_capacity(items.len());
            for item in items {
                rows.push(vec![
                    text(&item["session_id"]),
                    text(&item["agent_type"]),
                    text(&item["pid"]),
                    text(&item["status"]),
                    text(&item["focus_file"]),
                    text(&item["last_heartbeat"]),
                ]);
            }
            print_table(
                vec![
                    "SESSION_ID",
                    "AGENT",
                    "PID",
                    "STATUS",
                    "FOCUS_FILE",
                    "LAST_HEARTBEAT",
                ],
                rows,
            );
        }
        None => print_json(value),
    }
}

/// Best-effort tabular render of a `Vec<Reservation>`.
pub fn print_reservation_list(value: &Value) {
    match extract_array(value) {
        Some(items) => {
            let mut rows = Vec::with_capacity(items.len());
            for item in items {
                rows.push(vec![
                    text(&item["reservation_id"]),
                    text(&item["session_id"]),
                    text(&item["path"]),
                    text(&item["mode"]),
                    text(&item["acquired_at"]),
                    text(&item["expires_at"]),
                ]);
            }
            print_table(
                vec![
                    "RESERVATION_ID",
                    "SESSION_ID",
                    "PATH",
                    "MODE",
                    "ACQUIRED_AT",
                    "EXPIRES_AT",
                ],
                rows,
            );
        }
        None => print_json(value),
    }
}

/// Best-effort tabular render of a `Vec<InboxMessage>`.
pub fn print_inbox_list(value: &Value) {
    match extract_array(value) {
        Some(items) => {
            let mut rows = Vec::with_capacity(items.len());
            for item in items {
                let read_marker = if item["read"].as_bool().unwrap_or(false) {
                    "yes"
                } else {
                    "no"
                };
                rows.push(vec![
                    text(&item["message_id"]),
                    text(&item["from_session"]),
                    text(&item["to_session"]),
                    text(&item["subject"]),
                    text(&item["priority"]),
                    read_marker.to_string(),
                    text(&item["ts"]),
                ]);
            }
            print_table(
                vec![
                    "MESSAGE_ID",
                    "FROM",
                    "TO",
                    "SUBJECT",
                    "PRIORITY",
                    "READ",
                    "TS",
                ],
                rows,
            );
        }
        None => print_json(value),
    }
}

/// Best-effort tabular render of a `LiveState` (single record).
pub fn print_state(value: &Value) {
    if !value.is_object() {
        print_json(value);
        return;
    }
    let rows = vec![
        vec!["session_id".into(), text(&value["session_id"])],
        vec!["focus_file".into(), text(&value["focus_file"])],
        vec!["focus_branch".into(), text(&value["focus_branch"])],
        vec!["worktree".into(), text(&value["worktree"])],
        vec!["status".into(), text(&value["status"])],
        vec!["last_heartbeat".into(), text(&value["last_heartbeat"])],
    ];
    print_table(vec!["FIELD", "VALUE"], rows);
}

/// Pull an array out of either a bare `Vec` or a `{"<key>": [...]}` envelope.
fn extract_array(value: &Value) -> Option<&Vec<Value>> {
    if let Value::Array(arr) = value {
        return Some(arr);
    }
    if let Some(obj) = value.as_object() {
        for (_k, v) in obj {
            if let Value::Array(arr) = v {
                return Some(arr);
            }
        }
    }
    None
}

/// Stringify a `serde_json::Value` for table cell rendering. `Null`,
/// missing fields, and unhandled types render as `-`.
fn text(v: &Value) -> String {
    match v {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

/// Render the M0 placeholder line used when a daemon method is not yet
/// implemented. Returns the placeholder string (does not print).
pub fn m0_placeholder(method: &str) -> String {
    format!(
        "(M0 placeholder) `{method}` is not yet implemented in the daemon — full support lands in M1–M3."
    )
}
