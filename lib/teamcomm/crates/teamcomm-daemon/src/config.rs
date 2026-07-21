// SPDX-License-Identifier: MIT OR Apache-2.0
//! Daemon configuration: socket/pid paths, heartbeat timeouts, log level.

use std::path::{Path, PathBuf};

/// Static configuration for a running daemon instance.
///
/// All paths are resolved at construction time (via [`DaemonConfig::from_args`]
/// or [`DaemonConfig::default_paths`]); the listener does no further
/// path manipulation at runtime.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Path of the Unix-domain socket the daemon listens on.
    pub socket_path: PathBuf,
    /// Path of the file the daemon writes its PID to on startup.
    pub pid_file_path: PathBuf,
    /// A session is considered lost after this many seconds without a
    /// heartbeat. M0 does not actually sweep lost sessions; M1 will.
    /// Default: 90 (3 missed 30-second heartbeats).
    pub heartbeat_timeout_sec: u64,
    /// Tracing/log filter string (e.g. `"info"`, `"teamcomm_daemon=debug"`).
    /// Default: `"info"`.
    pub log_level: String,
}

impl DaemonConfig {
    /// Build a [`DaemonConfig`] from explicit optional paths, falling back
    /// to [`default_paths`] for any field that is `None`.
    pub fn from_args(socket: Option<PathBuf>, pid_file: Option<PathBuf>) -> Self {
        let (def_sock, def_pid) = default_paths();
        Self {
            socket_path: socket.unwrap_or(def_sock),
            pid_file_path: pid_file.unwrap_or(def_pid),
            heartbeat_timeout_sec: 90,
            log_level: "info".to_string(),
        }
    }

    /// Build a [`DaemonConfig`] with the default socket/pid paths and
    /// custom timeouts / log level.
    pub fn with_overrides(
        socket: Option<PathBuf>,
        pid_file: Option<PathBuf>,
        heartbeat_timeout_sec: u64,
        log_level: String,
    ) -> Self {
        let mut cfg = Self::from_args(socket, pid_file);
        cfg.heartbeat_timeout_sec = heartbeat_timeout_sec;
        cfg.log_level = log_level;
        cfg
    }

    /// Helper: derive a parent directory from the socket path. Used by
    /// the listener when it needs to `mkdir -p` the runtime dir.
    pub fn socket_parent(&self) -> &Path {
        self.socket_path
            .parent()
            .unwrap_or_else(|| Path::new("/tmp"))
    }
}

/// Default socket and pid file paths.
///
/// Uses [`dirs::runtime_dir`] (which maps to `$XDG_RUNTIME_DIR` on Linux
/// and `$TMPDIR` on macOS) and falls back to `/tmp` when no runtime dir
/// is available.
pub fn default_paths() -> (PathBuf, PathBuf) {
    let runtime = dirs::runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    (
        runtime.join("teamcomm").join("daemon.sock"),
        runtime.join("teamcomm").join("daemon.pid"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_args_uses_defaults_when_none() {
        let cfg = DaemonConfig::from_args(None, None);
        let (def_sock, def_pid) = default_paths();
        assert_eq!(cfg.socket_path, def_sock);
        assert_eq!(cfg.pid_file_path, def_pid);
        assert_eq!(cfg.heartbeat_timeout_sec, 90);
        assert_eq!(cfg.log_level, "info");
    }

    #[test]
    fn from_args_respects_overrides() {
        let sock = PathBuf::from("/tmp/custom.sock");
        let pid = PathBuf::from("/tmp/custom.pid");
        let cfg = DaemonConfig::from_args(Some(sock.clone()), Some(pid.clone()));
        assert_eq!(cfg.socket_path, sock);
        assert_eq!(cfg.pid_file_path, pid);
    }

    #[test]
    fn with_overrides_replaces_timeouts_and_log_level() {
        let cfg = DaemonConfig::with_overrides(None, None, 30, "debug".into());
        assert_eq!(cfg.heartbeat_timeout_sec, 30);
        assert_eq!(cfg.log_level, "debug");
    }

    #[test]
    fn socket_parent_strips_filename() {
        let cfg = DaemonConfig::from_args(
            Some(PathBuf::from("/a/b/c.sock")),
            Some(PathBuf::from("/a/b/c.pid")),
        );
        assert_eq!(cfg.socket_parent(), Path::new("/a/b"));
    }

    #[test]
    fn default_paths_contain_teamcomm_dir() {
        let (sock, pid) = default_paths();
        assert!(sock.ends_with("daemon.sock"), "got {sock:?}");
        assert!(pid.ends_with("daemon.pid"), "got {pid:?}");
        // Both paths share the same parent dir in M0.
        assert_eq!(sock.parent(), pid.parent());
    }
}
