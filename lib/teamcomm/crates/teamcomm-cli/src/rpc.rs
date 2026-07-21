// SPDX-License-Identifier: MIT OR Apache-2.0
//! JSON-RPC 2.0 over a Unix-domain socket: write one request line, read one
//! response line, and surface success vs. error to the caller.
//!
//! The transport is intentionally minimal: a single line of JSON terminated
//! by `\n` in each direction. The daemon is expected to write one JSON
//! response (with newline) per request and may close the connection after
//! that or keep it open for further calls.

use std::fmt;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::connect;

/// A structured error from a JSON-RPC call. Carries either a daemon-side
/// error response (code + message + optional data) or a transport-level
/// failure wrapped as `Transport`.
#[derive(Debug)]
pub enum RpcCallError {
    /// Daemon returned a structured JSON-RPC error response.
    Server {
        code: i32,
        message: String,
        /// Original `data` payload from the JSON-RPC error. Kept on the
        /// type for callers that want to introspect structured error info,
        /// even though the current `Display` impl elides it.
        #[allow(dead_code)]
        data: Option<Value>,
    },
    /// Transport-level failure (could not connect, write, or read).
    Transport(String),
    /// Sentinel for "method not found" (-32601). Callers can branch on
    /// this to print the M0 placeholder message instead of a hard error.
    MethodNotFound { message: String },
}

impl fmt::Display for RpcCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcCallError::Server { code, message, .. } => {
                write!(f, "daemon returned error {code}: {message}")
            }
            RpcCallError::Transport(s) => write!(f, "transport error: {s}"),
            RpcCallError::MethodNotFound { message } => write!(f, "method not found: {message}"),
        }
    }
}

impl std::error::Error for RpcCallError {}

/// Outcome of an RPC call: either a successful result payload, or a
/// structured error from the daemon.
pub type RpcResult = Result<Value, RpcCallError>;

/// Send a JSON-RPC 2.0 request to the daemon and read the matching response.
///
/// `id` is auto-assigned to `1`. The wire envelope matches the spec in
/// `teamcomm-protocol::rpc`. This is the convenience entry point — the
/// `call_into` variant returns the structured `RpcResult` so callers can
/// branch on `MethodNotFound` to emit the M0 placeholder message.
#[allow(dead_code)]
pub async fn call(socket: &Option<PathBuf>, method: &str, params: Value) -> anyhow::Result<Value> {
    let result = call_into(socket, method, params).await?;
    result.map_err(anyhow::Error::from)
}

/// Same as [`call`] but returns the structured [`RpcResult`] so callers
/// can branch on `MethodNotFound` and emit the M0 placeholder message.
pub async fn call_into(
    socket: &Option<PathBuf>,
    method: &str,
    params: Value,
) -> anyhow::Result<RpcResult> {
    let stream = connect::connect(socket)
        .await
        .map_err(|e| RpcCallError::Transport(format!("connect: {e}")))?;
    Ok(call_on_stream(stream, method, params).await)
}

async fn call_on_stream(stream: UnixStream, method: &str, params: Value) -> RpcResult {
    let request = Request {
        jsonrpc: "2.0",
        id: 1,
        method,
        params: &params,
    };
    let mut payload = serde_json::to_string(&request).expect("request is always serializable");
    payload.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    if let Err(e) = write_half.write_all(payload.as_bytes()).await {
        return Err(RpcCallError::Transport(format!("write: {e}")));
    }

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    let n = match reader.read_line(&mut line).await {
        Ok(n) => n,
        Err(e) => return Err(RpcCallError::Transport(format!("read: {e}"))),
    };
    if n == 0 {
        return Err(RpcCallError::Server {
            code: -32099,
            message: "daemon closed connection before sending a response".into(),
            data: None,
        });
    }
    let line = line.trim_end_matches(['\n', '\r']);

    // Try success-response first, then error-response.
    if let Ok(resp) = serde_json::from_str::<Response>(line) {
        return Ok(resp.result);
    }
    if let Ok(err) = serde_json::from_str::<ErrorResponse>(line) {
        let code = err.error.code;
        let message = err.error.message.clone();
        if code == -32601 {
            return Err(RpcCallError::MethodNotFound { message });
        }
        return Err(RpcCallError::Server {
            code,
            message,
            data: err.error.data,
        });
    }
    Err(RpcCallError::Server {
        code: -32700,
        message: format!(
            "daemon response was not a valid JSON-RPC envelope: {}",
            truncate(line, 200)
        ),
        data: None,
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

// ---- Wire envelope structs (local; mirror `teamcomm-protocol::rpc` but
// keep the field names we need in this crate and avoid a deep import).
// We intentionally use `i64` for the id to keep the wire shape trivially
// round-trippable from the protocol's `RpcId::Number` variant.

#[derive(serde::Serialize)]
struct Request<'a> {
    jsonrpc: &'static str,
    id: i64,
    method: &'a str,
    params: &'a Value,
}

#[derive(Deserialize)]
struct Response {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: serde_json::Value,
    result: Value,
}

#[derive(Deserialize)]
struct ErrorResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: serde_json::Value,
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    code: i32,
    message: String,
    data: Option<Value>,
}
