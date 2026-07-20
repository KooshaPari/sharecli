//! FR-008 — Speculative Coalesce / Debounce / Queue
//! FR: FR-008
//!
//! AC-008.1 command_key stability
//! AC-008.2 CoalesceCache with_lock runs once
//! AC-008.3 thermal Refuse gates before coalesce/spawn
//! AC-008.4 second identical Hypervisor run hits cache
//! AC-008.5 CoalesceCache TTL treats stale entries as miss
//! AC-008.6 debounce window waits/shares before re-run
//! (Mesh membership ACs live under FR-010.)

use sharecli_core::{
    FakeThermalGate, Hypervisor, SpawnRequest, ThermalDecision, THERMAL_MAX_RETRIES,
};
use sharecli_ipc::{command_key, CachedResult, CoalesceCache};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// FR-008 / AC-008.1 — identical argv/cwd/env → same key; different argv → different.
#[test]
fn fr008_command_key_stable() {
    let cwd = Path::new("/tmp");
    let a = command_key(&["echo".into(), "x".into()], cwd, &[]);
    let b = command_key(&["echo".into(), "x".into()], cwd, &[]);
    let c = command_key(&["echo".into(), "y".into()], cwd, &[]);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

/// FR-008 / AC-008.2 — with_lock executes the miss path once per key.
#[test]
fn fr008_with_lock_runs_once() {
    let dir = TempDir::new().expect("tempdir");
    let cache = CoalesceCache::new(dir.path());
    let key = command_key(&["true".into()], dir.path(), &[]);
    let hits = AtomicU32::new(0);

    let first = cache
        .with_lock(&key, || {
            hits.fetch_add(1, Ordering::SeqCst);
            Ok(CachedResult {
                exit_code: 0,
                stdout: b"once".to_vec(),
                stderr: vec![],
            })
        })
        .expect("first lock");
    let second = cache
        .with_lock(&key, || {
            hits.fetch_add(1, Ordering::SeqCst);
            Ok(CachedResult {
                exit_code: 1,
                stdout: b"again".to_vec(),
                stderr: vec![],
            })
        })
        .expect("second lock");

    assert_eq!(hits.load(Ordering::SeqCst), 1, "miss path MUST run once");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(second.exit_code, 0);
}

/// FR-008 / AC-008.3 — Refuse thermal gate fails loudly before speculative work.
#[tokio::test(start_paused = true)]
async fn fr008_thermal_gate_before_coalesce() {
    let dir = TempDir::new().expect("tempdir");
    let gate = Arc::new(FakeThermalGate::new(ThermalDecision::Refuse));
    let hv = Hypervisor::with_thermal_gate(dir.path(), gate);

    #[cfg(unix)]
    let argv = vec!["echo".to_string(), "should-not-run".to_string()];
    #[cfg(windows)]
    let argv = vec![
        "cmd".to_string(),
        "/C".to_string(),
        "echo".to_string(),
        "should-not-run".to_string(),
    ];

    let err = hv
        .run(SpawnRequest {
            argv,
            cwd: dir.path().to_path_buf(),
            env: vec![],
        })
        .await
        .expect_err("Refuse MUST err after retries");

    let msg = err.to_string();
    assert!(
        msg.contains("thermally throttled"),
        "error must mention thermally throttled, got {msg}; max_retries={THERMAL_MAX_RETRIES}"
    );
}

/// FR-008 / AC-008.4 — second identical Allow run is served from coalesce cache.
#[tokio::test]
async fn fr008_second_run_from_cache() {
    let dir = TempDir::new().expect("tempdir");
    let gate = Arc::new(FakeThermalGate::new(ThermalDecision::Allow));
    let hv = Hypervisor::with_thermal_gate(dir.path(), gate);

    #[cfg(unix)]
    let argv = vec!["echo".to_string(), "coalesce".to_string()];
    #[cfg(windows)]
    let argv = vec![
        "cmd".to_string(),
        "/C".to_string(),
        "echo".to_string(),
        "coalesce".to_string(),
    ];

    let req = SpawnRequest {
        argv,
        cwd: dir.path().to_path_buf(),
        env: vec![],
    };
    let first = hv.run(req.clone()).await.expect("first");
    assert!(!first.from_cache);
    let second = hv.run(req).await.expect("second");
    assert!(second.from_cache, "mesh coalesce MUST hit cache on replay");
    assert_eq!(first.stdout, second.stdout);
}

/// FR-008 / AC-008.5 — lookup treats entries older than TTL as a miss.
#[test]
fn fr008_ttl_stale_entry_is_miss() {
    let dir = TempDir::new().expect("tempdir");
    let ttl = Duration::from_millis(80);
    let cache = CoalesceCache::with_ttl(dir.path(), ttl);
    let key = command_key(&["ttl-probe".into()], dir.path(), &[]);

    cache
        .store(
            &key,
            &CachedResult {
                exit_code: 0,
                stdout: b"fresh".to_vec(),
                stderr: vec![],
            },
        )
        .expect("store");

    let hit = cache.lookup(&key).expect("lookup").expect("fresh hit");
    assert_eq!(hit.stdout, b"fresh");

    thread::sleep(ttl + Duration::from_millis(40));

    assert!(
        cache.lookup(&key).expect("stale lookup").is_none(),
        "entry older than TTL MUST be treated as miss (AC-008.5)"
    );

    let hits = AtomicU32::new(0);
    let rerun = cache
        .with_lock(&key, || {
            hits.fetch_add(1, Ordering::SeqCst);
            Ok(CachedResult {
                exit_code: 0,
                stdout: b"rerun".to_vec(),
                stderr: vec![],
            })
        })
        .expect("with_lock after TTL");
    assert_eq!(hits.load(Ordering::SeqCst), 1, "stale miss MUST re-run");
    assert_eq!(rerun.stdout, b"rerun");
}

/// FR-008 / AC-008.6 — debounce waits then shares a result completed in-window.
#[test]
fn fr008_debounce_waits_and_shares() {
    let dir = TempDir::new().expect("tempdir");
    let debounce = Duration::from_millis(120);
    let cache = CoalesceCache::with_options(dir.path(), Duration::from_secs(300), debounce);
    let key = command_key(&["debounce-probe".into()], dir.path(), &[]);
    let hits = Arc::new(AtomicU32::new(0));

    let cache_bg = CoalesceCache::with_options(dir.path(), Duration::from_secs(300), debounce);
    let key_bg = key.clone();
    let hits_bg = Arc::clone(&hits);
    let producer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(40));
        cache_bg
            .store(
                &key_bg,
                &CachedResult {
                    exit_code: 0,
                    stdout: b"shared".to_vec(),
                    stderr: vec![],
                },
            )
            .expect("bg store");
        hits_bg.fetch_add(1, Ordering::SeqCst);
    });

    let result = cache
        .with_lock(&key, || {
            hits.fetch_add(1, Ordering::SeqCst);
            Ok(CachedResult {
                exit_code: 1,
                stdout: b"should-not-run".to_vec(),
                stderr: vec![],
            })
        })
        .expect("debounced with_lock");

    producer.join().expect("producer join");

    assert_eq!(result.stdout, b"shared", "debounce MUST share in-window result");
    assert_eq!(
        result.exit_code, 0,
        "shared result MUST come from producer, not miss path"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "miss path MUST NOT run when debounce shares a recent store (AC-008.6)"
    );
}
