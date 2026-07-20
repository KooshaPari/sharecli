use std::path::{Path, PathBuf};

use harness_native::find_real;
use harness_native::strategies::{execute, ExecRequest, RuleOpts};

// FR: FR-003 — native harness strategy coverage lift

#[cfg(unix)]
fn success_cmd() -> PathBuf {
    PathBuf::from("/bin/true")
}

#[cfg(windows)]
fn success_cmd() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\cmd.exe")
}

#[cfg(windows)]
fn success_args() -> Vec<String> {
    vec!["/C".into(), "exit".into(), "0".into()]
}

#[cfg(unix)]
fn success_args() -> Vec<String> {
    Vec::new()
}

fn exec(strategy: &str, opts: RuleOpts) -> Result<i32, String> {
    let cmd = success_cmd();
    let args = success_args();
    execute(ExecRequest {
        strategy,
        harness_home: Path::new("."),
        real_cmd: &cmd,
        cmd_name: cmd.file_name().and_then(|s| s.to_str()).unwrap_or("cmd"),
        subcmd: "",
        cache_key: "",
        opts: &opts,
        args: &args,
        agent_name: "coverage-lift",
    })
}

#[test]
fn find_real_ignores_empty_cache_entries() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let proxy = tmp.path().join("proxy");
    std::fs::create_dir_all(&proxy).expect("proxy dir");
    std::fs::write(proxy.join(".fake.real"), "\n").expect("cache file");

    let found = find_real::find_real(&proxy, tmp.path(), None, "fake");

    assert!(found.is_none());
}

#[test]
fn find_real_reads_valid_cache_entry() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let proxy = tmp.path().join("proxy");
    std::fs::create_dir_all(&proxy).expect("proxy dir");
    let real = success_cmd();
    std::fs::write(proxy.join(format!(".{}.real", "tool")), real.display().to_string())
        .expect("cache file");

    let found = find_real::find_real(&proxy, tmp.path(), None, "tool");
    assert_eq!(found, Some(real));
}

#[test]
fn rule_options_have_safe_defaults() {
    let opts = RuleOpts::default();

    assert_eq!(opts.ttl, 0);
    assert_eq!(opts.max_concurrent, 0);
    assert!(!opts.jobserver_borrow);
}

#[test]
fn batch_strategy_reports_contract_error() {
    let opts = RuleOpts::default();
    let args = Vec::<String>::new();
    let result = execute(ExecRequest {
        strategy: "batch",
        harness_home: Path::new("."),
        real_cmd: Path::new("missing-command"),
        cmd_name: "missing-command",
        subcmd: "",
        cache_key: "",
        opts: &opts,
        args: &args,
        agent_name: "test",
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("batch strategy"));
}

#[test]
fn passthrough_and_coalesce_strategies_run_success_cmd() {
    for strategy in ["passthrough", "coalesce", "cache"] {
        let code = exec(strategy, RuleOpts::default()).expect(strategy);
        assert_eq!(code, 0, "{strategy} must succeed");
    }
}

#[test]
fn queue_and_priority_queue_strategies_run_success_cmd() {
    for strategy in ["queue", "priority_queue"] {
        let code = exec(strategy, RuleOpts::default()).expect(strategy);
        assert_eq!(code, 0, "{strategy} must succeed");
    }
}

#[test]
fn debounce_strategy_waits_then_runs() {
    let mut opts = RuleOpts::default();
    opts.debounce_ms = 1;
    let code = exec("debounce", opts).expect("debounce");
    assert_eq!(code, 0);
}

#[test]
fn retry_strategy_eventually_succeeds() {
    let mut opts = RuleOpts::default();
    opts.retry_max = 1;
    opts.retry_backoff_ms = 1;
    opts.retry_jitter = 0.0;
    let code = exec("retry", opts).expect("retry");
    assert_eq!(code, 0);
}

#[test]
fn incremental_strategy_streams_success_cmd() {
    let code = exec("incremental", RuleOpts::default()).expect("incremental");
    assert_eq!(code, 0);
}

#[test]
fn circuit_breaker_strategy_runs_success_cmd() {
    let mut opts = RuleOpts::default();
    opts.breaker_threshold = 3;
    opts.breaker_window = 60;
    let code = exec("circuit_breaker", opts).expect("circuit_breaker");
    assert_eq!(code, 0);
}

#[test]
fn throttle_load_balance_and_jobserver_strategies_run() {
    for strategy in ["resource_throttle", "load_balance", "jobserver"] {
        let code = exec(strategy, RuleOpts::default()).expect(strategy);
        assert_eq!(code, 0, "{strategy} must succeed");
    }
}

#[test]
fn speculative_proactive_warm_and_causal_order_run() {
    for strategy in ["speculative", "proactive_warm", "causal_order"] {
        let code = exec(strategy, RuleOpts::default()).expect(strategy);
        assert_eq!(code, 0, "{strategy} must succeed");
    }
}

#[test]
fn unknown_strategy_falls_back_to_coalesce() {
    let code = exec("not-a-real-strategy", RuleOpts::default()).expect("fallback");
    assert_eq!(code, 0);
}

#[test]
fn missing_command_surfaces_not_found_error() {
    let opts = RuleOpts::default();
    let args = Vec::<String>::new();
    let err = execute(ExecRequest {
        strategy: "queue",
        harness_home: Path::new("."),
        real_cmd: Path::new("definitely-missing-command-xyz"),
        cmd_name: "missing",
        subcmd: "",
        cache_key: "",
        opts: &opts,
        args: &args,
        agent_name: "test",
    })
    .unwrap_err();
    assert!(err.contains("not found") || err.contains("failed"));
}
