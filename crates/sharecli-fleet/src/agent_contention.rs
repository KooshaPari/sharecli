//! Host agent-count contention for thermal+agent gating (FR-011).
//!
//! When many coding agents run concurrently, speculative coalesce pressure rises
//! even if raw thermal readings are still Green. This module maps proc-scan agent
//! inventory size to escalation tiers consumed by [`sharecli_core::AgentAwareThermalGate`].

use crate::proc_scan::scan_host_agents;
use crate::thermal::ThermalLevel;

/// Thresholds for host agent inventory escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentContentionThresholds {
    /// At or above: escalate Allow → Warn (Hypervisor still proceeds).
    pub warn_at: usize,
    /// At or above: escalate to Refuse (Hypervisor back-pressures / errors).
    pub refuse_at: usize,
}

impl Default for AgentContentionThresholds {
    fn default() -> Self {
        Self {
            warn_at: 4,
            refuse_at: 8,
        }
    }
}

/// Contention tier derived from live agent count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentContentionTier {
    Ok,
    Warn,
    Refuse,
}

/// Map host agent count to a contention tier.
pub fn agent_contention_tier(
    count: usize,
    thresholds: AgentContentionThresholds,
) -> AgentContentionTier {
    if count >= thresholds.refuse_at {
        AgentContentionTier::Refuse
    } else if count >= thresholds.warn_at {
        AgentContentionTier::Warn
    } else {
        AgentContentionTier::Ok
    }
}

/// Live host agent inventory size (FR-006 proc scan).
pub fn count_host_agents() -> usize {
    scan_host_agents().len()
}

/// Operator-facing ADMIT/DENY label matching Hypervisor gate semantics.
///
/// Red thermal or agent Refuse tier → DENY; otherwise ADMIT (including Yellow
/// thermal and agent Warn tier — those still proceed with warnings).
pub fn effective_gate_decision(thermal: ThermalLevel, agent_count: usize) -> &'static str {
    if thermal == ThermalLevel::Red {
        return "DENY";
    }
    if agent_contention_tier(agent_count, AgentContentionThresholds::default())
        == AgentContentionTier::Refuse
    {
        return "DENY";
    }
    "ADMIT"
}

fn thermal_level_label(level: ThermalLevel) -> &'static str {
    match level {
        ThermalLevel::Green => "GREEN",
        ThermalLevel::Yellow => "YELLOW",
        ThermalLevel::Red => "RED",
    }
}

fn contention_tier_label(tier: AgentContentionTier) -> &'static str {
    match tier {
        AgentContentionTier::Ok => "OK",
        AgentContentionTier::Warn => "WARN",
        AgentContentionTier::Refuse => "REFUSE",
    }
}

/// Structured gate fields for JSON report / programmatic surfaces (FR-011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateStatusSnapshot {
    /// Thermal level label (`GREEN` / `YELLOW` / `RED`).
    pub thermal_pressure: String,
    /// Live proc-scan agent inventory size.
    pub detected_agents: usize,
    /// Agent contention tier label (`OK` / `WARN` / `REFUSE`).
    pub agent_contention: String,
    /// Effective ADMIT/DENY decision.
    pub gate_decision: String,
}

/// Build structured gate fields from live thermal level + agent inventory.
pub fn gate_status_snapshot(thermal: ThermalLevel, agent_count: usize) -> GateStatusSnapshot {
    let thresholds = AgentContentionThresholds::default();
    let tier = agent_contention_tier(agent_count, thresholds);
    GateStatusSnapshot {
        thermal_pressure: thermal_level_label(thermal).to_string(),
        detected_agents: agent_count,
        agent_contention: contention_tier_label(tier).to_string(),
        gate_decision: effective_gate_decision(thermal, agent_count).to_string(),
    }
}

/// Operator status section: thermal level, agent inventory, effective gate (FR-011).
pub fn format_gate_status_section(thermal: ThermalLevel, agent_count: usize) -> String {
    let thresholds = AgentContentionThresholds::default();
    let snap = gate_status_snapshot(thermal, agent_count);
    format!(
        "\n=== Thermal Gate (FR-011) ===\n\n\
         Thermal level: {}\n\
         Detected agents: {}\n\
         Agent contention: {} (warn>={}, refuse>={})\n\
         Gate decision: [{}]\n",
        snap.thermal_pressure,
        snap.detected_agents,
        snap.agent_contention,
        thresholds.warn_at,
        thresholds.refuse_at,
        snap.gate_decision,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_contention_tier_boundaries() {
        let t = AgentContentionThresholds {
            warn_at: 4,
            refuse_at: 8,
        };
        assert_eq!(agent_contention_tier(0, t), AgentContentionTier::Ok);
        assert_eq!(agent_contention_tier(3, t), AgentContentionTier::Ok);
        assert_eq!(agent_contention_tier(4, t), AgentContentionTier::Warn);
        assert_eq!(agent_contention_tier(7, t), AgentContentionTier::Warn);
        assert_eq!(agent_contention_tier(8, t), AgentContentionTier::Refuse);
    }

    #[test]
    fn effective_gate_decision_agent_refuse_denies_on_green() {
        assert_eq!(
            effective_gate_decision(ThermalLevel::Green, 8),
            "DENY",
            "agent refuse tier MUST deny even when thermal is Green"
        );
    }

    #[test]
    fn effective_gate_decision_agent_warn_still_admits() {
        assert_eq!(
            effective_gate_decision(ThermalLevel::Green, 4),
            "ADMIT",
            "agent warn tier MUST still ADMIT (Warn proceeds in Hypervisor)"
        );
    }

    #[test]
    fn format_gate_status_section_includes_decision() {
        let section = format_gate_status_section(ThermalLevel::Green, 8);
        assert!(section.contains("=== Thermal Gate (FR-011) ==="));
        assert!(section.contains("Thermal level: GREEN"));
        assert!(section.contains("Detected agents: 8"));
        assert!(section.contains("Agent contention: REFUSE"));
        assert!(section.contains("Gate decision: [DENY]"));
    }

    #[test]
    fn gate_status_snapshot_matches_section() {
        let snap = gate_status_snapshot(ThermalLevel::Yellow, 4);
        assert_eq!(snap.thermal_pressure, "YELLOW");
        assert_eq!(snap.detected_agents, 4);
        assert_eq!(snap.agent_contention, "WARN");
        assert_eq!(snap.gate_decision, "ADMIT");
    }
}
