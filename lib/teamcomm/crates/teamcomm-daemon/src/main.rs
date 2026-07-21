// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm-daemon` CLI.
//!
//! Subcommands:
//! - `start [--foreground] [--socket-path <path>] [--pid-file <path>]`
//! - `stop [--socket-path <path>]`
//! - `status [--socket-path <path>]`
//!
//! The default `start` mode spawns a child process running the same
//! binary in `--foreground` mode and prints the resulting pid + socket
//! path. Use `--foreground` to run the listener in the current
//! process (the usual choice for systemd / launchd supervisors and
//! for development).

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use teamcomm_daemon::{default_paths, Daemon, DaemonConfig};

#[derive(Parser, Debug)]
#[command(
    name = "teamcomm-daemon",
    about = "teamcomm — long-running multi-agent coordinator (M0)",
    long_about = "Long-running daemon that brokers sessions, file reservations, inbox, \
                  and live state for multi-agent coding workflows. Speaks newline-delimited \
                  JSON-RPC 2.0 over a Unix-domain socket.",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Start the daemon. Defaults to forking into the background; pass
    /// `--foreground` to run in the current process.
    Start {
        /// Run in the foreground (do not fork).
        #[arg(long)]
        foreground: bool,
        /// Override the Unix-domain socket path the daemon should listen on.
        #[arg(long)]
        socket_path: Option<PathBuf>,
        /// Override the pid file the daemon should write to.
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
    /// Stop a running daemon by reading its pid file and sending SIGTERM.
    Stop {
        #[arg(long)]
        socket_path: Option<PathBuf>,
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
    /// Report daemon status (stopped / running / stale).
    Status {
        #[arg(long)]
        socket_path: Option<PathBuf>,
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Initialise tracing as early as possible so the fork and signal
    // paths can log usefully.
    init_tracing();

    match cli.cmd {
        Cmd::Start {
            foreground,
            socket_path,
            pid_file,
        } => cmd_start(foreground, socket_path, pid_file).await,
        Cmd::Stop {
            socket_path,
            pid_file,
        } => cmd_stop(socket_path, pid_file).await,
        Cmd::Status {
            socket_path,
            pid_file,
        } => cmd_status(socket_path, pid_file).await,
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false).with_writer(std::io::stderr))
        .try_init();
}

async fn cmd_start(
    foreground: bool,
    socket_path: Option<PathBuf>,
    pid_file: Option<PathBuf>,
) -> Result<()> {
    if foreground {
        start_foreground(socket_path, pid_file).await
    } else {
        start_detached(socket_path, pid_file)
    }
}

async fn start_foreground(socket_path: Option<PathBuf>, pid_file: Option<PathBuf>) -> Result<()> {
    let config = DaemonConfig::from_args(socket_path, pid_file);
    // Refuse to start if a daemon is already up.
    teamcomm_daemon::pid::write_pid_file(&config.pid_file_path, std::process::id())?;
    info!(
        socket = %config.socket_path.display(),
        pid_file = %config.pid_file_path.display(),
        "teamcomm-daemon starting in foreground"
    );
    let daemon = Daemon::new(config.clone());

    let result = daemon.run_foreground().await;

    // Best-effort cleanup of the pid file on the way out.
    if let Err(e) = teamcomm_daemon::pid::remove_pid_file(&config.pid_file_path) {
        warn!(error = %e, "failed to remove pid file on shutdown");
    }
    result
}

/// Re-exec the current binary with `--foreground` and the same
/// socket/pid paths, redirecting stdio to /dev/null. The parent
/// process prints the child's pid and exits.
fn start_detached(socket_path: Option<PathBuf>, pid_file: Option<PathBuf>) -> Result<()> {
    let exe = std::env::current_exe().context("failed to resolve current_exe")?;
    let (def_sock, def_pid) = default_paths();
    let socket = socket_path.unwrap_or(def_sock);
    let pid = pid_file.unwrap_or(def_pid);

    // Make sure we are not already running before forking.
    if let Some(existing) = teamcomm_daemon::pid::read_pid_file(&pid)? {
        if teamcomm_daemon::pid::is_pid_running(existing) {
            bail!("daemon already running (pid {existing}); refusing to start a second instance");
        }
    }

    let child = Command::new(&exe)
        .arg("start")
        .arg("--foreground")
        .arg("--socket-path")
        .arg(&socket)
        .arg("--pid-file")
        .arg(&pid)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to spawn daemon child from {}", exe.display()))?;

    let child_pid = child.id();
    println!(
        "daemon started, pid {child_pid}, socket {}",
        socket.display()
    );
    // Parent exits immediately; the child continues running the
    // listener loop.
    Ok(())
}

async fn cmd_stop(socket_path: Option<PathBuf>, pid_file: Option<PathBuf>) -> Result<()> {
    let (def_sock, def_pid) = default_paths();
    let _ = socket_path.unwrap_or(def_sock); // accepted for symmetry; not used in M0
    let pid_file = pid_file.unwrap_or(def_pid);

    let pid = match teamcomm_daemon::pid::read_pid_file(&pid_file)? {
        Some(p) => p,
        None => {
            println!("daemon not running (no pid file at {})", pid_file.display());
            return Ok(());
        }
    };

    if !teamcomm_daemon::pid::is_pid_running(pid) {
        println!("daemon not running (stale pid file for pid {pid})");
        teamcomm_daemon::pid::remove_pid_file(&pid_file)?;
        return Ok(());
    }

    // Send SIGTERM via `kill`. The child listener is the one that
    // installed the signal handler; this just kicks it.
    let status = Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .with_context(|| format!("failed to send SIGTERM to pid {pid}"))?;
    if !status.success() {
        bail!("kill -TERM {pid} exited with {status:?}");
    }

    // Give the child up to 3 seconds to exit and clean up its pid
    // file; then remove the file ourselves.
    for _ in 0..30 {
        if !teamcomm_daemon::pid::is_pid_running(pid) {
            teamcomm_daemon::pid::remove_pid_file(&pid_file)?;
            println!("daemon stopped (pid {pid})");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    warn!(
        pid,
        "daemon did not exit within 3s; removing pid file anyway"
    );
    teamcomm_daemon::pid::remove_pid_file(&pid_file)?;
    Ok(())
}

async fn cmd_status(socket_path: Option<PathBuf>, pid_file: Option<PathBuf>) -> Result<()> {
    let (def_sock, def_pid) = default_paths();
    let socket = socket_path.unwrap_or(def_sock);
    let pid_file = pid_file.unwrap_or(def_pid);

    let pid = match teamcomm_daemon::pid::read_pid_file(&pid_file)? {
        Some(p) => p,
        None => {
            println!("stopped (no pid file at {})", pid_file.display());
            std::process::exit(3);
        }
    };

    if teamcomm_daemon::pid::is_pid_running(pid) {
        println!("running (pid {pid}, socket {})", socket.display());
        Ok(())
    } else {
        println!("stale (pid file points to dead pid {pid})");
        std::process::exit(2);
    }
}
