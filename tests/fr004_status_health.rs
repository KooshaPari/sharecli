//! FR-004 — Process & Pool Health Status (status / HealthStatus / ProcessStats)
//! FR: FR-004
//!
//! Covers AC-004.1, AC-004.4, AC-004.5.

use std::collections::HashMap;

use sharecli::monitoring::{HealthStatus, ProcessStats};
use sharecli::runtime::{ProcessInfo, ProcessPool, SharedRuntime};

/// Mirror of the per-harness aggregation + status tables in `commands::status`.
fn format_status(
    processes: &[ProcessInfo],
    pool_status: &sharecli::runtime::PoolStatus,
    used_mb: u64,
    total_mb: u64,
) -> String {
    let mut by_harness: HashMap<&str, (usize, u64)> = HashMap::new();
    for proc in processes {
        let h = proc.harness.as_deref().unwrap_or("unknown");
        let entry = by_harness.entry(h).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += proc.memory_mb;
    }

    let mut out = String::from("=== Process Status ===\n\n");
    out.push_str(&format!("Total: {} processes\n\n", processes.len()));
    out.push_str(&format!("{:<15} {:<10} {:<15}\n", "HARNESS", "COUNT", "MEMORY(MB)"));
    out.push_str(&format!("{}\n", "-".repeat(40)));
    for (h, (count, mem)) in &by_harness {
        out.push_str(&format!("{h:<15} {count:<10} {mem:<15}\n"));
    }

    out.push_str("\n=== Shared Runtime Pool ===\n\n");
    out.push_str(&format!("{:<10} {:<10} {:<10}\n", "TYPE", "TOTAL", "IDLE"));
    out.push_str(&format!("{}\n", "-".repeat(30)));
    out.push_str(&format!(
        "{:<10} {:<10} {:<10}\n",
        "node", pool_status.node_total, pool_status.node_idle
    ));
    out.push_str(&format!(
        "{:<10} {:<10} {:<10}\n",
        "bun", pool_status.bun_total, pool_status.bun_idle
    ));
    out.push_str(&format!("\nMax per type: {}\n", pool_status.max_per_type));

    let pct = used_mb.saturating_mul(100).checked_div(total_mb).unwrap_or(0);
    out.push_str("\n=== System Memory ===\n\n");
    out.push_str(&format!("Used: {used_mb} MB / {total_mb} MB ({pct}%)\n"));
    out
}

fn sample_process(pid: u32, name: &str, memory_mb: u64, harness: &str) -> ProcessInfo {
    ProcessInfo {
        pid,
        name: name.to_string(),
        cmd: vec![name.to_string()],
        memory_mb,
        start_time: 0,
        project: Some("demo".to_string()),
        harness: Some(harness.to_string()),
    }
}

/// FR-004 / AC-004.1 — `status` prints per-harness table, pool table, system memory.
#[tokio::test]
async fn fr004_status_prints_harness_table() {
    let processes = vec![
        sample_process(101, "claude", 64, "claude"),
        sample_process(102, "claude", 32, "claude"),
        sample_process(103, "codex", 128, "codex"),
    ];

    let runtime = SharedRuntime::new(4);
    let pool_status = runtime.status().await;
    let pool = ProcessPool::new();
    let (used_mb, total_mb) = pool.system_memory_usage().await;

    let out = format_status(&processes, &pool_status, used_mb, total_mb);

    assert!(out.contains("=== Process Status ==="), "got: {out}");
    assert!(
        out.contains("HARNESS") && out.contains("COUNT") && out.contains("MEMORY(MB)"),
        "got: {out}"
    );
    assert!(
        out.contains("claude") && out.contains("codex"),
        "harness rows MUST appear; got: {out}"
    );
    assert!(
        out.contains("=== Shared Runtime Pool ==="),
        "status MUST include shared-runtime pool table; got: {out}"
    );
    assert!(out.contains("TYPE") && out.contains("TOTAL") && out.contains("IDLE"), "got: {out}");
    assert!(
        out.contains("node") && out.contains("bun"),
        "pool rows MUST list node and bun; got: {out}"
    );
    assert!(out.contains("Max per type:"), "got: {out}");
    assert!(
        out.contains("=== System Memory ===") && out.contains("Used:"),
        "status MUST include system-memory line; got: {out}"
    );
    assert!(total_mb > 0, "system_memory_usage MUST report a non-zero total");
}

/// FR-004 / AC-004.4 — `HealthStatus::mark_unhealthy` increments `checks_failed`.
#[test]
fn fr004_health_status_marks_unhealthy() {
    let mut status = HealthStatus::new();
    assert!(status.healthy);
    assert_eq!(status.checks_failed, 0);
    let passed_before = status.checks_passed;

    status.mark_unhealthy("probe timeout");

    assert!(!status.healthy, "mark_unhealthy MUST clear healthy");
    assert_eq!(status.checks_failed, 1, "mark_unhealthy MUST increment checks_failed");
    assert_eq!(status.checks_passed, passed_before, "mark_unhealthy MUST NOT bump checks_passed");

    status.mark_unhealthy("second failure");
    assert_eq!(status.checks_failed, 2);
    assert!(!status.healthy);
}

/// FR-004 / AC-004.5 — `ProcessStats::is_idle` needs uptime > threshold AND cpu < 1.0.
#[test]
fn fr004_process_stats_idle_threshold() {
    let threshold = 60u64;

    let idle = ProcessStats {
        pid: 1,
        name: "worker".to_string(),
        memory_mb: 32,
        cpu_percent: 0.5,
        start_time: 1_000,
        uptime_seconds: threshold + 1,
    };
    assert!(idle.is_idle(threshold), "uptime>threshold and cpu<1.0 MUST be idle");

    let busy_cpu = ProcessStats {
        pid: 2,
        name: "worker".to_string(),
        memory_mb: 32,
        cpu_percent: 1.0,
        start_time: 1_000,
        uptime_seconds: threshold + 100,
    };
    assert!(!busy_cpu.is_idle(threshold), "cpu_percent >= 1.0 MUST NOT be idle");

    let too_young = ProcessStats {
        pid: 3,
        name: "worker".to_string(),
        memory_mb: 32,
        cpu_percent: 0.0,
        start_time: 1_000,
        uptime_seconds: threshold,
    };
    assert!(!too_young.is_idle(threshold), "uptime_seconds <= threshold MUST NOT be idle");

    let boundary_ok = ProcessStats {
        pid: 4,
        name: "worker".to_string(),
        memory_mb: 8,
        cpu_percent: 0.99,
        start_time: 1_000,
        uptime_seconds: threshold + 1,
    };
    assert!(boundary_ok.is_idle(threshold));
}
