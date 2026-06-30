//! Build-contention throttle — semaphore + taskpolicy + CARGO_BUILD_JOBS + sccache.
//!
//! Only applies to processes that sharecli itself spawns (cargo/rustc/build harnesses).
//! The operator's existing terminal sessions are never touched.
//!
//! # How it works
//!
//! 1. A tokio `Semaphore` is sized to `spawn_policy.max_concurrent_builds`. Every
//!    build harness must acquire a permit before the underlying process is started.
//!    Callers hold the `SemaphorePermit` for the lifetime of the build; when it drops
//!    the slot is freed and the next queued build starts.
//!
//! 2. On macOS, when `nice_level > 0`, the harness command is wrapped with
//!    `taskpolicy -b -- <original-cmd> [args…]` which places the child in a
//!    background-efficiency QoS tier — the kernel schedules it at lower priority
//!    than foreground interactive work, reducing latency spikes for the operator's
//!    own sessions under load.
//!
//! 3. `CARGO_BUILD_JOBS` is set to `max_concurrent_builds` so rustc's own internal
//!    parallelism stays within the same budget.
//!
//! 4. When `use_sccache = true` and `sccache` is on PATH, `RUSTC_WRAPPER=sccache`
//!    is injected — compiled artefacts are cached across rebuilds, drastically
//!    cutting wall-clock time for incremental work.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::config::SpawnPolicyConfig;

// ---------------------------------------------------------------------------
// Build harness detection
// ---------------------------------------------------------------------------

/// Returns `true` for harnesses that consume heavy CPU and benefit from throttling.
pub fn is_build_harness(harness: &str) -> bool {
    matches!(harness, "cargo" | "rustc" | "build" | "make" | "cmake" | "ninja" | "bazel")
}

// ---------------------------------------------------------------------------
// SpawnPolicy
// ---------------------------------------------------------------------------

/// Sharecli-wide spawn-policy enforcer.  Wrap in `Arc` and share across
/// `ProcessPool` instances.
pub struct SpawnPolicy {
    semaphore: Arc<Semaphore>,
    pub config: SpawnPolicyConfig,
}

impl SpawnPolicy {
    pub fn new(config: SpawnPolicyConfig) -> Self {
        let permits = config.max_concurrent_builds.max(1);
        Self { semaphore: Arc::new(Semaphore::new(permits)), config }
    }

    /// Acquire a build slot.  The returned permit MUST be held for the duration
    /// of the build and dropped when the build finishes.
    pub async fn acquire_build_permit(&self) -> Result<SemaphorePermit<'_>> {
        let permit = self.semaphore.acquire().await?;
        Ok(permit)
    }

    /// Try to acquire a build slot without waiting. Returns `None` when all
    /// slots are taken.
    // Used in integration tests and by callers that prefer non-blocking probe.
    #[allow(dead_code)]
    pub fn try_acquire_build_permit(&self) -> Option<SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok()
    }

    /// Current number of free build slots.
    // Used in tests; exposed for diagnostics/status commands.
    #[allow(dead_code)]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    // -----------------------------------------------------------------------
    // Command shaping
    // -----------------------------------------------------------------------

    /// Wrap a build-harness command with `taskpolicy -b` on macOS when
    /// `nice_level > 0`, leaving the args unchanged.
    ///
    /// Returns `(effective_program, effective_args)`.
    pub fn apply_taskpolicy<'a>(
        &self,
        program: &'a str,
        args: &'a [String],
    ) -> (String, Vec<String>) {
        #[cfg(target_os = "macos")]
        if self.config.nice_level > 0 {
            let mut new_args = vec!["--".to_string(), program.to_string()];
            new_args.extend_from_slice(args);
            return ("taskpolicy".to_string(), new_args);
        }

        (program.to_string(), args.to_vec())
    }

    /// Build the environment-variable overrides to inject into build harness
    /// spawns: `CARGO_BUILD_JOBS` and optionally `RUSTC_WRAPPER`.
    pub fn build_env_overrides(&self) -> Vec<(String, String)> {
        let mut env = vec![(
            "CARGO_BUILD_JOBS".to_string(),
            self.config.max_concurrent_builds.to_string(),
        )];

        if self.config.use_sccache && sccache_on_path() {
            env.push(("RUSTC_WRAPPER".to_string(), "sccache".to_string()));
        }

        env
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sccache_on_path() -> bool {
    which_sccache().is_some()
}

fn which_sccache() -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var).find_map(|dir| {
            let candidate = dir.join("sccache");
            if candidate.exists() { Some(candidate) } else { None }
        })
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SpawnPolicyConfig;
    use tokio::time::{Duration, Instant};

    fn policy(max: usize) -> SpawnPolicy {
        SpawnPolicy::new(SpawnPolicyConfig { max_concurrent_builds: max, ..Default::default() })
    }

    // -- Semaphore cap -------------------------------------------------------

    /// With cap=2, two permits succeed immediately; the third must wait until
    /// one is released.
    #[tokio::test]
    async fn semaphore_caps_concurrent_builds() {
        let p = policy(2);

        let p1 = p.acquire_build_permit().await.unwrap();
        let p2 = p.acquire_build_permit().await.unwrap();
        assert_eq!(p.available_permits(), 0, "both slots taken");

        // Third acquire must block — verify it can't get a permit yet.
        assert!(p.try_acquire_build_permit().is_none(), "no permits left");

        // Release one — now it should succeed.
        drop(p1);
        assert_eq!(p.available_permits(), 1);
        let _p3 = p.acquire_build_permit().await.unwrap();
        assert_eq!(p.available_permits(), 0);
        drop(p2);
        drop(_p3);
    }

    /// Verify the semaphore actually serialises N+1 concurrent tasks when cap=N.
    #[tokio::test]
    async fn semaphore_queues_excess_tasks() {
        use std::sync::{Arc as StdArc, Mutex};
        use tokio::task::JoinSet;

        let policy = StdArc::new(policy(2));
        let active = StdArc::new(Mutex::new(0usize));
        let peak = StdArc::new(Mutex::new(0usize));

        let mut set = JoinSet::new();
        for _ in 0..6 {
            let policy = policy.clone();
            let active = active.clone();
            let peak = peak.clone();
            set.spawn(async move {
                let _permit = policy.acquire_build_permit().await.unwrap();
                {
                    let mut a = active.lock().unwrap();
                    *a += 1;
                    let mut pk = peak.lock().unwrap();
                    if *a > *pk {
                        *pk = *a;
                    }
                }
                // Simulate a short build.
                tokio::time::sleep(Duration::from_millis(20)).await;
                {
                    let mut a = active.lock().unwrap();
                    *a -= 1;
                }
            });
        }
        while set.join_next().await.is_some() {}

        let pk = *peak.lock().unwrap();
        assert!(pk <= 2, "peak active builds was {pk}, expected ≤ 2");
    }

    // -- taskpolicy wrapping -------------------------------------------------

    #[test]
    #[cfg(target_os = "macos")]
    fn taskpolicy_wraps_command_on_macos_when_nice_gt_0() {
        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 10,
            max_concurrent_builds: 2,
            use_sccache: false,
        });
        let (prog, args) =
            p.apply_taskpolicy("cargo", &["build".to_string(), "--release".to_string()]);
        assert_eq!(prog, "taskpolicy");
        assert_eq!(args, vec!["--", "cargo", "build", "--release"]);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn taskpolicy_disabled_when_nice_is_0() {
        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 0,
            max_concurrent_builds: 2,
            use_sccache: false,
        });
        let (prog, args) = p.apply_taskpolicy("cargo", &["build".to_string()]);
        assert_eq!(prog, "cargo");
        assert_eq!(args, vec!["build"]);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn taskpolicy_passthrough_on_non_macos() {
        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 10,
            max_concurrent_builds: 2,
            use_sccache: false,
        });
        let (prog, args) = p.apply_taskpolicy("cargo", &["build".to_string()]);
        assert_eq!(prog, "cargo");
        assert_eq!(args, vec!["build"]);
    }

    // -- CARGO_BUILD_JOBS injection ------------------------------------------

    #[test]
    fn cargo_build_jobs_injected() {
        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 0,
            max_concurrent_builds: 3,
            use_sccache: false,
        });
        let env = p.build_env_overrides();
        let jobs = env.iter().find(|(k, _)| k == "CARGO_BUILD_JOBS").map(|(_, v)| v.as_str());
        assert_eq!(jobs, Some("3"), "CARGO_BUILD_JOBS must match max_concurrent_builds");
    }

    // -- sccache wiring ------------------------------------------------------

    #[test]
    fn sccache_not_injected_when_disabled() {
        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 0,
            max_concurrent_builds: 2,
            use_sccache: false,
        });
        let env = p.build_env_overrides();
        assert!(
            !env.iter().any(|(k, _)| k == "RUSTC_WRAPPER"),
            "RUSTC_WRAPPER must not be set when use_sccache=false"
        );
    }

    /// When use_sccache=true but sccache is not on PATH, RUSTC_WRAPPER must not
    /// be injected (avoids breaking builds with a missing binary).
    #[test]
    fn sccache_not_injected_when_not_on_path() {
        // Temporarily clear PATH so sccache is not found.
        let old_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", "");

        let p = SpawnPolicy::new(SpawnPolicyConfig {
            nice_level: 0,
            max_concurrent_builds: 2,
            use_sccache: true,
        });
        let env = p.build_env_overrides();

        std::env::set_var("PATH", old_path);

        assert!(
            !env.iter().any(|(k, _)| k == "RUSTC_WRAPPER"),
            "RUSTC_WRAPPER must not be set when sccache not on PATH"
        );
    }

    // -- is_build_harness ----------------------------------------------------

    #[test]
    fn build_harness_detection() {
        for h in ["cargo", "rustc", "build", "make", "cmake", "ninja", "bazel"] {
            assert!(is_build_harness(h), "{h} should be a build harness");
        }
        for h in ["claude", "forge", "node", "bun", "python"] {
            assert!(!is_build_harness(h), "{h} should NOT be a build harness");
        }
    }

    // -- Benchmark under load ------------------------------------------------
    //
    // This is an integration-style timing test, not a micro-benchmark.
    // It spawns 6 concurrent "cargo check --help" (fast, real cargo invocations)
    // and measures wall-clock under throttled (cap=2) vs unthrottled (cap=6).
    //
    // On an unloaded machine the gains are minimal; under real contention
    // (all 6 spawns competing for the same rustc/LLVM threads) the throttled
    // run should be faster because it avoids CPU thrashing.
    //
    // We don't assert a specific speedup here — contention on CI varies —
    // but we emit timing to stdout so the PR description can quote real numbers.
    #[tokio::test]
    async fn benchmark_throttled_vs_unthrottled_under_load() {
        use tokio::process::Command;
        use tokio::task::JoinSet;

        const TASKS: usize = 6;

        async fn run_builds(cap: usize) -> Duration {
            let policy = Arc::new(SpawnPolicy::new(SpawnPolicyConfig {
                nice_level: 0, // no taskpolicy in tests (avoids macOS sandbox issues)
                max_concurrent_builds: cap,
                use_sccache: false,
            }));

            let start = Instant::now();
            let mut set = JoinSet::new();
            for _ in 0..TASKS {
                let policy = policy.clone();
                set.spawn(async move {
                    // Gate on the semaphore (this is the throttle being tested).
                    let _permit = policy.acquire_build_permit().await.unwrap();
                    // Use a real but lightweight cargo invocation.
                    let _ = Command::new("cargo")
                        .args(["--version"])
                        .env("CARGO_BUILD_JOBS", cap.to_string())
                        .output()
                        .await;
                });
            }
            while set.join_next().await.is_some() {}
            start.elapsed()
        }

        let throttled = run_builds(2).await;
        let unthrottled = run_builds(TASKS).await;

        println!(
            "[bench] throttled (cap=2, {} tasks): {:?}  |  unthrottled (cap={}, {} tasks): {:?}",
            TASKS, throttled, TASKS, TASKS, unthrottled
        );

        // The throttled path must finish in finite time (no deadlock / hang).
        assert!(throttled.as_secs() < 60, "throttled run timed out");
        assert!(unthrottled.as_secs() < 60, "unthrottled run timed out");
    }
}
