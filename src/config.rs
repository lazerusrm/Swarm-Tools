use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main configuration structure for Swarm-Tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmConfig {
    /// General settings.
    pub general: GeneralConfig,
    /// Role-aware routing settings.
    pub role_routing: RoleRoutingConfig,
    /// Trajectory compression settings.
    pub trajectory_compression: TrajectoryCompressionConfig,
    /// Resource allocation settings.
    pub resource_allocation: ResourceAllocationConfig,
    /// Codified reasoning settings.
    pub reasoning: ReasoningConfig,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            role_routing: RoleRoutingConfig::default(),
            trajectory_compression: TrajectoryCompressionConfig::default(),
            resource_allocation: ResourceAllocationConfig::default(),
            reasoning: ReasoningConfig::default(),
        }
    }
}

/// General configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    /// Default context budget in tokens.
    pub default_context_budget: usize,
    /// Maximum parallel agents.
    pub max_parallel_agents: usize,
    /// Context threshold percentage (70-90 recommended).
    pub context_threshold: f64,
    /// Variance threshold for token usage.
    pub variance_threshold: f64,
    /// Enable debug mode.
    pub debug: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_context_budget: 200_000,
            max_parallel_agents: 3,
            context_threshold: 80.0,
            variance_threshold: 2.0,
            debug: false,
        }
    }
}

/// Role-aware routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleRoutingConfig {
    /// Enable role-based filtering.
    pub enabled: bool,
    /// Default relevance threshold (0.0-1.0).
    pub relevance_threshold: f64,
    /// Recency multiplier for last 10% of messages.
    pub recency_multiplier_max: f64,
    /// Minimum impact score to preserve.
    pub min_impact_score: f64,
}

impl Default for RoleRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            relevance_threshold: 0.3,
            recency_multiplier_max: 2.0,
            min_impact_score: 0.5,
        }
    }
}

/// Trajectory compression configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrajectoryCompressionConfig {
    /// Enable trajectory compression.
    pub enabled: bool,
    /// Minimum steps before compression triggers.
    pub min_steps: usize,
    /// Token threshold for compression.
    pub token_threshold: usize,
    /// Preserve threshold for high-impact entries.
    pub preserve_threshold: f64,
    /// Maximum summaries to keep.
    pub max_summaries: usize,
    /// Enable superseded detection.
    pub detect_superseded: bool,
    /// Enable redundant filtering.
    pub filter_redundant: bool,
}

impl Default for TrajectoryCompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_steps: 18,
            token_threshold: 25000,
            preserve_threshold: 0.7,
            max_summaries: 10,
            detect_superseded: true,
            filter_redundant: true,
        }
    }
}

/// Resource allocation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceAllocationConfig {
    /// Enable automatic resource allocation.
    pub enabled: bool,
    /// Safety reserve percentage (15% recommended).
    pub safety_reserve_percent: f64,
    /// Minimum budget per agent.
    pub min_per_agent: u32,
    /// Auto-reduce low contributor budget.
    pub auto_reduce_low_contrib: bool,
    /// Reduction percentage for low contributors.
    pub low_contrib_reduction_percent: f64,
    /// Contribution threshold for pruning (0.0-1.0).
    pub pruning_contribution_threshold: f64,
    /// Turns before considering agent for pruning.
    pub pruning_turns_threshold: u32,
    /// Imbalance threshold (20% variance).
    pub imbalance_threshold: f64,
}

impl Default for ResourceAllocationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            safety_reserve_percent: 15.0,
            min_per_agent: 10_000,
            auto_reduce_low_contrib: false,
            low_contrib_reduction_percent: 20.0,
            pruning_contribution_threshold: 0.3,
            pruning_turns_threshold: 5,
            imbalance_threshold: 0.20,
        }
    }
}

/// Codified reasoning configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningConfig {
    /// Enable structured JSON planning.
    pub enabled: bool,
    /// Maximum steps in a plan.
    pub max_plan_steps: u32,
    /// Priority calculation weight for contribution.
    pub contribution_weight: f64,
    /// Priority calculation weight for urgency.
    pub urgency_weight: f64,
    /// Enable plan summarization.
    pub enable_summarization: bool,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_plan_steps: 10,
            contribution_weight: 0.7,
            urgency_weight: 0.3,
            enable_summarization: true,
        }
    }
}

/// Loads configuration from a JSON file.
///
/// # Arguments
/// * `path` - Path to the JSON configuration file.
///
/// # Returns
/// `SwarmConfig` on success, or default config on error.
pub fn load_config_from_json(path: impl AsRef<Path>) -> SwarmConfig {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config JSON: {}. Using defaults.", e);
                SwarmConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file: {}. Using defaults.", e);
            SwarmConfig::default()
        }
    }
}

/// Loads configuration from a YAML file.
///
/// # Arguments
/// * `path` - Path to the YAML configuration file.
///
/// # Returns
/// `SwarmConfig` on success, or default config on error.
#[cfg(feature = "yaml")]
pub fn load_config_from_yaml(path: impl AsRef<Path>) -> SwarmConfig {
    match fs::read_to_string(path) {
        Ok(content) => match serde_yaml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config YAML: {}. Using defaults.", e);
                SwarmConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file: {}. Using defaults.", e);
            SwarmConfig::default()
        }
    }
}

/// Saves configuration to a JSON file.
///
/// # Arguments
/// * `config` - The configuration to save.
/// * `path` - Path to save the JSON file.
pub fn save_config_to_json(config: &SwarmConfig, path: impl AsRef<Path>) -> Result<(), String> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write config: {}", e))
}

/// Merges two configurations, with `other` overriding `default` values.
///
/// # Arguments
/// * `default` - Base configuration.
/// * `other` - Override configuration.
///
/// # Returns
/// Merged configuration.
pub fn merge_configs(default: SwarmConfig, other: &SwarmConfig) -> SwarmConfig {
    SwarmConfig {
        general: if other.general != GeneralConfig::default() {
            other.general.clone()
        } else {
            default.general
        },
        role_routing: if other.role_routing != RoleRoutingConfig::default() {
            other.role_routing.clone()
        } else {
            default.role_routing
        },
        trajectory_compression: if other.trajectory_compression
            != TrajectoryCompressionConfig::default()
        {
            other.trajectory_compression.clone()
        } else {
            default.trajectory_compression
        },
        resource_allocation: if other.resource_allocation != ResourceAllocationConfig::default() {
            other.resource_allocation.clone()
        } else {
            default.resource_allocation
        },
        reasoning: if other.reasoning != ReasoningConfig::default() {
            other.reasoning.clone()
        } else {
            default.reasoning
        },
    }
}

/// Generates example configuration JSON.
pub fn generate_example_config() -> String {
    r#"{
  "general": {
    "default_context_budget": 200000,
    "max_parallel_agents": 3,
    "context_threshold": 80.0,
    "variance_threshold": 2.0,
    "debug": false
  },
  "role_routing": {
    "enabled": true,
    "relevance_threshold": 0.3,
    "recency_multiplier_max": 2.0,
    "min_impact_score": 0.5
  },
  "trajectory_compression": {
    "enabled": true,
    "min_steps": 18,
    "token_threshold": 25000,
    "preserve_threshold": 0.7,
    "max_summaries": 10,
    "detect_superseded": true,
    "filter_redundant": true
  },
  "resource_allocation": {
    "enabled": true,
    "safety_reserve_percent": 15.0,
    "min_per_agent": 10000,
    "auto_reduce_low_contrib": false,
    "low_contrib_reduction_percent": 20.0,
    "pruning_contribution_threshold": 0.3,
    "pruning_turns_threshold": 5,
    "imbalance_threshold": 0.20
  },
  "reasoning": {
    "enabled": true,
    "max_plan_steps": 10,
    "contribution_weight": 0.7,
    "urgency_weight": 0.3,
    "enable_summarization": true
  }
}"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_example_config() {
        let example = generate_example_config();
        assert!(example.contains("role_routing"));
        assert!(example.contains("trajectory_compression"));
    }

    #[test]
    fn test_merge_configs() {
        let default = SwarmConfig::default();
        let override_config = SwarmConfig {
            general: GeneralConfig {
                default_context_budget: 100000,
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = merge_configs(default, &override_config);
        assert_eq!(merged.general.default_context_budget, 100000);
    }

    #[test]
    fn test_default_config_values() {
        let config = SwarmConfig::default();
        assert_eq!(config.general.default_context_budget, 200000);
        assert!(config.trajectory_compression.enabled);
        assert!(config.resource_allocation.enabled);
    }
}
