use std::env;
use std::path::Path;

use sharecli_core::{Hypervisor, SpawnRequest};

use super::RuleOpts;

/// Harness cache root for Hypervisor coalesce / queue state.
fn hypervisor_cache_root(harness_home: &Path) -> std::path::PathBuf {
    harness_home.join("var").join("sharecli-hypervisor")
}

fn build_hypervisor(harness_home: &Path) -> Hypervisor {
    let cache_root = hypervisor_cache_root(harness_home);
    Hypervisor::new(cache_root)
}

fn spawn_request(
    real_cmd: &Path,
    args: &[&str],
    opts: &RuleOpts,
) -> Result<SpawnRequest, String> {
    let cwd = env::current_dir().map_err(|e| format!("harness queue: cwd: {e}"))?;
    let mut argv: Vec<String> = vec![real_cmd.to_string_lossy().into_owned()];
    argv.extend(args.iter().map(|s| (*s).to_string()));
    Ok(SpawnRequest::from_operator(argv, cwd, vec![], Some(&opts.priority)))
}

fn run_with_hypervisor(
    hv: &Hypervisor,
    real_cmd: &Path,
    cmd_name: &str,
    args: &[&str],
    opts: &RuleOpts,
) -> Result<i32, String> {
    let req = spawn_request(real_cmd, args, opts)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("harness queue: tokio runtime: {e}"))?;

    let outcome = rt
        .block_on(hv.run_queued(req, cmd_name))
        .map_err(|e| format!("harness queue: hypervisor: {e}"))?;

    Ok(outcome.exit_code)
}

/// Execute via Hypervisor SlotQueue lane (FR-008 AC-008.16).
pub fn run(
    harness_home: &Path,
    real_cmd: &Path,
    cmd_name: &str,
    args: &[&str],
    opts: &RuleOpts,
) -> Result<i32, String> {
    let hv = build_hypervisor(harness_home);
    run_with_hypervisor(&hv, real_cmd, cmd_name, args, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;
    use sharecli_core::{FakeThermalGate, ThermalDecision};
    use tempfile::TempDir;

    fn allow_hypervisor(dir: &Path) -> Hypervisor {
        Hypervisor::with_thermal_gate(
            hypervisor_cache_root(dir),
            Arc::new(FakeThermalGate::new(ThermalDecision::Allow)),
        )
    }

    /// FR-008 / AC-008.16 — harness `queue` strategy routes through Hypervisor (not raw spawn).
    #[test]
    fn queue_strategy_executes_via_hypervisor() {
        let tmp = TempDir::new().expect("tempdir");
        let opts = RuleOpts {
            priority: "normal".to_string(),
            ..RuleOpts::default()
        };
        let hv = allow_hypervisor(tmp.path());
        let code = run_with_hypervisor(
            &hv,
            Path::new("/bin/echo"),
            "echo",
            &["harness-queue-ok"],
            &opts,
        )
        .expect("queue strategy MUST succeed");
        assert_eq!(code, 0, "AC-008.16: harness queue MUST run via Hypervisor::run_queued");
    }

    /// FR-008 / AC-008.16 — `priority_queue` strategy shares Hypervisor queue lane.
    #[test]
    fn priority_queue_strategy_executes_via_hypervisor() {
        let tmp = TempDir::new().expect("tempdir");
        let opts = RuleOpts {
            priority: "high".to_string(),
            ..RuleOpts::default()
        };
        let hv = allow_hypervisor(tmp.path());
        let code = run_with_hypervisor(
            &hv,
            Path::new("/bin/echo"),
            "echo",
            &["priority-queue-ok"],
            &opts,
        )
        .expect("priority_queue strategy MUST succeed");
        assert_eq!(
            code, 0,
            "AC-008.16: harness priority_queue MUST run via Hypervisor::run_queued"
        );
    }
}
