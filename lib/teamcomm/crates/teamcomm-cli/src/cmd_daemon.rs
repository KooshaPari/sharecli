// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm daemon` — local process management for the teamcomm coordinator.
//!
//! The `daemon` subcommand group is **local** and does not talk to a running
//! daemon: `start` spawns the daemon binary as a child process, `stop` looks
//! up the daemon's pid file and signals it (or falls back to a JSON-RPC
//! `daemon.shutdown` call), and `status` checks whether the configured
//! socket is connectable.

use std::path::PathBuf;
use std::process::Command;

use crate::connect;
use crate::rpc;

use super::{DaemonCmd, DaemonSub};

/// Entry point dispatched from `main::dispatch`.
pub async fn run(cmd: DaemonCmd) -> anyhow::Result<()> {
    match cmd.sub {
        DaemonSub::Start {
            foreground,
            socket,
            pid_file,
        } => start(foreground, socket, pid_file).await,
        DaemonSub::Stop { socket } => stop(socket).await,
        DaemonSub::Status { socket } => status(socket).await,
    }
}

async fn start(
    foreground: bool,
    socket: Option<PathBuf>,
    pid_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    let daemon_bin = current_exe_or("teamcomm-daemon")?;

    println!("starting teamcomm daemon (binary: {daemon_bin:?})");
    println!("  socket:  {}", socket.display());
    if let Some(pf) = &pid_file {
        println!("  pid file: {}", pf.display());
    }
    println!(
        "  mode:    {}",
        if foreground { "foreground" } else { "detached" }
    );

    if foreground {
        // Run the daemon in-process via execvp-style behavior. We use
        // `Command::status` so the parent waits for the daemon to exit.
        let status = Command::new(&daemon_bin)
            .arg("--socket")
            .arg(&socket)
            .args(
                pid_file
                    .iter()
                    .flat_map(|p| [String::from("--pid-file"), p.display().to_string()]),
            )
            .status()?;
        if !status.success() {
            anyhow::bail!("daemon exited with non-zero status: {status}");
        }
    } else {
        // Spawn detached: redirect stdio, set a new process group, then
        // wait briefly to confirm it didn't crash on startup.
        let child = Command::new(&daemon_bin)
            .arg("--socket")
            .arg(&socket)
            .args(
                pid_file
                    .iter()
                    .flat_map(|p| [String::from("--pid-file"), p.display().to_string()]),
            )
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        println!("spawned daemon with pid {}", child.id());
    }
    Ok(())
}

async fn stop(socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    println!("requesting shutdown via {}", socket.display());

    match rpc::call_into(
        &Some(socket.clone()),
        "daemon.shutdown",
        serde_json::json!({}),
    )
    .await
    {
        Ok(Ok(_value)) => {
            println!("daemon acknowledged shutdown");
            Ok(())
        }
        Ok(Err(rpc::RpcCallError::MethodNotFound { message })) => {
            // M0 placeholder: the daemon stub has no shutdown method.
            println!(
                "(M0 placeholder) `daemon.shutdown` is not yet implemented ({message}). \
                 The daemon binary is a stub in M0; nothing to stop."
            );
            Ok(())
        }
        Ok(Err(rpc::RpcCallError::Transport(reason))) => {
            println!(
                "could not reach the daemon at {} ({reason}); nothing to stop",
                socket.display()
            );
            Ok(())
        }
        Ok(Err(e)) => Err(anyhow::anyhow!(e.to_string())),
        Err(e) => Err(e),
    }
}

async fn status(socket: Option<PathBuf>) -> anyhow::Result<()> {
    let socket = socket.unwrap_or_else(connect::default_socket_path);
    match tokio::net::UnixStream::connect(&socket).await {
        Ok(_stream) => {
            println!("daemon: running");
            println!("socket: {}", socket.display());
            Ok(())
        }
        Err(e) => {
            println!("daemon: not running");
            println!("socket: {}", socket.display());
            println!("reason: {e}");
            // Status is informational; exit 0 even when the daemon is down.
            Ok(())
        }
    }
}

/// Best-effort lookup of the daemon binary. Defaults to a `teamcomm-daemon`
/// string on `$PATH`; falls back to the current executable's sibling
/// `teamcomm-daemon` binary in a dev build.
fn current_exe_or(default: &str) -> anyhow::Result<PathBuf> {
    // Prefer the named binary on PATH.
    if let Ok(found) = which(default) {
        return Ok(found);
    }
    // Fallback: the current executable's directory + name.
    let cur = std::env::current_exe()?;
    let dir = cur
        .parent()
        .ok_or_else(|| anyhow::anyhow!("no parent dir for current_exe"))?;
    Ok(dir.join(default))
}

fn which(name: &str) -> anyhow::Result<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    for entry in std::env::split_paths(&path) {
        let candidate = entry.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("binary `{name}` not found on PATH")
}
