use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpRoutingConfig {
    pub enabled: bool,
    pub role_tool_filters: Option<HashMap<String, Vec<String>>>,
    pub default_tools: Option<Vec<String>>,
}

impl Default for McpRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            role_tool_filters: Some(HashMap::from([
                (
                    "extractor".to_string(),
                    vec![
                        "read_file".to_string(),
                        "git_diff".to_string(),
                        "grep".to_string(),
                        "glob".to_string(),
                    ],
                ),
                (
                    "analyzer".to_string(),
                    vec![
                        "search_code".to_string(),
                        "browse_web".to_string(),
                        "grep".to_string(),
                        "analyze".to_string(),
                    ],
                ),
                (
                    "writer".to_string(),
                    vec![
                        "write_file".to_string(),
                        "edit_file".to_string(),
                        "create".to_string(),
                    ],
                ),
                (
                    "reviewer".to_string(),
                    vec![
                        "read_file".to_string(),
                        "grep".to_string(),
                        "check".to_string(),
                        "lint".to_string(),
                    ],
                ),
                (
                    "synthesizer".to_string(),
                    vec![
                        "summarize".to_string(),
                        "browse_web".to_string(),
                        "search".to_string(),
                    ],
                ),
                (
                    "tester".to_string(),
                    vec![
                        "run_test".to_string(),
                        "execute".to_string(),
                        "verify".to_string(),
                    ],
                ),
                (
                    "documenter".to_string(),
                    vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "generate".to_string(),
                    ],
                ),
                (
                    "optimizer".to_string(),
                    vec![
                        "profile".to_string(),
                        "analyze".to_string(),
                        "benchmark".to_string(),
                    ],
                ),
            ])),
            default_tools: Some(vec!["message".to_string(), "communication".to_string()]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelTieringConfig {
    pub enabled: bool,
    pub simple_haiku_threshold: u32,
    pub moderate_sonnet_threshold: u32,
    pub fallback_model: String,
    pub high_impact_boost_enabled: bool,
}

impl Default for ModelTieringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            simple_haiku_threshold: 1000,
            moderate_sonnet_threshold: 5000,
            fallback_model: "claude-opus-4-5-2025".to_string(),
            high_impact_boost_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    pub enabled: bool,
    pub auto_prune_enabled: bool,
    pub prune_threshold: f64,
    pub prune_over_turns: usize,
    pub auto_rebalance_on_prune: bool,
    pub min_active_agents: usize,
    pub prune_safety_margin: f64,
}

impl Default for SelfHealingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_prune_enabled: false,
            prune_threshold: 0.3,
            prune_over_turns: 5,
            auto_rebalance_on_prune: true,
            min_active_agents: 2,
            prune_safety_margin: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedConfigSettings {
    pub enabled: bool,
    pub config_dir: String,
    pub override_file: String,
}

impl Default for SharedConfigSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            config_dir: ".claude/swarm-tools".to_string(),
            override_file: "config_override.json".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_routing_config_defaults() {
        let config = McpRoutingConfig::default();
        assert!(config.enabled);
        assert!(config.role_tool_filters.is_some());
        assert!(config.default_tools.is_some());
    }

    #[test]
    fn test_model_tiering_config_defaults() {
        let config = ModelTieringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.simple_haiku_threshold, 1000);
        assert_eq!(config.moderate_sonnet_threshold, 5000);
        assert_eq!(config.fallback_model, "claude-opus-4-5-2025");
    }

    #[test]
    fn test_self_healing_config_defaults() {
        let config = SelfHealingConfig::default();
        assert!(config.enabled);
        assert!(!config.auto_prune_enabled);
        assert_eq!(config.prune_threshold, 0.3);
        assert_eq!(config.prune_over_turns, 5);
        assert_eq!(config.min_active_agents, 2);
    }

    #[test]
    fn test_shared_config_settings_defaults() {
        let config = SharedConfigSettings::default();
        assert!(config.enabled);
        assert_eq!(config.config_dir, ".claude/swarm-tools");
    }
}
