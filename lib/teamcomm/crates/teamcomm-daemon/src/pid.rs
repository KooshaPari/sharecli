// SPDX-License-Identifier: MIT OR Apache-2.0
//! PID file management: write/read/remove the daemon's PID file with
//! "refuse if a daemon is already running" semantics.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use tracing::{info, warn};

/// Write `pid` to `path`.
///
/// Behaviour:
///
/// 1. If `path` does not exist: write the pid and return.
/// 2. If `path` exists and the recorded pid is still running: return an
///    error (a daemon is already up).
/// 3. If `path` exists and the recorded pid is stale: warn, remove the
///    stale file, write the new pid.
pub fn write_pid_file(path: &Path, pid: u32) -> Result<()> {
    if let Some(existing) = read_pid_file(path)? {
        if is_pid_running(existing) {
            return Err(anyhow!(
                "pid file {} already exists and pid {existing} is running",
                path.display()
            ));
        }
        warn!(
            pid = existing,
            path = %path.display(),
            "removing stale pid file"
        );
        let _ = fs::remove_file(path);
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create pid file parent {}", parent.display())
            })?;
        }
    }
    fs::write(path, pid.to_string())
        .with_context(|| format!("failed to write pid file {}", path.display()))?;
    info!(pid, path = %path.display(), "wrote pid file");
    Ok(())
}

/// Read the pid stored in `path`.
///
/// Returns `Ok(None)` if the file does not exist. Returns an error only
/// for genuine I/O / parse failures (permission denied, non-numeric
/// content, ...).
pub fn read_pid_file(path: &Path) -> Result<Option<u32>> {
    match fs::read_to_string(path) {
        Ok(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                let pid: u32 = trimmed
                    .parse()
                    .with_context(|| format!("non-numeric pid in {}", path.display()))?;
                Ok(Some(pid))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("failed to read pid file {}", path.display())),
    }
}

/// Best-effort removal of `path`. Missing file is not an error.
pub fn remove_pid_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => {
            info!(path = %path.display(), "removed pid file");
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("failed to remove pid file {}", path.display())),
    }
}

/// Returns `true` if `pid` currently names a running process.
///
/// Implementation: shells out to `kill -0 <pid>`. `kill -0` succeeds iff
/// the pid exists *and* the process is not a zombie we lack permission
/// to signal. Sufficient for M0 — no new dep on `nix` / `sysinfo`.
///
/// Public so the CLI's `start`/`stop`/`status` subcommands can probe
/// the pid file for liveness.
pub fn is_pid_running(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("daemon.pid");
        write_pid_file(&path, 1234).unwrap();
        assert_eq!(read_pid_file(&path).unwrap(), Some(1234));
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.pid");
        assert_eq!(read_pid_file(&path).unwrap(), None);
    }

    #[test]
    fn write_refuses_when_live_pid_present() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("daemon.pid");
        // Use our own pid — we are definitely running.
        let my_pid = std::process::id();
        write_pid_file(&path, my_pid).unwrap();
        let err = write_pid_file(&path, 9999).unwrap_err().to_string();
        assert!(err.contains("already exists"), "got: {err}");
        assert!(err.contains(&my_pid.to_string()), "got: {err}");
    }

    #[test]
    fn write_overwrites_stale_pid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("daemon.pid");
        // 0xffffffff is virtually guaranteed to not be a live pid; the kill
        // -0 probe will fail and the writer should treat the file as stale.
        write_pid_file(&path, 0xffff_ffff).unwrap();
        write_pid_file(&path, 1234).unwrap();
        assert_eq!(read_pid_file(&path).unwrap(), Some(1234));
    }

    #[test]
    fn remove_pid_file_missing_is_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.pid");
        assert!(remove_pid_file(&path).is_ok());
    }

    #[test]
    fn remove_pid_file_removes_existing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("daemon.pid");
        write_pid_file(&path, 4242).unwrap();
        remove_pid_file(&path).unwrap();
        assert_eq!(read_pid_file(&path).unwrap(), None);
    }
}
