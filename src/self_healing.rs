use crate::feature_config::SelfHealingConfig;
use crate::types::{AgentRole, TurnStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PruneDecision {
    Keep,
    Prune { reason: String },
    Hint { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunedAgentStats {
    pub agent_id: String,
    pub role: AgentRole,
    pub contribution_avg: f64,
    pub turns_active: usize,
    pub tokens_used: u32,
    pub reallocated_tokens: u32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceStats {
    pub pruned_agents: Vec<PrunedAgentStats>,
    pub reallocated_tokens: u32,
    pub boosted_agents: HashMap<String, u32>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingState {
    pub enabled: bool,
    pub agent_contributions: HashMap<String, f64>,
    pub agent_turns: HashMap<String, usize>,
    pub recent_contributions: HashMap<String, Vec<f64>>,
    pub total_reallocations: u32,
    pub total_prunes: usize,
}

pub struct SelfHealingManager {
    config: SelfHealingConfig,
    state: SelfHealingState,
}

impl SelfHealingManager {
    pub fn new() -> Self {
        Self::with_config(SelfHealingConfig::default())
    }

    pub fn with_config(config: SelfHealingConfig) -> Self {
        Self {
            config: config.clone(),
            state: SelfHealingState {
                enabled: config.enabled,
                agent_contributions: HashMap::new(),
                agent_turns: HashMap::new(),
                recent_contributions: HashMap::new(),
                total_reallocations: 0,
                total_prunes: 0,
            },
        }
    }

    pub fn check_pruning_candidate(
        &self,
        agent_id: &str,
        role: AgentRole,
        current_contribution: f64,
    ) -> PruneDecision {
        if !self.config.enabled || !self.config.auto_prune_enabled {
            return PruneDecision::Hint {
                message: "Auto-prune disabled".to_string(),
            };
        }

        let turns = self.state.agent_turns.get(agent_id).copied().unwrap_or(0);
        if turns < self.config.prune_over_turns {
            return PruneDecision::Hint {
                message: format!(
                    "Agent {} has only {} turns, need {}",
                    agent_id, turns, self.config.prune_over_turns
                ),
            };
        }

        let recent_contribs = self.state.recent_contributions.get(agent_id);
        if let Some(contribs) = recent_contribs {
            let avg: f64 = contribs.iter().sum::<f64>() / contribs.len() as f64;
            if avg < self.config.prune_threshold {
                return PruneDecision::Prune {
                    reason: format!(
                        "Average contribution {:.2} below threshold {:.2} over {} turns",
                        avg, self.config.prune_threshold, self.config.prune_over_turns
                    ),
                };
            }
        }

        PruneDecision::Keep
    }

    pub fn prune_agent(
        &mut self,
        agent_id: &str,
        role: AgentRole,
        current_contribution: f64,
        active_agent_count: usize,
        total_budget: u32,
    ) -> Result<Option<PrunedAgentStats>, String> {
        if active_agent_count <= self.config.min_active_agents {
            return Err(format!(
                "Cannot prune: {} active agents, minimum is {}",
                active_agent_count, self.config.min_active_agents
            ));
        }

        let turns = self.state.agent_turns.get(agent_id).copied().unwrap_or(0);
        let tokens_used = self.state.agent_contributions.len() as u32 * 1000;

        let avg_contrib = self
            .state
            .recent_contributions
            .get(agent_id)
            .map(|v| v.iter().sum::<f64>() / v.len() as f64)
            .unwrap_or(current_contribution);

        let per_agent_budget = total_budget / active_agent_count as u32;
        let reallocated = if self.config.auto_rebalance_on_prune {
            per_agent_budget
        } else {
            0
        };

        let stats = PrunedAgentStats {
            agent_id: agent_id.to_string(),
            role,
            contribution_avg: avg_contrib,
            turns_active: turns,
            tokens_used,
            reallocated_tokens: reallocated,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };

        self.state.agent_contributions.remove(agent_id);
        self.state.agent_turns.remove(agent_id);
        self.state.recent_contributions.remove(agent_id);
        self.state.total_prunes += 1;
        self.state.total_reallocations += reallocated;

        Ok(Some(stats))
    }

    pub fn record_contribution(&mut self, agent_id: &str, contribution: f64) {
        let recent = self
            .state
            .recent_contributions
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new);
        recent.push(contribution);
        if recent.len() > 10 {
            recent.remove(0);
        }

        self.state
            .agent_contributions
            .insert(agent_id.to_string(), contribution);
        let turns = self
            .state
            .agent_turns
            .entry(agent_id.to_string())
            .or_insert(0);
        *turns += 1;
    }

    pub fn get_state(&self) -> &SelfHealingState {
        &self.state
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.config.auto_prune_enabled
    }

    pub fn get_config(&self) -> &SelfHealingConfig {
        &self.config
    }
}

impl Default for SelfHealingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_disabled_by_default() {
        let manager = SelfHealingManager::new();
        let decision = manager.check_pruning_candidate("agent1", AgentRole::General, 0.2);
        match decision {
            PruneDecision::Hint { message } => assert!(message.contains("disabled")),
            _ => panic!("Expected Hint when disabled"),
        }
    }

    #[test]
    fn test_keep_above_threshold() {
        let mut config = SelfHealingConfig::default();
        config.enabled = true;
        config.auto_prune_enabled = true;
        config.prune_threshold = 0.3;
        config.prune_over_turns = 1;

        let mut manager = SelfHealingManager::with_config(config);
        manager.record_contribution("agent1", 0.5);

        let decision = manager.check_pruning_candidate("agent1", AgentRole::General, 0.5);
        assert_eq!(decision, PruneDecision::Keep);
    }

    #[test]
    fn test_prune_below_threshold() {
        let mut config = SelfHealingConfig::default();
        config.enabled = true;
        config.auto_prune_enabled = true;
        config.prune_threshold = 0.3;
        config.prune_over_turns = 1;

        let mut manager = SelfHealingManager::with_config(config);
        manager.record_contribution("agent1", 0.2);

        let decision = manager.check_pruning_candidate("agent1", AgentRole::General, 0.2);
        match decision {
            PruneDecision::Prune { reason } => {
                assert!(reason.contains("below threshold"));
            }
            _ => panic!("Expected Prune decision"),
        }
    }

    #[test]
    fn test_safety_floor_prevents_prune() {
        let config = SelfHealingConfig::default();
        let mut manager = SelfHealingManager::with_config(config);

        let result = manager.prune_agent("agent1", AgentRole::General, 0.2, 2, 100000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("minimum"));
    }

    #[test]
    fn test_record_contribution() {
        let mut manager = SelfHealingManager::new();
        manager.record_contribution("agent1", 0.7);

        let state = manager.get_state();
        assert_eq!(state.agent_contributions.get("agent1"), Some(&0.7));
        assert_eq!(state.agent_turns.get("agent1"), Some(&1));
    }

    #[test]
    fn test_recent_contributions_tracked() {
        let mut manager = SelfHealingManager::new();
        for i in 1..=5 {
            manager.record_contribution("agent1", 0.5 + i as f64 * 0.05);
        }

        let state = manager.get_state();
        let recent = state.recent_contributions.get("agent1").unwrap();
        assert_eq!(recent.len(), 5);
    }
}
