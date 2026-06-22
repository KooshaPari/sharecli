use std::path::Path;

use harness_native::find_real;
use harness_native::strategies::{execute, ExecRequest, RuleOpts};

// FR: native harness ownership boundary

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
