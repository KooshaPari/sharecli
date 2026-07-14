//! `sharecli serve` -- lock-guarded HTTP + WebSocket dashboard server.
//!
//! GET  /healthz  -- liveness probe (JSON)
//! GET  /readyz   -- readiness probe (JSON; 503 if shutdown requested)
//! WS   /ws       -- streams periodic ProcessSummary snapshots as JSON,
//!                   plus thermal pressure events when pressure changes.

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use sharecli_fleet::thermal::{ThermalGovernor, ThermalLevel};
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{info, instrument, warn, Instrument};

use crate::audit_log;
use crate::config::Config;
use crate::config_watcher::ConfigWatcher;
use crate::health_check::{HealthCheckScheduler, HealthCheckStore};
use crate::http_red::{render_http_red_metrics, HttpRedMetrics};
use crate::notifier::Notifier;
use crate::runtime::ProcessPool;
use crate::serve_auth::{self, ServeAuth};
use crate::serve_lock::{decide, probe, Decision, OnConflict, ServeState};

// ---------------------------------------------------------------------------
// Pressure parsing (pure; tested without I/O)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Thermal event broadcast
// ---------------------------------------------------------------------------

/// Lightweight thermal event forwarded to WS clients.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ThermalEvent {
    ThermalWarning { pressure: u8 },
    ThermalCritical { pressure: u8 },
}

/// Parse a raw sysctl pressure integer into a [`ThermalLevel`].
///
/// This is a pure function with no I/O; it is unit-tested below.
pub fn parse_pressure_level(raw: u8) -> Option<ThermalLevel> {
    match raw {
        1 => Some(ThermalLevel::Green),
        2 => Some(ThermalLevel::Yellow),
        4 => Some(ThermalLevel::Red),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    /// Broadcast channel for thermal events.
    thermal_tx: Arc<broadcast::Sender<ThermalEvent>>,
    /// Set to `true` when a shutdown has been requested.
    shutdown_tx: Arc<watch::Sender<bool>>,
    /// Live config — updated on hot-reload without restart.
    config: Arc<RwLock<Config>>,
    /// Shared health-check status for all monitored processes.
    health_store: HealthCheckStore,
    /// In-process HTTP RED metrics (Rate / Errors / Duration).
    http_red: Arc<HttpRedMetrics>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Entry point for `sharecli serve`.
#[instrument(skip(on_conflict), fields(bind = %bind))]
pub async fn run(bind: &str, on_conflict: OnConflict) -> Result<()> {
    let state = probe("sharecli")?;

    match decide(&state, on_conflict) {
        Decision::Attach => {
            let url = match &state {
                ServeState::Running { info, .. } => info.url.clone(),
                ServeState::Free => unreachable!(),
            };
            info!(%url, "sharecli serve: attaching to existing instance");
            println!("sharecli serve already running at {url}");
            return Ok(());
        }
        Decision::Abort => {
            let url = match &state {
                ServeState::Running { info, .. } => info.url.clone(),
                ServeState::Free => unreachable!(),
            };
            anyhow::bail!("serve already running at {url}");
        }
        Decision::Serve | Decision::Replace => {}
    }

    let url = format!("http://{bind}");
    let lock =
        crate::serve_lock::ServeLock::try_acquire("sharecli", url.clone())?.ok_or_else(|| {
            anyhow::anyhow!("could not acquire serve lock -- another instance is running")
        })?;

    info!(%url, %bind, "sharecli serve: starting HTTP listener");

    // Log current thermal level on startup.
    let gov = ThermalGovernor::new();
    match gov.poll() {
        Ok(level) => info!("sharecli serve: startup thermal pressure = {:?}", level),
        Err(e) => warn!("sharecli serve: could not read thermal pressure: {e}"),
    }

    let (thermal_tx, _) = broadcast::channel::<ThermalEvent>(64);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Build the live config and start the hot-reload watcher.
    let initial_config = Config::load().unwrap_or_default();
    let config_arc = Arc::new(RwLock::new(initial_config.clone()));
    let (cfg_tx, mut cfg_rx) = watch::channel(initial_config.clone());

    let config_path = dirs::config_dir()
        .map(|d| d.join("sharecli").join("config.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("config.toml"));

    // `_config_watcher` is kept alive by the AppState so the file watch persists
    // for the lifetime of the server.
    let _config_watcher = ConfigWatcher::new(config_path, cfg_tx)
        .inspect_err(|e| {
            warn!("config_watcher: could not start file watcher: {e}; hot-reload disabled");
        })
        .ok();

    // Spawn a task that propagates config-reload signals into the shared RwLock.
    let config_arc_writer = Arc::clone(&config_arc);
    tokio::spawn(async move {
        while cfg_rx.changed().await.is_ok() {
            let new_cfg = cfg_rx.borrow().clone();
            *config_arc_writer.write().await = new_cfg;
            info!("serve: config hot-reloaded");
        }
    });

    // Build notifier from config.
    let notifier = Notifier::new(initial_config.notifications.clone());

    // Build the health-check store and start per-process schedulers.
    let health_store: HealthCheckStore =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    {
        let scheduler =
            HealthCheckScheduler::with_notifier(Arc::clone(&health_store), Arc::clone(&notifier));
        scheduler.start(initial_config.health_checks.clone());
    }

    let auth = ServeAuth::from_env_or_config(&initial_config.serve)
        .map_err(|e| anyhow::anyhow!("serve AuthN config error: {e}"))?;
    if auth.enabled() {
        info!(
            "sharecli serve: AuthN mode={} (probes /healthz /readyz remain public)",
            auth.mode_label()
        );
        audit_log::emit("auth_enabled", json!({ "mode": auth.mode_label() }));
    } else {
        info!("sharecli serve: AuthN disabled (set SHARECLI_SERVE_TOKEN or [serve.jwt])");
        audit_log::emit("auth_disabled", json!({ "mode": "open" }));
    }
    audit_log::emit("serve_start", json!({ "url": url, "bind": bind }));

    let state = AppState {
        thermal_tx: Arc::new(thermal_tx),
        shutdown_tx: Arc::new(shutdown_tx),
        config: config_arc,
        health_store,
        http_red: Arc::new(HttpRedMetrics::default()),
    };

    // Spawn background thermal poller (uses parse_pressure_level as the canonical parser).
    tokio::spawn(thermal_poll_task(Arc::clone(&state.thermal_tx), Arc::clone(&state.shutdown_tx)));

    println!("sharecli serve listening on {url}");

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/config", get(config_handler))
        .route("/health/processes", get(health_processes_handler))
        .route("/metrics/prometheus", get(metrics_prometheus_handler))
        .route("/debug/pprof/profile", get(crate::pprof_http::profile_handler))
        .route("/ws", get(ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), http_observability_middleware))
        .layer(middleware::from_fn_with_state(auth, serve_auth::require_bearer))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;

    tokio::select! {
        result = axum::serve(listener, app) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("sharecli serve: shutdown requested (Ctrl-C)");
            println!("sharecli serve shutting down (Ctrl-C)");
        }
        _ = wait_for_shutdown(shutdown_rx) => {
            info!("sharecli serve: shutdown requested (thermal critical)");
            println!("sharecli serve shutting down (thermal critical)");
        }
    }

    // Explicit drop for clarity; drop order would handle it anyway.
    drop(lock);
    info!("sharecli serve: stopped");
    audit_log::emit("serve_stop", json!({ "bind": bind }));
    crate::otel::shutdown();
    Ok(())
}

/// Wait until the shutdown watch channel is set to `true`.
async fn wait_for_shutdown(mut rx: watch::Receiver<bool>) {
    loop {
        if *rx.borrow() {
            return;
        }
        if rx.changed().await.is_err() {
            return;
        }
        if *rx.borrow() {
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Background thermal poller
// ---------------------------------------------------------------------------

/// Read the raw sysctl pressure integer (platform-specific).
fn read_raw_pressure() -> anyhow::Result<u8> {
    // Delegate to ThermalGovernor for the sysctl call, then re-encode to u8
    // so that `parse_pressure_level` remains the single canonical parser.
    let gov = ThermalGovernor::new();
    let level = gov.poll()?;
    Ok(match level {
        ThermalLevel::Green => 1,
        ThermalLevel::Yellow => 2,
        ThermalLevel::Red => 4,
    })
}

async fn thermal_poll_task(
    tx: Arc<broadcast::Sender<ThermalEvent>>,
    shutdown_tx: Arc<watch::Sender<bool>>,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        let level = match read_raw_pressure() {
            Ok(raw) => parse_pressure_level(raw),
            Err(e) => {
                warn!("thermal poll error: {e}");
                continue;
            }
        };
        match level {
            Some(ThermalLevel::Red) => {
                info!("thermal pressure CRITICAL (4) -- broadcasting and initiating shutdown");
                let _ = tx.send(ThermalEvent::ThermalCritical { pressure: 4 });
                // Small delay so WS clients receive the message before connection drops.
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                let _ = shutdown_tx.send(true);
                return;
            }
            Some(ThermalLevel::Yellow) => {
                info!("thermal pressure WARNING (2) -- broadcasting");
                let _ = tx.send(ThermalEvent::ThermalWarning { pressure: 2 });
            }
            Some(ThermalLevel::Green) | None => {
                // No event needed for normal operation.
            }
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP handlers
// ---------------------------------------------------------------------------

/// Observability middleware: W3C `traceparent` extract/inject + HTTP RED metrics.
async fn http_observability_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();
    let incoming =
        req.headers().get("traceparent").and_then(|v| v.to_str().ok()).map(str::to_owned);

    let span = tracing::info_span!(
        "http.request",
        http.method = %method,
        http.route = %path,
        otel.kind = "server",
        traceparent = incoming.as_deref().unwrap_or(""),
    );

    let mut response = next.run(req).instrument(span).await;
    let status = response.status().as_u16();
    state.http_red.record(status, start.elapsed());

    let outgoing = incoming.unwrap_or_else(synthesize_traceparent);
    if let Ok(val) = HeaderValue::from_str(&outgoing) {
        response.headers_mut().insert(HeaderName::from_static("traceparent"), val);
    }
    response
}

/// Best-effort W3C traceparent when the client did not send one.
fn synthesize_traceparent() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let pid = u128::from(std::process::id());
    let trace_id = format!("{:032x}", nanos ^ (pid << 64));
    let span_id = format!("{:016x}", (nanos as u64) ^ (pid as u64));
    format!("00-{trace_id}-{span_id}-01")
}

const DASHBOARD_HTML: &str = include_str!("../dashboard.html");

#[instrument]
async fn dashboard() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], DASHBOARD_HTML)
}

/// Pure liveness JSON body (unit-tested without spinning the HTTP server).
pub fn healthz_json() -> serde_json::Value {
    json!({"status": "ok"})
}

/// Pure readiness decision: `(status, body)` from the shutdown flag.
pub fn readyz_response(shutdown_requested: bool) -> (StatusCode, serde_json::Value) {
    if shutdown_requested {
        (StatusCode::SERVICE_UNAVAILABLE, json!({"status": "unavailable"}))
    } else {
        (StatusCode::OK, json!({"status": "ok"}))
    }
}

#[instrument]
async fn healthz() -> impl IntoResponse {
    Json(healthz_json())
}

/// `GET /readyz` — readiness probe; 200 while serving, 503 once shutdown is requested.
#[instrument(skip(state))]
async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let (status, body) = readyz_response(*state.shutdown_tx.borrow());
    (status, Json(body))
}

/// `GET /config` — returns the current live config as JSON.
///
/// The value here reflects the last successful hot-reload; it updates
/// in-place whenever the config file is saved with valid TOML.
#[instrument(skip(state))]
async fn config_handler(State(state): State<AppState>) -> impl IntoResponse {
    let cfg = state.config.read().await.clone();
    Json(serde_json::to_value(cfg).unwrap_or_else(|_| json!({"error": "serialization failed"})))
}

// ---------------------------------------------------------------------------
// WebSocket handler
// ---------------------------------------------------------------------------

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut thermal_rx = state.thermal_tx.subscribe();
    let mut snapshot_interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    loop {
        tokio::select! {
            // Periodic process snapshot
            _ = snapshot_interval.tick() => {
                let snapshot = build_snapshot().await;
                let msg = match serde_json::to_string(&snapshot) {
                    Ok(s) => Message::Text(s.into()),
                    Err(e) => {
                        warn!("ws serialize error: {e}");
                        break;
                    }
                };
                if socket.send(msg).await.is_err() {
                    break;
                }
            }
            // Thermal event from background poller
            event = thermal_rx.recv() => {
                match event {
                    Ok(evt) => {
                        let msg = match serde_json::to_string(&evt) {
                            Ok(s) => Message::Text(s.into()),
                            Err(e) => {
                                warn!("ws thermal serialize error: {e}");
                                break;
                            }
                        };
                        if socket.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Skip missed events rather than disconnect.
                        warn!("ws thermal_rx lagged; skipping missed events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

/// `GET /health/processes` — returns health status of all monitored processes.
#[instrument(skip(state))]
async fn health_processes_handler(State(state): State<AppState>) -> impl IntoResponse {
    use std::collections::HashMap;

    let map = state.health_store.lock().await;
    // Serialize to a plain JSON map: process_name → status fields.
    let out: HashMap<&str, serde_json::Value> = map
        .iter()
        .map(|(name, status)| {
            (
                name.as_str(),
                json!({
                    "healthy": status.healthy,
                    "consecutive_failures": status.consecutive_failures,
                    "last_error": status.last_error,
                }),
            )
        })
        .collect();
    Json(serde_json::to_value(out).unwrap_or_else(|_| json!({})))
}

// ---------------------------------------------------------------------------
// Prometheus metrics handler
// ---------------------------------------------------------------------------

/// `GET /metrics/prometheus` — Prometheus text-format metrics for all tracked processes.
#[instrument(skip(state))]
async fn metrics_prometheus_handler(State(state): State<AppState>) -> impl IntoResponse {
    let pool = ProcessPool::new();
    let processes = pool.list().await;
    let health_map = state.health_store.lock().await;
    let mut body = render_prometheus_metrics(&processes, &health_map);
    render_http_red_metrics(&mut body, &state.http_red.snapshot());
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")], body)
}

/// Escape a Prometheus label value: backslash, double-quote, and newline must be escaped.
fn escape_label_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

/// Render all sharecli process metrics in Prometheus text exposition format.
///
/// This is a pure function — no I/O — so it is straightforward to unit-test.
pub fn render_prometheus_metrics(
    processes: &[crate::runtime::ProcessInfo],
    health_map: &std::collections::HashMap<String, crate::health_check::HealthStatus>,
) -> String {
    let mut out = String::with_capacity(512);

    // -- sharecli_process_memory_mb ------------------------------------------
    out.push_str("# HELP sharecli_process_memory_mb Resident memory usage in MiB per process\n");
    out.push_str("# TYPE sharecli_process_memory_mb gauge\n");
    for p in processes {
        let name = escape_label_value(&p.name);
        out.push_str(&format!(
            "sharecli_process_memory_mb{{process=\"{}\"}} {}\n",
            name, p.memory_mb
        ));
    }

    // -- sharecli_process_up -------------------------------------------------
    out.push_str(
        "# HELP sharecli_process_up 1 if the process is considered healthy, 0 otherwise\n",
    );
    out.push_str("# TYPE sharecli_process_up gauge\n");
    for p in processes {
        let name = escape_label_value(&p.name);
        let up = if let Some(status) = health_map.get(&p.name) {
            if status.healthy {
                1u8
            } else {
                0u8
            }
        } else {
            // No health-check configured → process is running, treat as up.
            1u8
        };
        out.push_str(&format!("sharecli_process_up{{process=\"{}\"}} {}\n", name, up));
    }

    // -- sharecli_health_check_consecutive_failures --------------------------
    out.push_str("# HELP sharecli_health_check_consecutive_failures Number of consecutive health-check failures\n");
    out.push_str("# TYPE sharecli_health_check_consecutive_failures gauge\n");
    for (proc_name, status) in health_map.iter() {
        let name = escape_label_value(proc_name);
        out.push_str(&format!(
            "sharecli_health_check_consecutive_failures{{process=\"{}\"}} {}\n",
            name, status.consecutive_failures
        ));
    }

    // -- sharecli_health_check_status ----------------------------------------
    out.push_str("# HELP sharecli_health_check_status 1 if health check is passing, 0 otherwise\n");
    out.push_str("# TYPE sharecli_health_check_status gauge\n");
    for (proc_name, status) in health_map.iter() {
        let name = escape_label_value(proc_name);
        let val = if status.healthy { 1u8 } else { 0u8 };
        out.push_str(&format!("sharecli_health_check_status{{process=\"{}\"}} {}\n", name, val));
    }

    out
}

async fn build_snapshot() -> serde_json::Value {
    let pool = ProcessPool::new();
    let procs = pool.list().await;
    let summaries: Vec<_> = procs
        .iter()
        .map(|p| {
            json!({
                "pid": p.pid,
                "name": p.name,
                "cmd": p.cmd,
                "memory_mb": p.memory_mb,
                "project": p.project,
                "harness": p.harness,
                "start_time": p.start_time,
            })
        })
        .collect();

    json!({ "processes": summaries })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use sharecli_fleet::thermal::ThermalLevel;

    use super::*;
    use crate::serve_lock::{decide, Decision, OnConflict, ServeInfo, ServeState};

    // --- serve_lock decision tests ---

    fn running_live(url: &str) -> ServeState {
        ServeState::Running {
            info: ServeInfo {
                pid: std::process::id(),
                service: "sharecli".into(),
                url: url.into(),
                started_at_unix: 1,
            },
            stale: false,
        }
    }

    fn running_stale() -> ServeState {
        ServeState::Running {
            info: ServeInfo {
                pid: u32::MAX,
                service: "sharecli".into(),
                url: "http://127.0.0.1:9000".into(),
                started_at_unix: 1,
            },
            stale: true,
        }
    }

    #[test]
    fn free_state_always_serves() {
        assert_eq!(decide(&ServeState::Free, OnConflict::Abort), Decision::Serve);
        assert_eq!(decide(&ServeState::Free, OnConflict::Attach), Decision::Serve);
        assert_eq!(decide(&ServeState::Free, OnConflict::Replace), Decision::Serve);
        assert_eq!(decide(&ServeState::Free, OnConflict::Prompt), Decision::Serve);
    }

    #[test]
    fn stale_running_serves_regardless_of_policy() {
        let stale = running_stale();
        assert_eq!(decide(&stale, OnConflict::Abort), Decision::Serve);
        assert_eq!(decide(&stale, OnConflict::Attach), Decision::Serve);
    }

    #[test]
    fn live_running_abort_policy_aborts() {
        let live = running_live("http://127.0.0.1:9000");
        assert_eq!(decide(&live, OnConflict::Abort), Decision::Abort);
        assert_eq!(decide(&live, OnConflict::Prompt), Decision::Abort);
    }

    #[test]
    fn live_running_attach_policy_attaches() {
        let live = running_live("http://127.0.0.1:9000");
        assert_eq!(decide(&live, OnConflict::Attach), Decision::Attach);
    }

    #[test]
    fn live_running_replace_policy_replaces() {
        let live = running_live("http://127.0.0.1:9000");
        assert_eq!(decide(&live, OnConflict::Replace), Decision::Replace);
    }

    // --- parse_pressure_level unit tests ---

    #[test]
    fn parse_pressure_green() {
        assert_eq!(parse_pressure_level(1), Some(ThermalLevel::Green));
    }

    #[test]
    fn parse_pressure_yellow() {
        assert_eq!(parse_pressure_level(2), Some(ThermalLevel::Yellow));
    }

    #[test]
    fn parse_pressure_red() {
        assert_eq!(parse_pressure_level(4), Some(ThermalLevel::Red));
    }

    #[test]
    fn parse_pressure_unknown_returns_none() {
        assert_eq!(parse_pressure_level(0), None);
        assert_eq!(parse_pressure_level(3), None);
        assert_eq!(parse_pressure_level(5), None);
        assert_eq!(parse_pressure_level(255), None);
    }

    // --- ThermalEvent serialization tests ---

    #[test]
    fn thermal_event_warning_serializes() {
        let evt = ThermalEvent::ThermalWarning { pressure: 2 };
        let s = serde_json::to_string(&evt).unwrap();
        assert!(s.contains("\"event\":\"thermal_warning\""));
        assert!(s.contains("\"pressure\":2"));
    }

    #[test]
    fn thermal_event_critical_serializes() {
        let evt = ThermalEvent::ThermalCritical { pressure: 4 };
        let s = serde_json::to_string(&evt).unwrap();
        assert!(s.contains("\"event\":\"thermal_critical\""));
        assert!(s.contains("\"pressure\":4"));
    }

    // --- healthz / readyz JSON contract (no full server spin) ---

    #[test]
    fn healthz_json_is_ok() {
        let v = healthz_json();
        assert_eq!(v["status"], "ok");
    }

    /// Soft C08 live corpus: fixtures with `expect.health` must match `healthz_json`.
    #[test]
    fn corpus_health_fixtures_match_healthz() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/eval/corpus/scenarios");
        let mut checked = 0u32;
        for entry in std::fs::read_dir(&dir).expect("corpus scenarios dir") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).expect("read scenario");
            let data: serde_json::Value = serde_json::from_str(&raw).expect("parse scenario");
            let Some(health) = data.pointer("/expect/health").and_then(|v| v.as_str()) else {
                continue;
            };
            assert_eq!(
                healthz_json()["status"].as_str(),
                Some(health),
                "fixture {} expect.health mismatch",
                path.display()
            );
            checked += 1;
        }
        assert!(checked >= 1, "expected at least one corpus fixture with expect.health");
    }

    /// Soft C08 live corpus: fixtures with `expect.gate` must match thermal `gate_decision`.
    #[test]
    fn corpus_thermal_gate_fixtures_match_gate_decision() {
        use sharecli_thermal_tui::gate_decision;

        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/eval/corpus/scenarios");
        let mut checked = 0u32;
        for entry in std::fs::read_dir(&dir).expect("corpus scenarios dir") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).expect("read scenario");
            let data: serde_json::Value = serde_json::from_str(&raw).expect("parse scenario");
            let Some(expect_gate) = data.pointer("/expect/gate").and_then(|v| v.as_str()) else {
                continue;
            };
            let level = match data.get("thermal").and_then(|v| v.as_str()) {
                Some("red") => ThermalLevel::Red,
                Some("yellow") => ThermalLevel::Yellow,
                Some("green") | None => ThermalLevel::Green,
                Some(other) => panic!("fixture {}: unknown thermal {:?}", path.display(), other),
            };
            assert_eq!(
                gate_decision(level),
                expect_gate,
                "fixture {} expect.gate mismatch for {:?}",
                path.display(),
                level
            );
            if let Some(spawn_allowed) =
                data.pointer("/expect/spawn_allowed").and_then(|v| v.as_bool())
            {
                let allowed = expect_gate != "DENY";
                assert_eq!(
                    spawn_allowed,
                    allowed,
                    "fixture {} expect.spawn_allowed inconsistent with gate",
                    path.display()
                );
            }
            checked += 1;
        }
        assert!(checked >= 1, "expected at least one corpus fixture with expect.gate");
    }

    #[test]
    fn readyz_ok_while_serving() {
        let (status, body) = readyz_response(false);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
    }

    #[test]
    fn readyz_unavailable_after_shutdown() {
        let (status, body) = readyz_response(true);
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["status"], "unavailable");
    }

    // --- render_prometheus_metrics unit tests ---

    fn make_process(name: &str, memory_mb: u64) -> crate::runtime::ProcessInfo {
        crate::runtime::ProcessInfo {
            pid: 1234,
            name: name.to_string(),
            cmd: vec!["fake".to_string()],
            memory_mb,
            start_time: 0,
            project: None,
            harness: None,
        }
    }

    fn make_health(healthy: bool, failures: u32) -> crate::health_check::HealthStatus {
        crate::health_check::HealthStatus {
            healthy,
            consecutive_failures: failures,
            last_error: None,
            last_check: std::time::Instant::now(),
        }
    }

    #[test]
    fn prometheus_output_contains_required_metric_names() {
        let processes = vec![make_process("myapp", 256)];
        let mut hmap = std::collections::HashMap::new();
        hmap.insert("myapp".to_string(), make_health(true, 0));
        let out = render_prometheus_metrics(&processes, &hmap);
        assert!(out.contains("sharecli_process_memory_mb"), "missing memory metric");
        assert!(out.contains("sharecli_process_up"), "missing process_up metric");
        assert!(
            out.contains("sharecli_health_check_consecutive_failures"),
            "missing failures metric"
        );
        assert!(out.contains("sharecli_health_check_status"), "missing health_check_status metric");
    }

    #[test]
    fn prometheus_process_up_reflects_health_status() {
        let processes = vec![make_process("svc", 64)];

        let mut healthy_map = std::collections::HashMap::new();
        healthy_map.insert("svc".to_string(), make_health(true, 0));
        let healthy_out = render_prometheus_metrics(&processes, &healthy_map);
        assert!(
            healthy_out.contains("sharecli_process_up{process=\"svc\"} 1"),
            "healthy process should have process_up=1"
        );

        let mut unhealthy_map = std::collections::HashMap::new();
        unhealthy_map.insert("svc".to_string(), make_health(false, 3));
        let unhealthy_out = render_prometheus_metrics(&processes, &unhealthy_map);
        assert!(
            unhealthy_out.contains("sharecli_process_up{process=\"svc\"} 0"),
            "unhealthy process should have process_up=0"
        );
    }

    #[test]
    fn prometheus_gauge_values_match_inputs() {
        let processes = vec![make_process("worker", 512)];
        let mut hmap = std::collections::HashMap::new();
        hmap.insert("worker".to_string(), make_health(true, 7));
        let out = render_prometheus_metrics(&processes, &hmap);
        assert!(out.contains("sharecli_process_memory_mb{process=\"worker\"} 512"));
        assert!(out.contains("sharecli_health_check_consecutive_failures{process=\"worker\"} 7"));
    }

    #[test]
    fn prometheus_label_escaping_handles_special_chars() {
        // Process name with double-quote and backslash
        let processes = vec![make_process("my\"app\\test", 128)];
        let hmap = std::collections::HashMap::new();
        let out = render_prometheus_metrics(&processes, &hmap);
        // Escaped label value should appear; raw chars must not appear unescaped inside quotes
        assert!(
            out.contains(r#"process="my\"app\\test""#),
            "label value not properly escaped: {out}"
        );
    }

    #[test]
    fn prometheus_process_up_defaults_to_1_when_no_health_check() {
        // Process present in pool but NOT in health_store → should be up=1
        let processes = vec![make_process("orphan", 32)];
        let hmap = std::collections::HashMap::new(); // empty
        let out = render_prometheus_metrics(&processes, &hmap);
        assert!(
            out.contains("sharecli_process_up{process=\"orphan\"} 1"),
            "process without health check should default to up=1"
        );
    }

    // --- broadcast channel test ---

    #[tokio::test]
    async fn thermal_broadcast_delivers_to_subscriber() {
        let (tx, mut rx) = broadcast::channel::<ThermalEvent>(8);
        tx.send(ThermalEvent::ThermalWarning { pressure: 2 }).unwrap();
        let received = rx.recv().await.unwrap();
        matches!(received, ThermalEvent::ThermalWarning { pressure: 2 });
    }
}
