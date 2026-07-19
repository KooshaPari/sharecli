//! FR-007 — Resource & Syscall-Relevant Watch
//! FR: FR-007
//!
//! AC-007.1 thermal governor mock levels round-trip
//! AC-007.2 FakeThermalGate maps Allow/Warn/Refuse
//! AC-007.3 ProcessStats idle heuristic (CPU/MEM-relevant signal)

use sharecli::monitoring::ProcessStats;
use sharecli_core::{FakeThermalGate, ThermalDecision, ThermalGate};
use sharecli_fleet::{ThermalGovernor, ThermalLevel};

/// FR-007 / AC-007.1 — mock thermal levels are visible via poll.
#[test]
fn fr007_thermal_governor_mock_levels() {
    for level in [ThermalLevel::Green, ThermalLevel::Yellow, ThermalLevel::Red] {
        let gov = ThermalGovernor::with_mock(level);
        let got = gov.poll().expect("mock poll");
        assert_eq!(got, level);
    }
}

/// FR-007 / AC-007.2 — FakeThermalGate decisions are stable.
#[test]
fn fr007_fake_thermal_gate_maps_decisions() {
    assert_eq!(
        FakeThermalGate::new(ThermalDecision::Allow).check(),
        ThermalDecision::Allow
    );
    assert_eq!(
        FakeThermalGate::new(ThermalDecision::Warn).check(),
        ThermalDecision::Warn
    );
    assert_eq!(
        FakeThermalGate::new(ThermalDecision::Refuse).check(),
        ThermalDecision::Refuse
    );
}

/// FR-007 / AC-007.3 — idle heuristic encodes CPU + uptime watch signal.
#[test]
fn fr007_process_stats_idle_heuristic() {
    let idle = ProcessStats {
        pid: 1,
        name: "agent".into(),
        memory_mb: 64,
        cpu_percent: 0.1,
        start_time: 0,
        uptime_seconds: 120,
    };
    assert!(idle.is_idle(60), "low CPU + long uptime MUST be idle");

    let busy = ProcessStats {
        pid: 2,
        name: "agent".into(),
        memory_mb: 512,
        cpu_percent: 42.0,
        start_time: 0,
        uptime_seconds: 120,
    };
    assert!(!busy.is_idle(60), "high CPU MUST not be idle");
}
