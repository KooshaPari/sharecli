// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm` — operator CLI for the teamcomm daemon.
//!
//! Connects to the running daemon over a Unix-domain socket and dispatches
//! subcommands (sessions, reservations, inbox, state, discovery) as
//! JSON-RPC 2.0 requests. The `daemon` subcommand group is purely local
//! (start/stop/status) and does not talk to a running daemon.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod cmd_daemon;
mod cmd_discover;
mod cmd_inbox;
mod cmd_reservations;
mod cmd_sessions;
mod cmd_state;
mod connect;
mod output;
mod rpc;

#[derive(Parser, Debug)]
#[command(
    name = "teamcomm",
    about = "Operator CLI for the teamcomm multi-agent coordinator",
    long_about = "Connect to a running teamcomm daemon over a Unix socket and dispatch \
                  subcommands for sessions, file reservations, inbox, live state, and discovery.",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Manage the teamcomm daemon process (start, stop, status).
    Daemon(DaemonCmd),
    /// List and inspect registered agent sessions.
    Sessions(SessionsCmd),
    /// File/path reservation commands.
    Reservations(ReservationsCmd),
    /// Inbox read/post commands.
    Inbox(InboxCmd),
    /// Live agent state (focus, branch, status) commands.
    State(StateCmd),
    /// Discovery queries (who is working on what).
    Discover(DiscoverCmd),
}

// ----- Daemon subcommand group -----

#[derive(Parser, Debug)]
struct DaemonCmd {
    #[command(subcommand)]
    sub: DaemonSub,
}

#[derive(Subcommand, Debug)]
enum DaemonSub {
    /// Start the daemon (foreground or detached).
    Start {
        /// Run in the foreground (do not detach).
        #[arg(long)]
        foreground: bool,
        /// Override the socket path the daemon should listen on.
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Write the daemon's pid to this file.
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
    /// Stop a running daemon by sending it a shutdown request.
    Stop {
        /// Override the socket path used to reach the daemon.
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Report daemon status (running, socket reachable, etc.).
    Status {
        /// Override the socket path used to reach the daemon.
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- Sessions subcommand group -----

#[derive(Parser, Debug)]
struct SessionsCmd {
    #[command(subcommand)]
    sub: SessionsSub,
}

#[derive(Subcommand, Debug)]
enum SessionsSub {
    /// List all registered sessions.
    List {
        /// Continuously refresh the list (poll every 1s).
        #[arg(long)]
        watch: bool,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Show details for a single session.
    Show {
        /// Session id to look up.
        session_id: String,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- Reservations subcommand group -----

#[derive(Parser, Debug)]
struct ReservationsCmd {
    #[command(subcommand)]
    sub: ReservationsSub,
}

#[derive(Subcommand, Debug)]
enum ReservationsSub {
    /// List active reservations, optionally filtered by path prefix.
    Ls {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Claim a path reservation.
    Claim {
        /// Path to reserve.
        path: PathBuf,
        /// Session id to claim on behalf of.
        #[arg(long)]
        session: String,
        #[arg(long, value_enum, default_value_t = ModeArg::Write)]
        mode: ModeArg,
        #[arg(long, default_value_t = 600)]
        ttl_sec: u64,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Release an existing reservation.
    Release {
        /// Reservation id to release.
        reservation_id: String,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- Inbox subcommand group -----

#[derive(Parser, Debug)]
struct InboxCmd {
    #[command(subcommand)]
    sub: InboxSub,
}

#[derive(Subcommand, Debug)]
enum InboxSub {
    /// List inbox messages.
    List {
        /// Session id to list inbox for.
        #[arg(long)]
        session: String,
        /// Only show unread messages.
        #[arg(long)]
        unread: bool,
        #[arg(long, default_value_t = 50)]
        limit: u32,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Read a single inbox message.
    Read {
        message_id: String,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Post a new inbox message.
    Post {
        /// Source session id.
        from_session: String,
        /// Target session id.
        to_session: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        body: String,
        #[arg(long, value_enum, default_value_t = PriorityArg::Normal)]
        priority: PriorityArg,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- State subcommand group -----

#[derive(Parser, Debug)]
struct StateCmd {
    #[command(subcommand)]
    sub: StateSub,
}

#[derive(Subcommand, Debug)]
enum StateSub {
    /// Show current live state (optionally for a specific session).
    Show {
        /// Restrict to a specific session id.
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Set the focus file for the current session.
    SetFocus {
        file: PathBuf,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Set the working status for the current session.
    SetStatus {
        /// New status string (e.g. "idle", "working", "blocked", "done").
        status: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- Discover subcommand group -----

#[derive(Parser, Debug)]
struct DiscoverCmd {
    #[command(subcommand)]
    sub: DiscoverSub,
}

#[derive(Subcommand, Debug)]
enum DiscoverSub {
    /// Discover agents, optionally filtered by path/branch/capability.
    Agents {
        /// Restrict to sessions focused on or under this path.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Restrict to sessions focused on this branch.
        #[arg(long)]
        branch: Option<String>,
        /// Capability tags the session must declare (all must match).
        #[arg(long, value_delimiter = ',')]
        capability: Vec<String>,
        #[arg(long)]
        socket: Option<PathBuf>,
    },
}

// ----- Helper enums (clap value-enum for CLI args) -----

/// Reservation strength to request on a `claim` invocation.
#[derive(clap::ValueEnum, Clone, Debug)]
enum ModeArg {
    Read,
    Write,
    Exclusive,
}

/// Delivery priority for a posted inbox message.
#[derive(clap::ValueEnum, Clone, Debug)]
enum PriorityArg {
    Low,
    Normal,
    High,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing: respect RUST_LOG, default to "info".
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false).with_writer(std::io::stderr))
        .init();

    let cli = Cli::parse();
    dispatch(cli.cmd).await
}

async fn dispatch(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Daemon(c) => cmd_daemon::run(c).await,
        Cmd::Sessions(c) => cmd_sessions::run(c).await,
        Cmd::Reservations(c) => cmd_reservations::run(c).await,
        Cmd::Inbox(c) => cmd_inbox::run(c).await,
        Cmd::State(c) => cmd_state::run(c).await,
        Cmd::Discover(c) => cmd_discover::run(c).await,
    }
}
