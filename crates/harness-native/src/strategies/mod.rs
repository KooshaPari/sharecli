//! Execution strategies: coalesce, queue, debounce, retry, etc.

mod batch;
mod causal_order;
mod circuit_breaker;
mod coalesce;
mod debounce;
mod incremental;
mod jobserver;
mod load_balance;
mod proactive_warm;
mod process;
mod queue;
mod resource_throttle;
mod retry;
mod speculative;

use std::path::Path;

/// Rule options parsed from rules.conf
#[derive(Debug, Clone, Default)]
pub struct RuleOpts {
    pub ttl: u64,
    pub debounce_ms: u64,
    pub error_ttl: u64,
    pub max_concurrent: u32,
    pub priority: String,
    pub cache_key: String,
    pub batch_key: String,
    pub causal_domain: String,
    pub breaker_threshold: u32,
    pub breaker_window: u64,
    pub breaker_cooldown: u64,
    pub retry_max: u32,
    pub retry_backoff_ms: u64,
    pub retry_jitter: f64,
    pub jobserver_auth: String,
    pub jobserver_tokens: u32,
    pub jobserver_borrow: bool,
    pub stale_threshold: u64,
    pub semantic: bool,
}

/// Execute a strategy. Returns exit code.
pub struct ExecRequest<'a> {
    pub strategy: &'a str,
    pub harness_home: &'a Path,
    pub real_cmd: &'a Path,
    pub cmd_name: &'a str,
    pub subcmd: &'a str,
    pub cache_key: &'a str,
    pub opts: &'a RuleOpts,
    pub args: &'a [String],
    pub agent_name: &'a str,
}

pub fn execute(req: ExecRequest<'_>) -> Result<i32, String> {
    let full_args: Vec<&str> = req.args.iter().map(|s| s.as_str()).collect();

    match req.strategy {
        "passthrough" => coalesce::run(req.real_cmd, &full_args),
        "coalesce" | "cache" => coalesce::run(req.real_cmd, &full_args),
        "queue" | "priority_queue" => queue::run(req.real_cmd, &full_args),
        "debounce" => debounce::run(req.real_cmd, req.opts.debounce_ms, &full_args),
        "retry" => retry::run(
            req.real_cmd,
            req.opts.retry_max,
            req.opts.retry_backoff_ms,
            req.opts.retry_jitter,
            &full_args,
        ),
        "incremental" => incremental::run(req.real_cmd, &full_args),
        "circuit_breaker" => circuit_breaker::run(
            req.real_cmd,
            req.opts.breaker_threshold,
            req.opts.breaker_window,
            &full_args,
        ),
        "resource_throttle" => resource_throttle::run(req.real_cmd, &full_args),
        "jobserver" => jobserver::run(req.real_cmd, &full_args),
        "load_balance" => load_balance::run(req.real_cmd, &full_args),
        "speculative" => speculative::run(req.real_cmd, &full_args),
        "proactive_warm" => proactive_warm::run(req.real_cmd, &full_args),
        "batch" => batch::run(req.real_cmd, &full_args),
        "causal_order" => causal_order::run(req.real_cmd, &full_args),
        _ => {
            let _ = (
                req.harness_home,
                req.cmd_name,
                req.subcmd,
                req.cache_key,
                req.opts,
                req.agent_name,
            );
            coalesce::run(req.real_cmd, &full_args)
        }
    }
}
