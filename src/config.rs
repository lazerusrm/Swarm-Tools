use serde::{Deserialize, Serialize};

/// Quality gate configuration for output scoring and refinement decisions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityGateConfig {
    pub enabled: bool,
    pub minimum_threshold: f64,
    pub impact_weight: f64,
    pub contribution_weight: f64,
    pub completeness_weight: f64,
    pub coherence_weight: f64,
    pub todo_penalty_weight: f64,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            minimum_threshold: 70.0,
            impact_weight: 0.30,
            contribution_weight: 0.25,
            completeness_weight: 0.20,
            coherence_weight: 0.15,
            todo_penalty_weight: 0.10,
        }
    }
}

/// Communication analyzer patterns configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunicationPatternsConfig {
    pub redundancy_patterns: Vec<RedundancyPatternConfig>,
    pub irrelevance_patterns: Vec<IrrelevancePatternConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedundancyPatternConfig {
    pub pattern: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IrrelevancePatternConfig {
    pub pattern: String,
    pub weight: f64,
}

impl Default for CommunicationPatternsConfig {
    fn default() -> Self {
        Self {
            redundancy_patterns: vec![
                RedundancyPatternConfig {
                    pattern: r"status:\s*working|in progress|proceeding".to_string(),
                    weight: 0.9,
                },
                RedundancyPatternConfig {
                    pattern: r"i am|i'm (working|proceeding|continuing)".to_string(),
                    weight: 0.8,
                },
                RedundancyPatternConfig {
                    pattern: r"continuing|proceeding with (task|work)".to_string(),
                    weight: 0.7,
                },
                RedundancyPatternConfig {
                    pattern: r"same (as|above|previous)".to_string(),
                    weight: 0.8,
                },
                RedundancyPatternConfig {
                    pattern: r"duplicate|duplicate copy|copy of".to_string(),
                    weight: 0.9,
                },
                RedundancyPatternConfig {
                    pattern: r"already (done|completed|finished)".to_string(),
                    weight: 0.85,
                },
                RedundancyPatternConfig {
                    pattern: r"no (change|updates|new information)".to_string(),
                    weight: 0.9,
                },
                RedundancyPatternConfig {
                    pattern: r"nothing (new|to report|additional)".to_string(),
                    weight: 0.9,
                },
            ],
            irrelevance_patterns: vec![
                IrrelevancePatternConfig {
                    pattern: r"acknowledged|ack|ok|understood|got it".to_string(),
                    weight: 0.95,
                },
                IrrelevancePatternConfig {
                    pattern: r"please|kindly|thank you|thanks".to_string(),
                    weight: 0.8,
                },
                IrrelevancePatternConfig {
                    pattern: r"as requested|following instruction".to_string(),
                    weight: 0.7,
                },
                IrrelevancePatternConfig {
                    pattern: r"will do|planning to|intend to".to_string(),
                    weight: 0.6,
                },
                IrrelevancePatternConfig {
                    pattern: r"background|context|history:".to_string(),
                    weight: 0.5,
                },
            ],
        }
    }
}

/// Cost-benefit analysis weights configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostBenefitWeightsConfig {
    pub tokens_weight: f64,
    pub time_weight: f64,
    pub quality_weight: f64,
    pub iteration_weight: f64,
    pub contribution_weight: f64,
}

impl Default for CostBenefitWeightsConfig {
    fn default() -> Self {
        Self {
            tokens_weight: 1.0,
            time_weight: 0.5,
            quality_weight: 2.0,
            iteration_weight: 0.3,
            contribution_weight: 1.5,
        }
    }
}

/// Role router keywords configuration for role-specific filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleRouterKeywordsConfig {
    pub extractor: Vec<String>,
    pub analyzer: Vec<String>,
    pub writer: Vec<String>,
    pub reviewer: Vec<String>,
    pub synthesizer: Vec<String>,
    pub general: Vec<String>,
    pub recency_multiplier_max: f64,
}

impl Default for RoleRouterKeywordsConfig {
    fn default() -> Self {
        Self {
            extractor: vec![
                "file_deltas".to_string(),
                "git_diff".to_string(),
                "changed_files".to_string(),
                "new_content".to_string(),
                "additions".to_string(),
                "modifications".to_string(),
            ],
            analyzer: vec![
                "metrics".to_string(),
                "patterns".to_string(),
                "analysis_results".to_string(),
                "findings".to_string(),
                "statistics".to_string(),
                "trends".to_string(),
            ],
            writer: vec![
                "draft_content".to_string(),
                "updates".to_string(),
                "modifications".to_string(),
                "revisions".to_string(),
                "text".to_string(),
                "documentation".to_string(),
            ],
            reviewer: vec![
                "code_changes".to_string(),
                "security_issues".to_string(),
                "quality_gate".to_string(),
                "bugs".to_string(),
                "errors".to_string(),
                "violations".to_string(),
            ],
            synthesizer: vec![
                "summaries".to_string(),
                "findings".to_string(),
                "consolidations".to_string(),
                "conclusions".to_string(),
                "recommendations".to_string(),
                "overview".to_string(),
            ],
            general: vec![
                "all".to_string(),
                "message".to_string(),
                "communication".to_string(),
                "update".to_string(),
            ],
            recency_multiplier_max: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_config_default() {
        let config = QualityGateConfig::default();
        assert!(config.enabled);
        assert_eq!(config.minimum_threshold, 70.0);
        assert_eq!(config.impact_weight, 0.30);
    }

    #[test]
    fn test_communication_patterns_config_default() {
        let config = CommunicationPatternsConfig::default();
        assert!(!config.redundancy_patterns.is_empty());
        assert!(!config.irrelevance_patterns.is_empty());
        assert_eq!(config.redundancy_patterns.len(), 8);
        assert_eq!(config.irrelevance_patterns.len(), 5);
    }

    #[test]
    fn test_cost_benefit_weights_config_default() {
        let config = CostBenefitWeightsConfig::default();
        assert_eq!(config.tokens_weight, 1.0);
        assert_eq!(config.quality_weight, 2.0);
    }

    #[test]
    fn test_role_router_keywords_config_default() {
        let config = RoleRouterKeywordsConfig::default();
        assert!(!config.extractor.is_empty());
        assert!(!config.analyzer.is_empty());
        assert!(!config.writer.is_empty());
        assert!(!config.reviewer.is_empty());
        assert!(!config.synthesizer.is_empty());
        assert!(!config.general.is_empty());
        assert_eq!(config.recency_multiplier_max, 2.0);
    }

    #[test]
    fn test_redundancy_pattern_config() {
        let pattern = RedundancyPatternConfig {
            pattern: r"test pattern".to_string(),
            weight: 0.5,
        };
        assert_eq!(pattern.pattern, "test pattern");
        assert_eq!(pattern.weight, 0.5);
    }

    #[test]
    fn test_irrelevance_pattern_config() {
        let pattern = IrrelevancePatternConfig {
            pattern: r"test pattern".to_string(),
            weight: 0.3,
        };
        assert_eq!(pattern.pattern, "test pattern");
        assert_eq!(pattern.weight, 0.3);
    }
}
