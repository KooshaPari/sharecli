//! Append-only JSONL audit log for security-relevant serve events.
//!
//! Default path: `$XDG_STATE_HOME/sharecli/audit.jsonl` or
//! `~/.local/state/sharecli/audit.jsonl` (Windows: `%LOCALAPPDATA%/sharecli/audit.jsonl`).
//! Override with `SHARECLI_AUDIT_LOG`.
//!
//! Failures to write are logged via `tracing::warn` and never fail the request
//! path (best-effort audit).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tracing::warn;

static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Emit one audit event (`event` + arbitrary JSON fields).
pub fn emit(event: &str, fields: Value) {
    emit_to(&audit_log_path(), event, fields);
}

/// Emit to an explicit path (tests + callers that already resolved the path).
pub fn emit_to(path: &Path, event: &str, fields: Value) {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!(error = %e, path = %path.display(), "audit log: create_dir_all failed");
            return;
        }
    }

    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);

    let mut record = json!({
        "ts": ts,
        "event": event,
        "service": "sharecli",
    });
    if let (Some(obj), Some(fields_obj)) = (record.as_object_mut(), fields.as_object()) {
        for (k, v) in fields_obj {
            obj.insert(k.clone(), v.clone());
        }
    }

    let line = match serde_json::to_string(&record) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "audit log: serialize failed");
            return;
        }
    };

    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{line}") {
                warn!(error = %e, path = %path.display(), "audit log: write failed");
            }
        }
        Err(e) => {
            warn!(error = %e, path = %path.display(), "audit log: open failed");
        }
    }
}

/// Resolve the audit log file path.
pub fn audit_log_path() -> PathBuf {
    if let Ok(p) = std::env::var("SHARECLI_AUDIT_LOG") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    state_dir().join("audit.jsonl")
}

fn state_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("sharecli");
        }
    }
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local).join("sharecli");
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("state")
        .join("sharecli")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn emit_appends_json_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        emit_to(&path, "test_event", json!({ "k": 1 }));
        emit_to(&path, "test_event", json!({ "k": 2 }));
        let body = fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"event\":\"test_event\""));
        assert!(lines[0].contains("\"k\":1"));
    }

    #[test]
    fn path_respects_env_override() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.jsonl");
        let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("SHARECLI_AUDIT_LOG", &path);
        }
        assert_eq!(audit_log_path(), path);
        unsafe {
            std::env::remove_var("SHARECLI_AUDIT_LOG");
        }
    }
}
