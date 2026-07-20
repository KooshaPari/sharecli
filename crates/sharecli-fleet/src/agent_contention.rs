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
}
