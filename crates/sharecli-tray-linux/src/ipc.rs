//! Blocking Unix-socket NDJSON-RPC client for the sharecli IPC server.
//!
//! Mirrors the wire contract in `crates/sharecli-ipc/src/handler.rs` and the
//! macOS Swift `IPCClient`. Each call opens its own connection — the Rust IPC
//! server handles concurrent connections, so no shared state is kept here.
//!
//! The RPC functions are only wired into the tray on Linux (the `ksni` binary
//! target), but the module compiles everywhere so its wire types and
//! `socket_path` stay unit-testable cross-platform.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Request<'a> {
    id: u64,
    method: &'a str,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct Response<T> {
    #[allow(dead_code)]
    id: u64,
    result: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessSummary {
    pub pid: u32,
    pub name: String,
    #[allow(dead_code)]
    pub cmd: Vec<String>,
    pub memory_mb: u64,
    pub project: Option<String>,
    pub harness: Option<String>,
    #[allow(dead_code)]
    pub start_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthSnapshot {
    pub managed_processes: usize,
    pub used_memory_mb: u64,
    pub total_memory_mb: u64,
    pub healthy: bool,
}

/// Resolve the IPC socket path, honoring `SHARECLI_IPC_SOCK` and falling back to
/// `$XDG_DATA_HOME/sharecli/ipc.sock` (matching `sharecli-ipc::socket_path`).
pub fn socket_path() -> PathBuf {
    if let Ok(v) = std::env::var("SHARECLI_IPC_SOCK") {
        return PathBuf::from(v);
    }
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("sharecli").join("ipc.sock")
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn call<T: for<'de> Deserialize<'de>>(method: &str, params: serde_json::Value) -> Result<T> {
    let path = socket_path();
    let stream = UnixStream::connect(&path)
        .with_context(|| format!("connect to sharecli IPC socket at {}", path.display()))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut writer = stream.try_clone()?;
    let payload = serde_json::to_string(&Request { id, method, params })?;
    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.trim().is_empty() {
        return Err(anyhow!("empty response from IPC server"));
    }

    let resp: Response<T> = serde_json::from_str(line.trim())
        .with_context(|| format!("decode IPC response for {method}"))?;
    if let Some(err) = resp.error {
        return Err(anyhow!("IPC error ({method}): {err}"));
    }
    resp.result.ok_or_else(|| anyhow!("IPC response for {method} had no result"))
}

pub fn list_processes() -> Result<Vec<ProcessSummary>> {
    call("process.list", serde_json::json!({}))
}

pub fn health() -> Result<HealthSnapshot> {
    call("health.status", serde_json::json!({}))
}

pub fn kill(pid: u32) -> Result<bool> {
    call("process.kill", serde_json::json!({ "pid": pid }))
}

pub fn kill_all() -> Result<bool> {
    call("process.kill_all", serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_honors_env_override() {
        std::env::set_var("SHARECLI_IPC_SOCK", "/tmp/custom-sharecli.sock");
        assert_eq!(socket_path(), PathBuf::from("/tmp/custom-sharecli.sock"));
        std::env::remove_var("SHARECLI_IPC_SOCK");
    }

    #[test]
    fn socket_path_default_ends_with_ipc_sock() {
        std::env::remove_var("SHARECLI_IPC_SOCK");
        let p = socket_path();
        assert!(p.ends_with("sharecli/ipc.sock"), "unexpected default path: {}", p.display());
    }

    #[test]
    fn process_summary_matches_server_wire_shape() {
        // Byte-for-byte the JSON emitted by sharecli-ipc::handler::ProcessSummary.
        let raw = r#"{"pid":4242,"name":"claude","cmd":["claude","--foo"],
            "memory_mb":128,"project":"omniroute","harness":"claude","start_time":17}"#;
        let p: ProcessSummary = serde_json::from_str(raw).unwrap();
        assert_eq!(p.pid, 4242);
        assert_eq!(p.name, "claude");
        assert_eq!(p.memory_mb, 128);
        assert_eq!(p.project.as_deref(), Some("omniroute"));
        assert_eq!(p.harness.as_deref(), Some("claude"));
    }

    #[test]
    fn process_summary_allows_null_optionals() {
        let raw = r#"{"pid":1,"name":"node","cmd":[],"memory_mb":0,
            "project":null,"harness":null,"start_time":0}"#;
        let p: ProcessSummary = serde_json::from_str(raw).unwrap();
        assert!(p.project.is_none());
        assert!(p.harness.is_none());
    }

    #[test]
    fn health_snapshot_matches_server_wire_shape() {
        let raw = r#"{"managed_processes":3,"used_memory_mb":2048,
            "total_memory_mb":16384,"healthy":true}"#;
        let h: HealthSnapshot = serde_json::from_str(raw).unwrap();
        assert_eq!(h.managed_processes, 3);
        assert_eq!(h.used_memory_mb, 2048);
        assert_eq!(h.total_memory_mb, 16384);
        assert!(h.healthy);
    }

    #[test]
    fn response_surfaces_server_error() {
        let raw = r#"{"id":7,"result":null,"error":"boom"}"#;
        let resp: Response<bool> = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.error.as_deref(), Some("boom"));
        assert!(resp.result.is_none());
    }
}
