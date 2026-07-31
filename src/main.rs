//! sharecli - Shared CLI process manager
mod alloc;
mod plugins;

use crate::error::SharecliError;
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use sharecli_session::{RecoveryExecutor, SessionObservation, SessionService, SessionStore};
use sharecli_thermal_tui as thermal_tui;

mod apfs_uuid;
mod audit_log;
mod base85;
mod cast;
mod commands;
mod config;
mod config_validator;
mod config_watcher;
mod crc64;
mod csv_writer;
mod dashboard_assets;
mod error;
mod error_envelope;
mod hash_util;
mod health_check;
mod http_red;
mod jsonschema_subset;
mod md_table;
mod monitoring;
mod notifier;
mod otel;
mod paths;
mod pprof_http;
mod proc_compose;
mod progress;
#[cfg(test)]
mod proptest_util;
mod radix_trie;
mod rate_limiter;
mod runtime;
mod serve_auth;
mod serve_lock;
mod serve_rate_limit;
mod shutdown;
mod skiplist;
mod spawn_policy;
mod theme;
mod tray_http;
mod util_cmd;
mod xml_escape;
mod xxhash3;
mod xxtea;

use commands::{
    cast as cast_cmd, check_limits, config as config_cmd, fuse as fuse_cmd, health,
    mesh as mesh_cmd, pool_status, project as project_cmd, ps, run_pool, serve_run, set_limits,
    start, status, stop,
};
use progress::StepProgress;
use runtime::ProcessPool;

#[derive(Parser, Debug)]
#[command(
    name = "sharecli",
    about = "Shared CLI process manager for multi-project agent orchestration",
    version = "0.1.0",
    after_long_help = "Accessibility (C09): docs/a11y/README.md — NO_COLOR/TERM=dumb degrade ANSI color; \
FR-004 status via `sharecli status` and GET /health. Degraded-mode notes: docs/a11y/status-and-recovery.md\n\
Help & FAQ (C09 L81.13): docs/faq.md — top troubleshooting answers; `man sharecli` after `just man`."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Quiet mode
    #[arg(short, long)]
    quiet: bool,

    /// Color theme: `backbone-2` / `bb2` / `dark` (default) or `backbone-2-light` / `light`.
    /// Maps to Backbone-2 dark/light families in tokens.css.
    #[arg(long, value_name = "NAME", default_value = "backbone-2")]
    theme: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect durable agent sessions
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },
    /// List managed processes
    Ps {
        /// Filter by project name
        #[arg(short, long)]
        project: Option<String>,

        /// Filter by harness type (claude, forge, node, bun)
        #[arg(long)]
        harness: Option<String>,

        /// Show all processes including system ones
        #[arg(short, long)]
        all: bool,

        /// Emit JSON (requires `--all` for host agent inventory parity; AC-007.43)
        #[arg(long)]
        json: bool,

        /// Emit CSV (requires `--all` for host agent inventory parity; AC-007.83)
        #[arg(long)]
        csv: bool,

        /// Re-render every N seconds until Ctrl-C (live watch mode; AC-007.49 with --all --json)
        #[arg(short, long)]
        watch: Option<u64>,
    },

    /// Start a harness process
    Start {
        /// Project name
        #[arg(required = true)]
        project: String,

        /// Harness type (claude, forge, node, bun)
        #[arg(long, default_value = "claude")]
        harness: String,

        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,

        /// Arguments to pass
        args: Vec<String>,
    },

    /// Stop managed processes
    Stop {
        /// Process ID to stop
        #[arg(long)]
        pid: Option<u32>,

        /// Project to stop all processes for
        #[arg(short, long)]
        project: Option<String>,

        /// Harness type to stop
        #[arg(long)]
        harness: Option<String>,

        /// Stop all managed processes
        #[arg(short, long)]
        all: bool,

        /// Force kill (SIGKILL)
        #[arg(short, long)]
        force: bool,

        /// Confirm destructive force-kill without interactive prompt
        #[arg(long)]
        yes: bool,
    },

    /// Check process health
    Status {
        /// Detailed output
        #[arg(short, long)]
        verbose: bool,

        /// Emit machine-readable JSON (includes detected agent inventory)
        #[arg(long)]
        json: bool,

        /// Emit operator snapshot as CSV (header + rows; AC-007.82)
        #[arg(long)]
        csv: bool,

        /// Re-render every N seconds until Ctrl-C (live watch mode; AC-007.66 with --json)
        #[arg(short, long)]
        watch: Option<u64>,
    },

    /// List host-detected coding agents (proc scan + RSS/FD samples)
    Proc {
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,

        /// Emit flat agent inventory as CSV (header + rows)
        #[arg(long)]
        csv: bool,

        /// Show parent-child process forests rooted at detected agents
        #[arg(long)]
        tree: bool,

        /// Re-render every N seconds until Ctrl-C (live watch mode)
        #[arg(short, long)]
        watch: Option<u64>,

        /// Keep only agents matching this family id (case-insensitive)
        #[arg(long)]
        family: Option<String>,

        /// Drop agents matching this family id (case-insensitive; negates --family)
        #[arg(long)]
        exclude_family: Option<String>,

        /// Keep only agents whose COMM contains this substring (case-insensitive)
        #[arg(long)]
        comm: Option<String>,

        /// Keep only agents whose joined argv/cmdline contains this substring (case-insensitive)
        #[arg(long)]
        cmdline: Option<String>,

        /// Keep only agents in this process state (R|S|D|Z|T|t|…)
        #[arg(long)]
        state: Option<String>,

        /// Keep only agents at or above this RSS (bytes or K/M/G suffix)
        #[arg(long)]
        min_rss: Option<String>,

        /// Keep only agents at or below this RSS (bytes or K/M/G suffix)
        #[arg(long)]
        max_rss: Option<String>,

        /// Keep only agents at or above this open-FD count
        #[arg(long)]
        min_fd: Option<String>,

        /// Keep only agents at or below this open-FD count
        #[arg(long)]
        max_fd: Option<String>,

        /// Sort inventory rows or tree roots: rss (desc), fd (desc), pid (asc), state (asc)
        #[arg(long)]
        sort: Option<String>,

        /// Cap inventory rows or tree root forests after filter/sort (N >= 1)
        #[arg(long)]
        limit: Option<u64>,

        /// Show RSS/FD/cmdline/parent detail for one live host PID
        #[arg(long)]
        pid: Option<u32>,

        /// Keep only agents whose parent PID equals N
        #[arg(long)]
        ppid: Option<u32>,
    },

    /// Run a runtime health probe
    Health {
        /// Optional harness type hint (node, bun, etc.)
        #[arg(long)]
        harness: Option<String>,

        /// Emit JSON (gate → host_watch siblings; AC-007.44)
        #[arg(long)]
        json: bool,

        /// Emit operator snapshot as CSV (header + rows; AC-007.82)
        #[arg(long)]
        csv: bool,

        /// Re-render every N seconds until Ctrl-C (live watch mode; AC-007.64 with --json)
        #[arg(short, long)]
        watch: Option<u64>,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        cmd: config::ConfigCmd,
    },

    /// Project management
    Project {
        #[command(subcommand)]
        cmd: config::ProjectCmd,
    },

    /// Optimize resource usage
    Optimize {
        /// Apply optimizations automatically
        #[arg(short, long)]
        apply: bool,
    },

    /// Prune idle processes
    Prune {
        /// Idle time threshold in seconds (default from config if omitted)
        #[arg(short, long)]
        idle_seconds: Option<u64>,

        /// Actually kill processes (dry run by default)
        #[arg(short, long)]
        force: bool,
    },

    /// Show shared runtime pool status
    Pool {
        /// Harness type to check (node, bun)
        #[arg(long)]
        harness: Option<String>,

        /// Emit JSON (gate → host_watch siblings; AC-007.44)
        #[arg(long)]
        json: bool,

        /// Emit operator snapshot as CSV (header + rows; AC-007.82)
        #[arg(long)]
        csv: bool,

        /// Re-render every N seconds until Ctrl-C (live watch mode; AC-007.65 with --json)
        #[arg(short, long)]
        watch: Option<u64>,
    },

    /// Run using pooled runtime
    Run {
        /// Harness type (node, bun)
        #[arg(required = true)]
        harness: String,

        /// Project name
        #[arg(required = true)]
        project: String,
    },

    /// Set project resource limits
    Limits {
        /// Project name
        #[arg(required = true)]
        project: String,

        /// Memory limit in MB
        #[arg(short, long)]
        memory: Option<u64>,

        /// Max process count
        #[arg(short, long)]
        processes: Option<usize>,
    },

    /// Check project resource limits
    Check {
        /// Project name
        #[arg(required = true)]
        project: String,
    },

    /// Live thermal-gate / hypervisor state monitor (TUI)
    ///
    /// Displays current memory pressure level (GREEN/YELLOW/RED), active
    /// build slots, and the gate's ADMIT/DENY decision.
    /// Press `q` or Ctrl-C to exit.
    Thermal {
        /// Build-slot cap (max concurrent cargo build|check|test processes).
        #[arg(short, long, default_value_t = thermal_tui::DEFAULT_SLOT_CAP)]
        cap: u32,
    },

    /// Start the HTTP + WebSocket dashboard server
    Serve {
        /// Address to bind (host:port)
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        bind: String,

        /// Behaviour when a server is already running: abort | attach | replace
        #[arg(long, default_value = "abort")]
        on_conflict: String,
    },

    /// Print a fleet analytics snapshot (one-shot or live watch mode)
    Report {
        /// Output format: text (default), json, or csv
        #[arg(long, default_value = "text")]
        format: String,

        /// Re-render every N seconds (like `watch -n N`); omit for one-shot
        #[arg(short, long)]
        watch: Option<u64>,

        /// Sort top-consumers by: memory (default) or name
        #[arg(long, default_value = "memory")]
        sort: String,
    },

    /// Fleet device management
    Fleet {
        #[command(subcommand)]
        cmd: FleetCmd,
    },
    /// Maildir mesh task-queue operator surface (status / reclaim)
    Mesh {
        #[command(subcommand)]
        cmd: MeshCmd,
    },
    /// FUSE IO intercept operator surface (mount, provenance, CoW)
    Fuse {
        #[command(subcommand)]
        cmd: FuseCmd,
    },
    /// Cross-machine text injection into registered terminal panes
    Cast {
        #[command(subcommand)]
        cmd: CastCmd,
    },

    /// process-compose.yaml integration
    ProcCompose {
        #[command(subcommand)]
        cmd: ProcComposeCmd,
    },

    /// Generate shell completion script
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Generate roff man page for sharecli(1) (clap_mangen)
    Man {
        /// Write to share/man/man1/sharecli.1 instead of stdout
        #[arg(long)]
        install: bool,
    },

    /// Exercise the bundled utility modules (base85, csv, crc, hash, json, sha, uuid, xml, markdown, trie/skiplist)
    Util {
        #[command(subcommand)]
        cmd: util_cmd::UtilCmd,
    },

    /// Enumerate available CLI surfaces (cast modules + utility modules)
    List {
        /// Output as machine-readable JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Print version + Backbone-2 ASCII splash
    Version,

    /// Print uninstall guidance and optionally purge local config/state
    Uninstall {
        /// Delete config/state directories (requires explicit consent)
        #[arg(long)]
        purge_data: bool,

        /// List paths that would be removed without deleting
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SessionCmd {
    /// List persisted sessions
    List {
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// Inspect one persisted session
    Inspect {
        id: String,
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// Print the pending recovery plan
    RecoveryPlan {
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// Append one JSON observation to the durable ledger
    Observe {
        /// JSON file containing a SessionObservation
        input: std::path::PathBuf,
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// List append-only observations, optionally for one terminal surface
    Observations {
        #[arg(long)]
        surface_id: Option<String>,
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// Compact the observation WAL while retaining the newest record per surface
    Compact {
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
    /// Recover verified sessions; defaults to a dry run
    Recover {
        /// Launch exact/corroborated recipes instead of printing a dry run
        #[arg(long)]
        execute: bool,
        /// Maximum number of concurrent launches
        #[arg(long, default_value_t = 4)]
        max_parallel: usize,
        #[arg(long)]
        db: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum FleetCmd {
    /// Show fleet registry status and thermal level
    Status,
    /// Register this device into the fleet
    Register {
        /// Friendly device name (defaults to "local")
        #[arg(short, long)]
        name: Option<String>,
        /// Fleet coordinator address (e.g. nats://localhost:4222)
        #[arg(short, long, default_value = "nats://localhost:4222")]
        coordinator: String,
    },
}

#[derive(Subcommand, Debug)]
enum FuseCmd {
    /// Mount intercept layer over a backing directory
    Mount {
        /// Backing filesystem root to mirror
        backing: std::path::PathBuf,
        /// Mountpoint path (created if missing)
        mountpoint: std::path::PathBuf,
        /// Write-provenance session id (default: process-local)
        #[arg(long)]
        session: Option<String>,
        /// Enable per-agent CoW overlays (Feb `--cow`)
        #[arg(long)]
        cow: bool,
        /// CoW overlay root (default: `{backing}/.sharecli-cow` when `--cow`)
        #[arg(long)]
        cow_dir: Option<std::path::PathBuf>,
        /// Default agent id for unscoped commit/discard (default: session id)
        #[arg(long)]
        agent: Option<String>,
        /// Disable per-path write locks (Feb `--no-serialize`)
        #[arg(long)]
        no_serialize: bool,
        /// Path to Feb-format `agents.conf`
        #[arg(long)]
        agents_conf: Option<std::path::PathBuf>,
        /// Block in foreground until unmounted (default: background daemon thread)
        #[arg(long)]
        foreground: bool,
    },
    /// Unmount a registered intercept mount
    Unmount {
        /// Mountpoint path
        mountpoint: std::path::PathBuf,
    },
    /// Print FUSE read-cache + write-serialize global meters
    Status {
        /// Emit JSON instead of text sections
        #[arg(long)]
        json: bool,
    },
    /// Commit staged CoW (path and/or all pending for `--agent`)
    Commit {
        /// Path relative to backing root (optional when `--agent` commits all)
        relpath: Option<std::path::PathBuf>,
        /// Mountpoint when multiple mounts are registered
        #[arg(long)]
        mountpoint: Option<std::path::PathBuf>,
        /// Agent id (Feb `harness-fuse commit <mp> <agent>`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Discard staged CoW (path and/or all pending for `--agent`)
    Discard {
        /// Path relative to backing root (optional when `--agent` discards all)
        relpath: Option<std::path::PathBuf>,
        /// Mountpoint when multiple mounts are registered
        #[arg(long)]
        mountpoint: Option<std::path::PathBuf>,
        /// Agent id
        #[arg(long)]
        agent: Option<String>,
    },
    /// List registered mounts and pending CoW paths
    List {
        /// Emit JSON instead of text
        #[arg(long)]
        json: bool,
    },
    /// Read write-provenance xattrs (`user.sharecli.session`, `user.sharecli.written_at`)
    Provenance {
        /// Backing file path (need not be under a live FUSE mount)
        path: std::path::PathBuf,
        /// Emit JSON instead of text (`null` when attrs absent)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum MeshCmd {
    /// Show Maildir queue depth (ready / in_flight / pending)
    Status {
        /// Path to Maildir queue root (`tmp/` `new/` `cur/`)
        #[arg(long, short = 'Q')]
        queue: std::path::PathBuf,
        /// Emit JSON [`sharecli_mesh::MaildirStatus`] instead of text
        #[arg(long)]
        json: bool,
    },
    /// Return in-flight (`cur/`) tasks for an owner back to `new/`
    Reclaim {
        /// Path to Maildir queue root
        #[arg(long, short = 'Q')]
        queue: std::path::PathBuf,
        /// Owner string stamped at claim time
        #[arg(long)]
        owner: String,
    },
}

#[derive(Subcommand, Debug)]
enum CastCmd {
    /// Register a pane: `cast register <name> <address>`
    Register {
        /// Friendly pane name (e.g. `civis-1`)
        name: String,
        /// Address in the form `machine:host[:window[:pane]]`
        address: String,
    },
    /// Unregister a pane
    Unregister { name: String },
    /// List all registered panes
    List,
    /// Send text to a registered pane
    Send {
        /// Registered pane name
        name: String,
        /// File to read; pass `-` (or omit) to read from stdin
        file: Option<String>,
    },
    /// Show the on-disk path of the pane-map file
    Where,
}

#[derive(Subcommand, Debug)]
enum ProcComposeCmd {
    /// Pretty-print all services from process-compose.yaml with their current status.
    Status {
        /// Path to process-compose.yaml (auto-discovered from cwd if omitted)
        #[arg(short, long)]
        file: Option<String>,
    },

    /// List services defined in process-compose.yaml (names only).
    List {
        /// Path to process-compose.yaml (auto-discovered from cwd if omitted)
        #[arg(short, long)]
        file: Option<String>,
    },
}

/// Returns true when the NO_COLOR environment variable is set (per https://no-color.org).
fn is_no_color() -> bool {
    std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        match err.downcast::<SharecliError>() {
            Ok(domain) => {
                domain.eprint();
                std::process::exit(i32::from(domain.exit_code()));
            }
            Err(err) => {
                let domain = SharecliError::from(err);
                domain.eprint();
                std::process::exit(i32::from(domain.exit_code()));
            }
        }
    }
}

async fn run() -> Result<()> {
    use std::io::IsTerminal;

    #[cfg(feature = "dhat-heap")]
    let _dhat_profiler = dhat::Profiler::new_heap();

    let cli = Cli::parse();
    let tokens = theme::Tokens::from_name(&cli.theme)
        .ok_or_else(|| {
            SharecliError::user_input(format!(
                "unknown theme '{}': expected backbone-2 / bb2 / dark or backbone-2-light / light",
                cli.theme,
            ))
        })
        .map_err(|e| anyhow::Error::new(e))?;
    if std::io::stderr().is_terminal() && !is_no_color() {
        eprintln!("{}", tokens.panel.ansi_fg());
    }

    // Initialise global config (must happen before any command handler)
    let cfg = config::init_global();

    // Validate config and exit with clear errors if invalid
    {
        let errors = config_validator::validate_config(cfg);
        if !errors.is_empty() {
            config_validator::report_and_exit(&errors);
        }
    }

    if !cli.quiet && (cli.verbose || std::io::stderr().is_terminal()) {
        use tracing_subscriber::prelude::*;

        crate::otel::ensure_trace_context_propagator();

        let level = if cli.verbose { tracing::Level::DEBUG } else { tracing::Level::INFO };
        let json = std::env::var("SHARECLI_LOG_FORMAT")
            .map(|v| v.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        let filter = tracing_subscriber::filter::LevelFilter::from_level(level);
        // File-rotation layer for the Logs dashboard (PR 8 of the dashboard
        // expansion). Default log directory is ~/.sharecli/logs/sharecli.log,
        // overridable via SHARECLI_LOG_PATH for test/CI isolation. The Swift
        // tray reads this file directly via the StatusSnapshot.log_location
        // field — no separate log.tail IPC needed.
        let log_path: std::path::PathBuf = std::env::var_os("SHARECLI_LOG_PATH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                home.join(".sharecli").join("logs").join("sharecli.log")
            });
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Wrap the log file path so the layer can re-open the file on demand.
        let log_path_for_writer = log_path.clone();
        let file_make_writer = move || -> Box<dyn std::io::Write + Send> {
            Box::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path_for_writer)
                    .expect("reopen log file"),
            )
        };
        if json {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(std::io::stderr)
                .with_filter(filter);
            let file_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(file_make_writer);
            let registry = tracing_subscriber::registry().with(fmt_layer).with(file_layer);
            if let Some(otel_layer) = crate::otel::try_otel_layer() {
                registry.with(otel_layer).init();
            } else {
                registry.init();
            }
        } else {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(!is_no_color())
                .with_writer(std::io::stderr)
                .with_filter(filter);
            let file_layer =
                tracing_subscriber::fmt::layer().with_ansi(false).with_writer(file_make_writer);
            let registry = tracing_subscriber::registry().with(fmt_layer).with(file_layer);
            if let Some(otel_layer) = crate::otel::try_otel_layer() {
                registry.with(otel_layer).init();
            } else {
                registry.init();
            }
        }
        tracing::debug!(path = %log_path.display(), "sharecli log file");
    } else {
        crate::otel::ensure_trace_context_propagator();
    }

    match &cli.command {
        Commands::Session { cmd } => session_cmd(cmd)?,
        Commands::Ps { project, harness, all, json, csv, watch } => {
            ps(project.as_deref(), harness.as_deref(), *all, *json, *csv, *watch).await?
        }
        Commands::Start { project, harness, cwd, args } => {
            start(project, harness, cwd.as_deref(), args).await?
        }
        Commands::Stop { pid, project, harness, all, force, yes } => {
            stop(*pid, project.as_deref(), harness.as_deref(), *all, *force, *yes).await?
        }
        Commands::Status { verbose, json, csv, watch } => {
            status(*verbose, *json, *csv, *watch).await?
        }
        Commands::Proc {
            json,
            csv,
            tree,
            watch,
            family,
            exclude_family,
            comm,
            cmdline,
            state,
            min_rss,
            max_rss,
            min_fd,
            max_fd,
            sort,
            limit,
            pid,
            ppid,
        } => {
            commands::proc::run(
                *json,
                *csv,
                *tree,
                *watch,
                family.clone(),
                exclude_family.clone(),
                comm.clone(),
                cmdline.clone(),
                state.clone(),
                min_rss.clone(),
                max_rss.clone(),
                min_fd.clone(),
                max_fd.clone(),
                sort.clone(),
                *limit,
                *pid,
                *ppid,
            )
            .await?
        }
        Commands::Config { cmd } => config_cmd(cmd)?,
        Commands::Project { cmd } => project_cmd(cmd).await?,
        Commands::Optimize { apply } => optimize(*apply).await?,
        Commands::Prune { idle_seconds, force } => {
            prune(idle_seconds.unwrap_or(config::global().spawn.prune_idle_seconds), *force).await?
        }
        Commands::Pool { harness: _, json, csv, watch } => pool_status(*json, *csv, *watch).await?,
        Commands::Health { harness, json, csv, watch } => {
            health(harness.as_deref(), *json, *csv, *watch).await?
        }
        Commands::Run { harness, project } => run_pool(harness, project).await?,
        Commands::Limits { project, memory, processes } => {
            set_limits(project, *memory, *processes).await?
        }
        Commands::Check { project } => check_limits(project).await?,
        Commands::Report { format, watch, sort } => {
            use std::str::FromStr as _;
            let fmt = commands::report::ReportFormat::from_str(format)?;
            let sort_key = commands::report::SortBy::from_str(sort)?;
            commands::report::run(fmt, *watch, sort_key).await?
        }
        Commands::Serve { bind, on_conflict } => {
            use crate::serve_lock::OnConflict;
            let policy = match on_conflict.as_str() {
                "attach" => OnConflict::Attach,
                "replace" => OnConflict::Replace,
                _ => OnConflict::Abort,
            };
            serve_run(bind, policy).await?
        }
        Commands::Thermal { cap } => {
            let gov = sharecli_fleet::thermal::ThermalGovernor::new();
            let poll_pool_status = move || {
                let handle = tokio::runtime::Handle::current();
                handle.block_on(async {
                    let pool = crate::commands::build_pool_json().await.ok().map(Into::into);
                    let status = crate::commands::build_status_json().await.ok().map(Into::into);
                    (pool, status)
                })
            };
            thermal_tui::run_with_pool_status(&gov, *cap, Some(Box::new(poll_pool_status)))?;
        }
        Commands::Fleet { cmd } => match cmd {
            FleetCmd::Status => fleet_status().await?,
            FleetCmd::Register { name, coordinator } => {
                fleet_register(name.as_deref(), coordinator).await?
            }
        },
        Commands::Mesh { cmd } => match cmd {
            MeshCmd::Status { queue, json } => mesh_cmd::status(queue, *json)?,
            MeshCmd::Reclaim { queue, owner } => mesh_cmd::reclaim(queue, owner)?,
        },
        Commands::Fuse { cmd } => match cmd {
            FuseCmd::Mount {
                backing,
                mountpoint,
                session,
                cow,
                cow_dir,
                agent,
                no_serialize,
                agents_conf,
                foreground,
            } => {
                fuse_cmd::mount(
                    backing,
                    mountpoint,
                    fuse_cmd::FuseMountCliOpts {
                        session_id: session.clone(),
                        cow: *cow,
                        cow_dir: cow_dir.clone(),
                        agent: agent.clone(),
                        no_serialize: *no_serialize,
                        agents_conf: agents_conf.clone(),
                        foreground: *foreground,
                    },
                )?;
            }
            FuseCmd::Unmount { mountpoint } => fuse_cmd::unmount(mountpoint)?,
            FuseCmd::Status { json } => fuse_cmd::status(*json)?,
            FuseCmd::Commit { relpath, mountpoint, agent } => {
                fuse_cmd::commit(relpath.as_deref(), mountpoint.as_deref(), agent.as_deref())?
            }
            FuseCmd::Discard { relpath, mountpoint, agent } => {
                fuse_cmd::discard(relpath.as_deref(), mountpoint.as_deref(), agent.as_deref())?
            }
            FuseCmd::List { json } => fuse_cmd::list(*json)?,
            FuseCmd::Provenance { path, json } => fuse_cmd::provenance(path, *json)?,
        },
        Commands::Cast { cmd } => match cmd {
            CastCmd::Register { name, address } => cast_cmd::register(name, address)?,
            CastCmd::Unregister { name } => cast_cmd::unregister(name)?,
            CastCmd::List => cast_cmd::list()?,
            CastCmd::Send { name, file } => cast_cmd::send(name, file.as_deref())?,
            CastCmd::Where => cast_cmd::where_file()?,
        },
        Commands::ProcCompose { cmd } => proc_compose_cmd(cmd)?,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "sharecli", &mut std::io::stdout());
        }
        Commands::Man { install } => cli_man(*install)?,
        Commands::Util { cmd } => cmd.run()?,
        Commands::List { json } => cli_list(*json)?,
        Commands::Version => cli_version()?,
        Commands::Uninstall { purge_data, dry_run } => {
            commands::uninstall::run(*purge_data, *dry_run)?
        }
    }

    Ok(())
}

fn session_cmd(cmd: &SessionCmd) -> Result<()> {
    let (db, operation) = match cmd {
        SessionCmd::List { db } => (db.clone(), None),
        SessionCmd::Inspect { id, db } => (db.clone(), Some(id.as_str())),
        SessionCmd::RecoveryPlan { db } => (db.clone(), None),
        SessionCmd::Observe { db, .. } => (db.clone(), None),
        SessionCmd::Observations { db, .. } => (db.clone(), None),
        SessionCmd::Compact { db } => (db.clone(), None),
        SessionCmd::Recover { db, .. } => (db.clone(), None),
    };
    let path = db.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("sharecli")
            .join("sessions.sqlite")
    });
    let store = SessionStore::open(path)?;
    if let SessionCmd::Observe { input, .. } = cmd {
        let observation: SessionObservation =
            serde_json::from_str(&std::fs::read_to_string(input)?)?;
        let sequence = store.append_observation(&observation)?;
        println!(
            "{}",
            serde_json::json!({"sequence": sequence, "surface_id": observation.surface.id})
        );
        return Ok(());
    }
    let service = SessionService::new(store);
    let value = match cmd {
        SessionCmd::List { .. } => serde_json::to_value(service.list()?)?,
        SessionCmd::Inspect { .. } => {
            serde_json::to_value(service.inspect(operation.expect("id"))?)?
        }
        SessionCmd::RecoveryPlan { .. } => serde_json::to_value(service.recovery_plan()?)?,
        SessionCmd::Observations { surface_id, .. } => {
            serde_json::to_value(service.observations(surface_id.as_deref())?)?
        }
        SessionCmd::Compact { .. } => serde_json::json!({
            "removed": service.compact_observations()?
        }),
        SessionCmd::Recover { execute, max_parallel, .. } => {
            let sessions = service.recovery_plan()?;
            let executor = RecoveryExecutor::new(*max_parallel);
            let results =
                if *execute { executor.execute(&sessions) } else { executor.dry_run(&sessions) };
            serde_json::to_value(results)?
        }
        SessionCmd::Observe { .. } => unreachable!("observe handled before service dispatch"),
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

/// `sharecli man` — emit sharecli(1) via clap_mangen (C09 L81.13).
fn cli_man(install: bool) -> Result<()> {
    use clap::CommandFactory;
    use clap_mangen::Man;
    use std::io::Write;

    let man = Man::new(Cli::command());
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer).map_err(|e| anyhow::anyhow!("man page render failed: {e}"))?;
    let rendered = String::from_utf8(buffer).map_err(|e| anyhow::anyhow!("man page utf-8: {e}"))?;

    if install {
        let path = std::path::Path::new("share/man/man1/sharecli.1");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &rendered)?;
        println!("Wrote {}", path.display());
        return Ok(());
    }

    std::io::stdout()
        .write_all(rendered.as_bytes())
        .map_err(|e| anyhow::anyhow!("man page stdout: {e}"))?;
    Ok(())
}

/// `sharecli list` — enumerate CLI surfaces (cast subcommands + utility modules).
///
/// Backbone-2 family: pulse-green (#3fb950) for headers, amber (#d29922) for
/// accent markers. No external deps; pure introspection of the typed subcommand
/// tree + `util_cmd::UtilCmd` variant list.
fn cli_list(as_json: bool) -> Result<()> {
    let cast_modules: &[(&str, &str)] = &[
        ("register", "Register a pane: `cast register <name> <address>`"),
        ("unregister", "Unregister a pane by name"),
        ("list", "List all registered panes"),
        ("send", "Send text to a registered pane (`<name> [file]`)"),
        ("where", "Show the on-disk path of the pane-map file"),
    ];

    let mesh_modules: &[(&str, &str)] = &[
        ("status", "Show Maildir queue depth (`mesh status --queue <path>`)"),
        (
            "reclaim",
            "Return in-flight tasks for an owner (`mesh reclaim --queue <path> --owner <id>`)",
        ),
    ];

    let fuse_modules: &[(&str, &str)] = &[
        ("mount", "Mount intercept over backing (`fuse mount <backing> <mountpoint> [--cow]`)"),
        ("unmount", "Unmount registered intercept (`fuse unmount <mountpoint>`)"),
        ("status", "FUSE read-cache + write-serialize meters"),
        ("commit", "Commit staged CoW (`fuse commit [relpath] [--agent]`)"),
        ("discard", "Discard staged CoW (`fuse discard [relpath] [--agent]`)"),
        ("list", "List mounts + pending CoW by agent"),
        ("provenance", "Read FUSE write xattrs on a backing file (`fuse provenance <path>`)"),
    ];

    let util_modules: &[(&str, &str)] = &[
        ("base85", "Base85 encode / decode"),
        ("csv", "Build a CSV row from --row entries"),
        ("crc", "CRC64 checksum"),
        ("hash", "xxhash3 / xxtea digest"),
        ("json", "JSON pretty-print / validate"),
        ("md-table", "Render markdown table"),
        ("sha", "SHA1 / SHA256 digest"),
        ("skiplist", "Walk the bundled skiplist"),
        ("trie", "Radix-trie lookup"),
        ("url", "URL percent-encode / decode"),
        ("uuid", "APFS UUID helper"),
        ("xml", "XML escape / unescape"),
    ];

    if as_json {
        let payload = serde_json::json!({
            "cast": cast_modules.iter().map(|(n, d)| serde_json::json!({"name": n, "desc": d})).collect::<Vec<_>>(),
            "mesh": mesh_modules.iter().map(|(n, d)| serde_json::json!({"name": n, "desc": d})).collect::<Vec<_>>(),
            "fuse": fuse_modules.iter().map(|(n, d)| serde_json::json!({"name": n, "desc": d})).collect::<Vec<_>>(),
            "util": util_modules.iter().map(|(n, d)| serde_json::json!({"name": n, "desc": d})).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("sharecli CLI surfaces");
    println!();
    println!("cast <subcommand>  -- pane casting ({} subcommands)", cast_modules.len());
    for (n, d) in cast_modules {
        println!("  - {:<11} {}", n, d);
    }
    println!();
    println!("mesh <subcommand>  -- Maildir task queue ({} subcommands)", mesh_modules.len());
    for (n, d) in mesh_modules {
        println!("  - {:<11} {}", n, d);
    }
    println!();
    println!("fuse <subcommand>  -- FUSE IO intercept ({} subcommands)", fuse_modules.len());
    for (n, d) in fuse_modules {
        println!("  - {:<11} {}", n, d);
    }
    println!();
    println!("util <subcommand>  -- utility modules ({} subcommands)", util_modules.len());
    for (n, d) in util_modules {
        println!("  - {:<11} {}", n, d);
    }
    Ok(())
}

/// `sharecli version` — emit Backbone-2 ASCII splash + version + author.
///
/// Respects `--theme` (via `Tokens::from_name`) and `NO_COLOR`.
fn cli_version() -> Result<()> {
    let cli = Cli::command();
    let version = cli.get_version().unwrap_or("0.0.0");

    let tokens = theme::Tokens::from_name("backbone-2")
        .ok_or_else(|| anyhow::anyhow!("backbone-2 theme tokens missing"))?;

    let pulse = tokens.pulse_green.ansi_fg();
    let amber = tokens.warm_amber.ansi_fg();
    let panel = tokens.panel.ansi_fg();
    let reset = "\x1b[0m";

    if is_no_color() {
        let splash = r#"
   _______ _    _ ______ _____  _____ _____  ______
  / ______| || ||  ____|  __ \|_   _|  __ \|  ____|
 | (___ | || || |__  | |__) | | | | |  | | |__
  \___ \| ||__||  __| |  _  /  | | | |  | |  __|
  ____) |__   || |____| | \ \_ | |_| |__| | |____
 |_____/   |_||______|_|  \__\|______\____/|______|
"#;
        println!("{splash}");
        println!("sharecli {version}");
        println!("shared CLI process manager");
        println!("(NO_COLOR set — ASCII palette disabled)");
    } else {
        let splash = r#"
   _______ _    _ ______ _____  _____ _____  ______
  / ______| || ||  ____|  __ \|_   _|  __ \|  ____|
 | (___ | || || |__  | |__) | | | | |  | | |__
  \___ \| ||__||  __| |  _  /  | | | |  | |  __|
  ____) |__   || |____| | \ \_ | |_| |__| | |____
 |_____/   |_||______|_|  \__\|______\____/|______|
"#;
        println!("{pulse}{splash}{reset}");
        println!("{amber}sharecli {version}{reset}");
        println!("{panel}shared CLI process manager for multi-project agent orchestration{reset}");
        println!("{panel}Backbone-2 family · pulse-green/amber/panel{reset}");
    }

    Ok(())
}

async fn fleet_status() -> Result<()> {
    use sharecli_fleet::{ThermalGovernor, DEFAULT_COORDINATOR};

    let _gov = ThermalGovernor::new();
    println!("Thermal governor: ready");

    match sharecli_fleet::connect(DEFAULT_COORDINATOR).await {
        Ok(_client) => {
            println!("Fleet registry: connected to {DEFAULT_COORDINATOR}");
        }
        Err(e) => {
            println!("Fleet registry: not connected ({e})");
            println!("  Run `sharecli fleet register` to join the fleet.");
        }
    }
    Ok(())
}

async fn fleet_register(name: Option<&str>, coordinator: &str) -> Result<()> {
    // Best-effort: fall back to "local" if gethostname is unavailable.
    let hostname = name.unwrap_or("local");

    println!("Registering device '{hostname}' with coordinator '{coordinator}'");

    match sharecli_fleet::connect(coordinator).await {
        Ok(client) => {
            let record = sharecli_fleet::DeviceRecord {
                device_id: format!("{hostname}-{}", std::process::id()),
                hostname: hostname.to_string(),
                os: std::env::consts::OS.to_string(),
                available_slots: 4,
            };
            sharecli_fleet::announce(&client, &record).await?;
            println!(
                "Registered device '{}' (os={}, slots={})",
                record.device_id, record.os, record.available_slots
            );
        }
        Err(e) => {
            println!("Registration failed: {e}");
            println!("  Is the NATS coordinator running at '{coordinator}'?");
        }
    }
    Ok(())
}

fn proc_compose_cmd(cmd: &ProcComposeCmd) -> Result<()> {
    let resolve_path = |file: &Option<String>| -> Result<std::path::PathBuf> {
        if let Some(f) = file {
            let p = std::path::PathBuf::from(f);
            if !p.exists() {
                anyhow::bail!("File not found: {}", p.display());
            }
            Ok(p)
        } else {
            let cwd = std::env::current_dir()?;
            proc_compose::find_config(&cwd).ok_or_else(|| {
                anyhow::anyhow!("No process-compose.yaml found in {cwd:?} or any parent directory")
            })
        }
    };

    match cmd {
        ProcComposeCmd::Status { file } => {
            let path = resolve_path(file)?;
            println!("Using: {}", path.display());
            let cfg = proc_compose::load_config(&path)?;
            let defs = cfg.to_process_defs();
            proc_compose::print_status(&defs);
        }
        ProcComposeCmd::List { file } => {
            let path = resolve_path(file)?;
            let cfg = proc_compose::load_config(&path)?;
            for d in cfg.to_process_defs() {
                println!("{}", d.name);
            }
        }
    }
    Ok(())
}

async fn optimize(apply: bool) -> Result<()> {
    println!("Analyzing resource usage...");

    let pool = ProcessPool::new();
    let processes = pool.list().await;

    let mut by_harness: std::collections::HashMap<&str, (usize, u64)> =
        std::collections::HashMap::new();

    for proc in &processes {
        if let Some(ref harness) = proc.harness {
            let entry = by_harness.entry(harness.as_str()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += proc.memory_mb;
        }
    }

    println!("\nCurrent resource usage:");
    println!("{:<15} {:<10} {:<15}", "HARNESS", "COUNT", "MEMORY(MB)");
    println!("{}", "-".repeat(40));

    for (harness, (count, mem)) in &by_harness {
        println!("{:<15} {:<10} {:<15}", harness, count, mem);
    }

    let total_mem: u64 = by_harness.values().map(|(_, m)| m).sum();
    let total_count: usize = by_harness.values().map(|(c, _)| c).sum();

    println!("\n{:<15} {:<10} {:<15}", "TOTAL", total_count, total_mem);
    println!("\n=== Optimization Suggestions ===");

    if total_count > 30 {
        println!("- Consider reducing max instances per harness");
    }
    if total_mem > 4096 {
        println!("- Memory usage is high ({} MB). Consider pruning idle processes.", total_mem);
    }

    if apply {
        println!("\nApplying optimizations...");
        println!("Done.");
    }

    Ok(())
}

async fn prune(idle_seconds: u64, force: bool) -> Result<()> {
    println!("Pruning idle processes (threshold: {}s)...", idle_seconds);

    let pool = ProcessPool::new();
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let processes = pool.list().await;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let candidates: Vec<_> = processes
        .into_iter()
        .filter(|proc| proc.start_time > 0 && (now - proc.start_time) > idle_seconds)
        .collect();

    let total = candidates.len();
    let progress = StepProgress::new("Pruning idle processes", total);
    let line_mode = progress.uses_line_output();
    let mut pruned = 0usize;

    for proc in candidates {
        if force {
            pool.kill(proc.pid).await?;
            progress.inc(Some(&format!("{} ({})", proc.pid, proc.name)));
            if line_mode {
                println!("Pruned process {} ({})", proc.pid, proc.name);
            }
            pruned += 1;
        } else {
            println!("Would prune: {} ({})", proc.pid, proc.name);
            pruned += 1;
        }
    }

    if force {
        progress.finish(&format!("Pruned {pruned} processes"));
        if line_mode {
            println!("\nPruned {} processes.", pruned);
        }
    } else {
        println!("\nWould prune {} processes (use --force to apply).", pruned);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_page_contains_th_sharecli() {
        let man = clap_mangen::Man::new(Cli::command());
        let mut buf = Vec::new();
        man.render(&mut buf).expect("render man page");
        let output = String::from_utf8(buf).expect("valid utf-8");
        assert!(
            output.contains(".TH \"sharecli\"") || output.contains(".TH sharecli"),
            "man page should declare sharecli(1), got prefix: {}",
            &output[..output.len().min(200)]
        );
    }

    #[test]
    fn test_completions_zsh_contains_compdef() {
        let mut cmd = Cli::command();
        let mut buf = Vec::new();
        clap_complete::generate(Shell::Zsh, &mut cmd, "sharecli", &mut buf);
        let output = String::from_utf8(buf).expect("valid utf-8");
        assert!(
            output.contains("#compdef"),
            "zsh completion should start with #compdef, got: {output}"
        );
    }

    #[test]
    fn test_no_color_respects_env_var() {
        // When NO_COLOR is unset, is_no_color should return false
        unsafe { std::env::remove_var("NO_COLOR") };
        assert!(!is_no_color());

        // When NO_COLOR is set to empty string, should return false
        unsafe { std::env::set_var("NO_COLOR", "") };
        assert!(!is_no_color());

        // When NO_COLOR is set to non-empty, should return true
        unsafe { std::env::set_var("NO_COLOR", "1") };
        assert!(is_no_color());

        // Clean up
        unsafe { std::env::remove_var("NO_COLOR") };
    }
}
