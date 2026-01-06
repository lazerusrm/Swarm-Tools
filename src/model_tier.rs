use crate::feature_config::ModelTieringConfig;
use crate::types::TaskComplexity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelTier {
    Haiku,
    Sonnet,
    Opus,
    Custom(String),
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Haiku => write!(f, "claude-haiku-4-5-2025"),
            ModelTier::Sonnet => write!(f, "claude-sonnet-4-5-2025"),
            ModelTier::Opus => write!(f, "claude-opus-4-5-2025"),
            ModelTier::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    pub tier: ModelTier,
    pub model_name: String,
    pub reasoning: String,
    pub token_limit: u32,
}

pub struct ModelTierer {
    config: ModelTieringConfig,
}

impl ModelTierer {
    pub fn new() -> Self {
        Self::with_config(ModelTieringConfig::default())
    }

    pub fn with_config(config: ModelTieringConfig) -> Self {
        Self { config }
    }

    pub fn select_model(
        &self,
        estimated_tokens: u32,
        complexity: TaskComplexity,
        impact_score: f64,
    ) -> ModelSelection {
        if !self.config.enabled {
            return self.fallback_selection(impact_score);
        }

        let mut tier = self.determine_base_tier(estimated_tokens);

        if self.config.high_impact_boost_enabled && impact_score > 0.8 {
            tier = self.boost_for_high_impact(tier);
        }

        tier = self.adjust_for_complexity(tier, complexity);

        self.create_selection(tier, impact_score)
    }

    fn determine_base_tier(&self, estimated_tokens: u32) -> ModelTier {
        if estimated_tokens < self.config.simple_haiku_threshold {
            ModelTier::Haiku
        } else if estimated_tokens < self.config.moderate_sonnet_threshold {
            ModelTier::Sonnet
        } else {
            ModelTier::Opus
        }
    }

    fn boost_for_high_impact(&self, current_tier: ModelTier) -> ModelTier {
        match current_tier {
            ModelTier::Haiku => ModelTier::Sonnet,
            ModelTier::Sonnet => ModelTier::Opus,
            ModelTier::Opus => ModelTier::Opus,
            ModelTier::Custom(_) => current_tier,
        }
    }

    fn adjust_for_complexity(&self, tier: ModelTier, complexity: TaskComplexity) -> ModelTier {
        match complexity {
            TaskComplexity::VeryComplex => ModelTier::Opus,
            TaskComplexity::Complex => {
                if tier == ModelTier::Haiku {
                    ModelTier::Sonnet
                } else {
                    tier
                }
            }
            TaskComplexity::Moderate => tier,
            TaskComplexity::Simple => tier,
        }
    }

    fn create_selection(&self, tier: ModelTier, impact_score: f64) -> ModelSelection {
        let model_name = match &tier {
            ModelTier::Haiku => "claude-haiku-4-5-2025".to_string(),
            ModelTier::Sonnet => "claude-sonnet-4-5-2025".to_string(),
            ModelTier::Opus => "claude-opus-4-5-2025".to_string(),
            ModelTier::Custom(name) => name.clone(),
        };

        let token_limit = match &tier {
            ModelTier::Haiku => 200_000,
            ModelTier::Sonnet => 200_000,
            ModelTier::Opus => 200_000,
            ModelTier::Custom(_) => 200_000,
        };

        let reasoning = format!(
            "Selected {} for {} tokens, impact {:.2}",
            model_name,
            match &tier {
                ModelTier::Haiku => "low",
                ModelTier::Sonnet => "moderate",
                ModelTier::Opus => "high",
                ModelTier::Custom(_) => "custom",
            },
            impact_score
        );

        ModelSelection {
            tier,
            model_name,
            reasoning,
            token_limit,
        }
    }

    fn fallback_selection(&self, impact_score: f64) -> ModelSelection {
        let model_name = "claude-opus-4-5-2025".to_string();
        ModelSelection {
            tier: ModelTier::Custom(model_name.clone()),
            model_name,
            reasoning: "Tiering disabled, using fallback".to_string(),
            token_limit: 200_000,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn get_thresholds(&self) -> (u32, u32) {
        (
            self.config.simple_haiku_threshold,
            self.config.moderate_sonnet_threshold,
        )
    }
}

impl Default for ModelTierer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haiku_for_low_tokens() {
        let tierer = ModelTierer::new();
        let result = tierer.select_model(500, TaskComplexity::Simple, 0.3);
        assert_eq!(result.tier, ModelTier::Haiku);
    }

    #[test]
    fn test_sonnet_for_moderate_tokens() {
        let tierer = ModelTierer::new();
        let result = tierer.select_model(3000, TaskComplexity::Moderate, 0.5);
        assert_eq!(result.tier, ModelTier::Sonnet);
    }

    #[test]
    fn test_opus_for_high_tokens() {
        let tierer = ModelTierer::new();
        let result = tierer.select_model(10000, TaskComplexity::Complex, 0.7);
        assert_eq!(result.tier, ModelTier::Opus);
    }

    #[test]
    fn test_high_impact_boost() {
        let tierer = ModelTierer::new();
        let result = tierer.select_model(500, TaskComplexity::Simple, 0.85);
        assert_eq!(result.tier, ModelTier::Sonnet);
    }

    #[test]
    fn test_very_complex_always_opus() {
        let tierer = ModelTierer::new();
        let result = tierer.select_model(500, TaskComplexity::VeryComplex, 0.3);
        assert_eq!(result.tier, ModelTier::Opus);
    }

    #[test]
    fn test_disabled_uses_fallback() {
        let mut config = ModelTieringConfig::default();
        config.enabled = false;
        let tierer = ModelTierer::with_config(config);
        let result = tierer.select_model(500, TaskComplexity::Simple, 0.5);
        assert_eq!(
            result.tier,
            ModelTier::Custom("claude-opus-4-5-2025".to_string())
        );
    }

    #[test]
    fn test_model_name_display() {
        assert_eq!(ModelTier::Haiku.to_string(), "claude-haiku-4-5-2025");
        assert_eq!(ModelTier::Sonnet.to_string(), "claude-sonnet-4-5-2025");
        assert_eq!(ModelTier::Opus.to_string(), "claude-opus-4-5-2025");
    }
}
