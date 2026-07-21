// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unix-socket connection helpers.
//!
//! All daemon RPCs go over a Unix-domain socket. The default location is
//! `$XDG_RUNTIME_DIR/teamcomm/daemon.sock` (or `/tmp/teamcomm/daemon.sock`
//! if `XDG_RUNTIME_DIR` is not set).

use std::path::PathBuf;

/// Default socket path the CLI connects to when `--socket` is not given.
pub fn default_socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("teamcomm/daemon.sock")
}

/// Open a Unix-stream connection to the daemon, using `socket` if supplied
/// or the default [`default_socket_path`] otherwise.
pub async fn connect(socket: &Option<PathBuf>) -> anyhow::Result<tokio::net::UnixStream> {
    let path = socket.clone().unwrap_or_else(default_socket_path);
    let stream = tokio::net::UnixStream::connect(&path).await?;
    Ok(stream)
}
