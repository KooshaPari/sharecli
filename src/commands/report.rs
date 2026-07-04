//! Fleet analytics report command.
//!
//! `sharecli report [--format text|json] [--watch <secs>] [--sort memory|name]`
//! prints a fleet analytics snapshot to stdout.  With `--watch N` it clears the
//! terminal and re-renders every N seconds until Ctrl-C.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::runtime::{ProcessInfo, ProcessPool};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Output format for `sharecli report`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReportFormat {
    #[default]
    Text,
    Json,
}

/// Sort key for top-consumers list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SortBy {
    /// Descending memory usage (default).
    #[default]
    Memory,
    /// Ascending process name (alphabetical).
    Name,
}

impl std::str::FromStr for SortBy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "name" => Ok(Self::Name),
            other => anyhow::bail!("unknown sort key '{}'; expected 'memory' or 'name'", other),
        }
    }
}

impl std::str::FromStr for ReportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => anyhow::bail!("unknown format '{}'; expected 'text' or 'json'", other),
        }
    }
}

/// Per-project breakdown included in the report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectBreakdown {
    pub count: usize,
    pub memory_mb: u64,
}

/// Summary of one of the top memory consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopConsumer {
    pub pid: u32,
    pub name: String,
    pub project: Option<String>,
    pub memory_mb: u64,
}

/// Full analytics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetReport {
    /// Unix timestamp (seconds) when the snapshot was taken.
    pub timestamp: u64,
    /// Approximate daemon uptime in seconds (based on earliest process start time).
    pub uptime_seconds: u64,
    /// Total number of tracked processes.
    pub total_processes: usize,
    /// Sum of `memory_mb` across all tracked processes.
    pub total_memory_mb: u64,
    /// Per-project count + memory.
    pub by_project: HashMap<String, ProjectBreakdown>,
    /// Top-5 memory consumers (descending).
    pub top_consumers: Vec<TopConsumer>,
    /// Thermal pressure level string ("GREEN" / "YELLOW" / "RED" or "UNAVAILABLE").
    pub thermal_pressure: String,
}

// ---------------------------------------------------------------------------
// Aggregation logic (pure function — easy to unit-test)
// ---------------------------------------------------------------------------

/// Sort a mutable slice of [`TopConsumer`] in-place according to `sort`.
///
/// - `SortBy::Memory` — descending by `memory_mb` (highest first).
/// - `SortBy::Name`   — ascending by `name` (alphabetical).
pub fn sort_consumers(consumers: &mut [TopConsumer], sort: &SortBy) {
    match sort {
        SortBy::Memory => consumers.sort_by_key(|c| Reverse(c.memory_mb)),
        SortBy::Name => consumers.sort_by(|a, b| a.name.cmp(&b.name)),
    }
}

/// Build a [`FleetReport`] from a slice of process snapshots.
///
/// `thermal` is the current thermal pressure string (caller supplies it so
/// the function stays sync and testable without hitting sysfs).
/// `sort` controls the order of `top_consumers`.
pub fn build_report(processes: &[ProcessInfo], thermal: &str, sort: &SortBy) -> FleetReport {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let total_memory_mb: u64 = processes.iter().map(|p| p.memory_mb).sum();

    // Per-project breakdown
    let mut by_project: HashMap<String, ProjectBreakdown> = HashMap::new();
    for p in processes {
        let key = p.project.clone().unwrap_or_else(|| "<untagged>".to_string());
        let entry = by_project.entry(key).or_insert(ProjectBreakdown { count: 0, memory_mb: 0 });
        entry.count += 1;
        entry.memory_mb += p.memory_mb;
    }

    // Top-5 consumers, ordered by `sort`
    let mut candidates: Vec<TopConsumer> = processes
        .iter()
        .map(|p| TopConsumer {
            pid: p.pid,
            name: p.name.clone(),
            project: p.project.clone(),
            memory_mb: p.memory_mb,
        })
        .collect();
    // Always collect top-5 by memory first so we get the "most relevant" 5,
    // then re-sort by the requested key.
    candidates.sort_by_key(|c| Reverse(c.memory_mb));
    candidates.truncate(5);
    sort_consumers(&mut candidates, sort);
    let top_consumers = candidates;

    // Uptime: time since the earliest process started (0 if no processes)
    let earliest_start = processes.iter().map(|p| p.start_time).filter(|&t| t > 0).min();
    let uptime_seconds = earliest_start.map(|t| now.saturating_sub(t)).unwrap_or(0);

    FleetReport {
        timestamp: now,
        uptime_seconds,
        total_processes: processes.len(),
        total_memory_mb,
        by_project,
        top_consumers,
        thermal_pressure: thermal.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn render_text(report: &FleetReport) {
    println!("=== Fleet Analytics Report ===");
    println!("Timestamp:       {}", report.timestamp);
    println!("Uptime:          {} s", report.uptime_seconds);
    println!("Thermal:         {}", report.thermal_pressure);
    println!("Total processes: {}", report.total_processes);
    println!("Total memory:    {} MB", report.total_memory_mb);

    println!("\n--- Per-Project Breakdown ---");
    println!("{:<25} {:>8} {:>12}", "PROJECT", "PROCS", "MEM (MB)");
    println!("{}", "-".repeat(47));
    let mut projects: Vec<(&String, &ProjectBreakdown)> = report.by_project.iter().collect();
    projects.sort_by(|a, b| a.0.cmp(b.0));
    for (name, bd) in &projects {
        println!("{:<25} {:>8} {:>12}", name, bd.count, bd.memory_mb);
    }

    if !report.top_consumers.is_empty() {
        println!("\n--- Top Memory Consumers ---");
        println!("{:>8} {:<25} {:<20} {:>12}", "PID", "NAME", "PROJECT", "MEM (MB)");
        println!("{}", "-".repeat(67));
        for tc in &report.top_consumers {
            println!(
                "{:>8} {:<25} {:<20} {:>12}",
                tc.pid,
                tc.name,
                tc.project.as_deref().unwrap_or("-"),
                tc.memory_mb
            );
        }
    }
}

fn render_json(report: &FleetReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Render one snapshot and print it according to `format`.
async fn render_once(format: &ReportFormat, sort: &SortBy) -> Result<()> {
    let pool = ProcessPool::new();
    let processes = pool.list().await;

    // Best-effort thermal level via sharecli-fleet
    let thermal = {
        use sharecli_fleet::thermal::ThermalGovernor;
        let gov = ThermalGovernor::new();
        match gov.poll() {
            Ok(level) => format!("{level:?}"),
            Err(_) => "UNAVAILABLE".to_string(),
        }
    };

    let report = build_report(&processes, &thermal, sort);

    match format {
        ReportFormat::Text => render_text(&report),
        ReportFormat::Json => render_json(&report)?,
    }

    Ok(())
}

/// Run the report command.
///
/// - `watch`: if `Some(n)`, clear terminal and re-render every `n` seconds
///   until Ctrl-C; if `None`, run once and exit.
/// - `sort`: controls ordering of the top-consumers section.
pub async fn run(format: ReportFormat, watch: Option<u64>, sort: SortBy) -> Result<()> {
    match watch {
        None => render_once(&format, &sort).await,
        Some(interval_secs) => {
            loop {
                // Clear terminal (ANSI: erase screen + move cursor to top-left)
                print!("\x1b[2J\x1b[H");

                render_once(&format, &sort).await?;

                println!("\n[watch] Refreshing every {interval_secs}s — press Ctrl-C to stop.");

                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(interval_secs)) => {},
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nExiting watch mode.");
                        break;
                    }
                }
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proc(
        pid: u32,
        name: &str,
        project: Option<&str>,
        memory_mb: u64,
        start_time: u64,
    ) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            cmd: vec![],
            memory_mb,
            start_time,
            project: project.map(String::from),
            harness: None,
        }
    }

    #[test]
    fn test_build_report_empty() {
        let report = build_report(&[], "GREEN", &SortBy::Memory);
        assert_eq!(report.total_processes, 0);
        assert_eq!(report.total_memory_mb, 0);
        assert!(report.by_project.is_empty());
        assert!(report.top_consumers.is_empty());
        assert_eq!(report.thermal_pressure, "GREEN");
    }

    #[test]
    fn test_build_report_aggregation() {
        let procs = vec![
            make_proc(1, "cargo", Some("alpha"), 300, 1_000_000),
            make_proc(2, "bun", Some("alpha"), 100, 1_000_100),
            make_proc(3, "node", Some("beta"), 200, 1_000_200),
            make_proc(4, "forge", None, 50, 1_000_300),
        ];
        let report = build_report(&procs, "YELLOW", &SortBy::Memory);

        assert_eq!(report.total_processes, 4);
        assert_eq!(report.total_memory_mb, 650);

        let alpha = report.by_project.get("alpha").expect("alpha missing");
        assert_eq!(alpha.count, 2);
        assert_eq!(alpha.memory_mb, 400);

        let beta = report.by_project.get("beta").expect("beta missing");
        assert_eq!(beta.count, 1);
        assert_eq!(beta.memory_mb, 200);

        let untagged = report.by_project.get("<untagged>").expect("untagged missing");
        assert_eq!(untagged.count, 1);
        assert_eq!(untagged.memory_mb, 50);
    }

    #[test]
    fn test_top_consumers_order_and_limit() {
        let procs: Vec<ProcessInfo> =
            (0u32..8).map(|i| make_proc(i, "proc", None, (i as u64 + 1) * 100, 0)).collect();
        let report = build_report(&procs, "GREEN", &SortBy::Memory);

        assert_eq!(report.top_consumers.len(), 5);
        // First element must be the highest memory consumer
        assert_eq!(report.top_consumers[0].memory_mb, 800);
        // Must be in descending order
        for w in report.top_consumers.windows(2) {
            assert!(w[0].memory_mb >= w[1].memory_mb);
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let procs = vec![make_proc(10, "claude", Some("proj-a"), 512, 1_700_000_000)];
        let report = build_report(&procs, "RED", &SortBy::Memory);
        let json = serde_json::to_string(&report).expect("serialize");
        let back: FleetReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.total_processes, report.total_processes);
        assert_eq!(back.total_memory_mb, report.total_memory_mb);
        assert_eq!(back.thermal_pressure, "RED");
        let pa = back.by_project.get("proj-a").unwrap();
        assert_eq!(pa.count, 1);
        assert_eq!(pa.memory_mb, 512);
    }

    // ------------------------------------------------------------------
    // Sort logic tests
    // ------------------------------------------------------------------

    #[test]
    fn test_sort_by_name_ascending() {
        let procs = vec![
            make_proc(1, "zebra", None, 500, 0),
            make_proc(2, "alpha", None, 100, 0),
            make_proc(3, "mango", None, 300, 0),
        ];
        let report = build_report(&procs, "GREEN", &SortBy::Name);
        let names: Vec<&str> = report.top_consumers.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mango", "zebra"]);
    }

    #[test]
    fn test_sort_by_memory_descending() {
        let procs = vec![
            make_proc(1, "low", None, 50, 0),
            make_proc(2, "high", None, 900, 0),
            make_proc(3, "mid", None, 400, 0),
        ];
        let report = build_report(&procs, "GREEN", &SortBy::Memory);
        let mems: Vec<u64> = report.top_consumers.iter().map(|c| c.memory_mb).collect();
        assert_eq!(mems, vec![900, 400, 50]);
    }

    #[test]
    fn test_sort_consumers_in_place() {
        let mut consumers = vec![
            TopConsumer { pid: 1, name: "zebra".into(), project: None, memory_mb: 10 },
            TopConsumer { pid: 2, name: "alpha".into(), project: None, memory_mb: 50 },
            TopConsumer { pid: 3, name: "mango".into(), project: None, memory_mb: 30 },
        ];
        sort_consumers(&mut consumers, &SortBy::Name);
        assert_eq!(consumers[0].name, "alpha");
        assert_eq!(consumers[1].name, "mango");
        assert_eq!(consumers[2].name, "zebra");

        sort_consumers(&mut consumers, &SortBy::Memory);
        assert_eq!(consumers[0].memory_mb, 50);
        assert_eq!(consumers[1].memory_mb, 30);
        assert_eq!(consumers[2].memory_mb, 10);
    }

    #[test]
    fn test_sort_by_from_str() {
        use std::str::FromStr;
        assert_eq!(SortBy::from_str("memory").unwrap(), SortBy::Memory);
        assert_eq!(SortBy::from_str("MEMORY").unwrap(), SortBy::Memory);
        assert_eq!(SortBy::from_str("name").unwrap(), SortBy::Name);
        assert_eq!(SortBy::from_str("NAME").unwrap(), SortBy::Name);
        assert!(SortBy::from_str("pid").is_err());
    }

    #[test]
    fn test_report_format_from_str() {
        use std::str::FromStr;
        assert_eq!(ReportFormat::from_str("text").unwrap(), ReportFormat::Text);
        assert_eq!(ReportFormat::from_str("TEXT").unwrap(), ReportFormat::Text);
        assert_eq!(ReportFormat::from_str("json").unwrap(), ReportFormat::Json);
        assert_eq!(ReportFormat::from_str("JSON").unwrap(), ReportFormat::Json);
        assert!(ReportFormat::from_str("xml").is_err());
    }
}
