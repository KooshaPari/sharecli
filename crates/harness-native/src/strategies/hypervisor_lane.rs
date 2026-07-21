use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sharecli_core::{Hypervisor, HypervisorConfig, SpawnRequest};
use sharecli_ipc::CoalesceCache;

use super::RuleOpts;

/// Harness cache root for Hypervisor coalesce / queue state.
pub fn hypervisor_cache_root(harness_home: &Path) -> PathBuf {
    harness_home.join("var").join("sharecli-hypervisor")
}

/// Map rules.conf options into a [`HypervisorConfig`] (FR-008 AC-008.17).
pub fn config_from_rule_opts(harness_home: &Path, opts: &RuleOpts) -> HypervisorConfig {
    let cache_root = hypervisor_cache_root(harness_home);
    let coalesce_ttl = if opts.ttl == 0 {
        CoalesceCache::DEFAULT_TTL
    } else {
        Duration::from_secs(opts.ttl)
    };
    let queue_max_concurrent = if opts.max_concurrent == 0 {
        1
    } else {
        opts.max_concurrent as usize
    };
    HypervisorConfig {
        cache_root: cache_root.clone(),
        queue_root: cache_root.join("queue"),
        queue_max_concurrent,
        coalesce_ttl,
        coalesce_debounce: Duration::from_millis(opts.debounce_ms),
    }
}

/// Build a production hypervisor from harness home + rules.conf options.
pub fn build_hypervisor(harness_home: &Path, opts: &RuleOpts) -> Hypervisor {
    Hypervisor::from_config(config_from_rule_opts(harness_home, opts))
}

pub fn spawn_request(
    real_cmd: &Path,
    args: &[&str],
    opts: &RuleOpts,
) -> Result<SpawnRequest, String> {
    let cwd = env::current_dir().map_err(|e| format!("harness spawn: cwd: {e}"))?;
    let mut argv: Vec<String> = vec![real_cmd.to_string_lossy().into_owned()];
    argv.extend(args.iter().map(|s| (*s).to_string()));
    Ok(SpawnRequest::from_operator(argv, cwd, vec![], Some(&opts.priority)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sharecli_core::{FakeThermalGate, ThermalDecision};
    use std::sync::Arc;
    use tempfile::TempDir;

    /// FR-008 / AC-008.17 — rules.conf ttl/debounce/max_concurrent map into HypervisorConfig.
    #[test]
    fn rule_opts_plumb_hypervisor_config() {
        let tmp = TempDir::new().expect("tempdir");
        let opts = RuleOpts {
            ttl: 42,
            debounce_ms: 99,
            max_concurrent: 3,
            ..RuleOpts::default()
        };
        let cfg = config_from_rule_opts(tmp.path(), &opts);
        assert_eq!(cfg.coalesce_ttl, Duration::from_secs(42));
        assert_eq!(cfg.coalesce_debounce, Duration::from_millis(99));
        assert_eq!(cfg.queue_max_concurrent, 3);

        let hv = Hypervisor::from_config_with_gate(
            cfg,
            Arc::new(FakeThermalGate::new(ThermalDecision::Allow)),
        );
        assert_eq!(hv.coalesce_ttl(), Duration::from_secs(42));
        assert_eq!(hv.coalesce_debounce(), Duration::from_millis(99));
        assert_eq!(hv.queue_max_concurrent(), 3);
    }
}
