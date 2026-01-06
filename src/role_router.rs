use crate::config::RoleRouterKeywordsConfig;
use crate::types::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for role-based context filtering.
///
/// Contains keywords, filters, and recency settings specific to each agent role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// The agent role this config applies to.
    pub role: AgentRole,
    /// List of content filters for this role.
    pub filters: Vec<String>,
    /// Keywords that indicate relevance for this role.
    pub keywords: Vec<String>,
    /// Maximum multiplier for recency scoring (last 10% of messages get up to this multiplier).
    #[serde(default = "default_recency_multiplier")]
    pub recency_multiplier_max: f64,
}

fn default_recency_multiplier() -> f64 {
    2.0
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self {
            role: AgentRole::General,
            filters: vec!["all".to_string()],
            keywords: vec![],
            recency_multiplier_max: 2.0,
        }
    }
}

/// Result of role-based context filtering.
///
/// Contains relevance scores and filtered content for each message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleContext {
    /// The agent role this context was filtered for.
    pub role: AgentRole,
    /// Individual relevance scores for each message.
    pub relevance_scores: Vec<f64>,
    /// Filtered content items with metadata.
    pub filtered_content: Vec<FilteredContent>,
    /// Sum of all relevance scores.
    pub total_relevance: f64,
}

/// A single piece of filtered content with its relevance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredContent {
    /// Original index in the message sequence.
    pub original_index: usize,
    /// The filtered content text.
    pub content: String,
    /// Calculated relevance score (0.0 to 1.0).
    pub relevance_score: f64,
    /// Whether this content is in the last 10% (recent) messages.
    pub is_recent: bool,
    /// Impact score for this content.
    pub impact_score: f64,
}

/// Router for filtering context based on agent roles.
///
/// Uses keyword matching, position-based recency weighting, and impact scores
/// to determine which messages are most relevant for a given agent role.
/// Based on RCR-Router (Aug 2025) for 40-65% communication savings.
#[derive(Debug, Clone)]
pub struct RoleRouter {
    /// Role-specific configurations.
    role_configs: HashMap<AgentRole, RoleConfig>,
    /// Custom role configurations by name.
    custom_configs: HashMap<String, RoleConfig>,
    /// Default filters for each role.
    default_filters: HashMap<AgentRole, Vec<String>>,
}

impl RoleRouter {
    /// Creates a new RoleRouter with default configurations for all agent roles.
    ///
    /// Initializes role-specific filters based on typical responsibilities:
    /// - Extractor: file_deltas, git_diff, changed_files
    /// - Analyzer: metrics, patterns, analysis_results
    /// - Writer: draft_content, updates, modifications
    /// - Reviewer: code_changes, security_issues, quality_gate
    /// - Synthesizer: summaries, findings, consolidations
    /// - General: all, message, communication, update
    pub fn new() -> Self {
        Self::with_config(RoleRouterKeywordsConfig::default())
    }

    /// Creates a new RoleRouter with custom keywords from config.
    pub fn with_config(config: RoleRouterKeywordsConfig) -> Self {
        let mut default_filters = HashMap::new();
        default_filters.insert(AgentRole::Extractor, config.extractor);
        default_filters.insert(AgentRole::Analyzer, config.analyzer);
        default_filters.insert(AgentRole::Writer, config.writer);
        default_filters.insert(AgentRole::Reviewer, config.reviewer);
        default_filters.insert(AgentRole::Synthesizer, config.synthesizer);
        default_filters.insert(AgentRole::General, config.general);

        let mut role_configs = HashMap::new();
        for (role, filters) in &default_filters {
            role_configs.insert(
                *role,
                RoleConfig {
                    role: *role,
                    filters: filters.clone(),
                    keywords: filters.clone(),
                    recency_multiplier_max: config.recency_multiplier_max,
                },
            );
        }

        Self {
            role_configs,
            custom_configs: HashMap::new(),
            default_filters,
        }
    }

    /// Calculates a relevance score for content based on agent role.
    ///
    /// The score combines:
    /// - Keyword matching (base relevance)
    /// - Recency weighting (2.0x multiplier for last 10% of messages)
    /// - Impact boost (30% bonus based on impact_score)
    ///
    /// # Arguments
    /// * `content` - The message content to score
    /// * `role` - The target agent role
    /// * `position` - Position in the message sequence (0-indexed)
    /// * `total_messages` - Total number of messages in the sequence
    /// * `impact_score` - Importance score for this message (0.0 to 1.0)
    ///
    /// # Returns
    /// A composite relevance score. Higher scores indicate greater relevance.
    pub fn score_for_role(
        &self,
        content: &str,
        role: AgentRole,
        position: usize,
        total_messages: usize,
        impact_score: f64,
    ) -> f64 {
        let keywords = self.get_role_keywords(role);

        let keyword_score = self.keyword_matching(content, &keywords);

        let recency_threshold = (total_messages as f64 * 0.9).floor() as usize;
        let recency_multiplier_max = self
            .role_configs
            .get(&role)
            .map(|c| c.recency_multiplier_max)
            .unwrap_or(2.0);

        let position_score = if position >= recency_threshold {
            let recency_position = position - recency_threshold;
            let recency_range = total_messages.saturating_sub(recency_threshold);
            if recency_range > 0 {
                let recency_factor = recency_position as f64 / recency_range as f64;
                1.0 + (recency_multiplier_max - 1.0) * recency_factor
            } else {
                1.0
            }
        } else {
            let decay_factor = if recency_threshold > 0 {
                position as f64 / recency_threshold as f64
            } else {
                0.0
            };
            1.0 - (0.2 * decay_factor)
        };

        let impact_boost = 1.0 + (impact_score * 0.3);

        keyword_score * position_score * impact_boost
    }

    fn get_role_keywords(&self, role: AgentRole) -> Vec<String> {
        self.custom_configs
            .values()
            .find(|c| c.role == role)
            .map(|c| c.keywords.clone())
            .or(self.role_configs.get(&role).map(|c| c.keywords.clone()))
            .or(self.default_filters.get(&role).cloned())
            .unwrap_or_else(|| vec!["all".to_string()])
    }

    fn keyword_matching(&self, content: &str, keywords: &[String]) -> f64 {
        if keywords.is_empty() || keywords.iter().any(|k| k == "all") {
            return 0.5;
        }

        let content_lower = content.to_lowercase();
        let mut match_count = 0;
        let mut total_weight = 0.0;

        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            if content_lower.contains(&keyword_lower) {
                match_count += 1;
                let weight = keyword.len() as f64;
                total_weight += weight;
            }
        }

        if match_count == 0 {
            0.1
        } else {
            let base_score = match_count as f64 / keywords.len() as f64;
            let length_bonus = (total_weight / 100.0).min(0.5);
            (base_score + length_bonus).min(1.0)
        }
    }

    /// Filters and scores a sequence of messages for a specific agent role.
    ///
    /// Each message is scored based on role-specific keywords and recency.
    /// Messages in the last 10% of the sequence receive up to 2.0x recency boost.
    ///
    /// # Arguments
    /// * `messages` - Slice of tuples containing (content, position, impact_score)
    /// * `role` - The target agent role for filtering
    ///
    /// # Returns
    /// A RoleContext containing filtered content with relevance scores.
    pub fn filter_context(&self, messages: &[(&str, usize, f64)], role: AgentRole) -> RoleContext {
        let keywords = self.get_role_keywords(role);
        let recency_multiplier_max = self
            .role_configs
            .get(&role)
            .map(|c| c.recency_multiplier_max)
            .unwrap_or(2.0);

        let total_messages = messages.len();
        let recency_threshold = (total_messages as f64 * 0.9).floor() as usize;

        let mut filtered_content = Vec::new();
        let mut relevance_scores = Vec::new();

        for (idx, (content, _pos, impact)) in messages.iter().enumerate() {
            let keyword_score = self.keyword_matching(content, &keywords);

            let is_recent = idx >= recency_threshold;
            let position_score = if is_recent {
                let recency_position = idx - recency_threshold;
                let recency_range = total_messages.saturating_sub(recency_threshold);
                if recency_range > 0 {
                    let recency_factor = recency_position as f64 / recency_range as f64;
                    1.0 + (recency_multiplier_max - 1.0) * recency_factor
                } else {
                    1.0
                }
            } else {
                let decay_factor = if recency_threshold > 0 {
                    idx as f64 / recency_threshold as f64
                } else {
                    0.0
                };
                1.0 - (0.2 * decay_factor)
            };

            let impact_boost = 1.0 + (impact * 0.3);
            let relevance = keyword_score * position_score * impact_boost;

            filtered_content.push(FilteredContent {
                original_index: idx,
                content: content.to_string(),
                relevance_score: relevance,
                is_recent,
                impact_score: *impact,
            });
            relevance_scores.push(relevance);
        }

        let total_relevance: f64 = relevance_scores.iter().sum();

        RoleContext {
            role,
            relevance_scores,
            filtered_content,
            total_relevance,
        }
    }

    /// Adds a custom role configuration.
    ///
    /// Custom configs override default filters for a specific role.
    ///
    /// # Arguments
    /// * `name` - Name to identify this custom configuration
    /// * `config` - The RoleConfig to add
    pub fn add_custom_config(&mut self, name: String, config: RoleConfig) {
        self.custom_configs.insert(name, config);
    }

    /// Gets the filter keywords for a specific agent role.
    ///
    /// Returns custom config filters if available, otherwise default filters.
    ///
    /// # Arguments
    /// * `role` - The agent role to get filters for
    ///
    /// # Returns
    /// Vector of filter keywords for the specified role.
    pub fn get_role_filter(&self, role: AgentRole) -> Vec<String> {
        self.custom_configs
            .values()
            .find(|c| c.role == role)
            .map(|c| c.filters.clone())
            .or(self.default_filters.get(&role).cloned())
            .unwrap_or_else(|| vec!["all".to_string()])
    }
}

impl Default for RoleRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_router_new() {
        let router = RoleRouter::new();
        assert!(router.role_configs.contains_key(&AgentRole::Extractor));
        assert!(router.role_configs.contains_key(&AgentRole::Analyzer));
        assert!(router.role_configs.contains_key(&AgentRole::General));
    }

    #[test]
    fn test_keyword_matching() {
        let router = RoleRouter::new();
        let score = router.keyword_matching(
            "The file_deltas show git_diff changes in changed_files",
            &vec![
                "file_deltas".to_string(),
                "git_diff".to_string(),
                "changed_files".to_string(),
            ],
        );
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_for_role_with_high_impact() {
        let router = RoleRouter::new();
        let content = "The file_deltas show important changes";
        let score_high_impact = router.score_for_role(content, AgentRole::Extractor, 9, 10, 0.9);
        let score_low_impact = router.score_for_role(content, AgentRole::Extractor, 9, 10, 0.1);
        assert!(score_high_impact > score_low_impact);
    }

    #[test]
    fn test_score_for_role_recency() {
        let router = RoleRouter::new();
        let content = "file_deltas";
        let recent_score = router.score_for_role(content, AgentRole::Extractor, 9, 10, 0.5);
        let old_score = router.score_for_role(content, AgentRole::Extractor, 1, 10, 0.5);
        assert!(recent_score > old_score);
    }

    #[test]
    fn test_filter_context() {
        let router = RoleRouter::new();
        let messages = vec![
            ("old message", 1, 0.5),
            ("file_deltas and changes", 2, 0.7),
            ("very recent update", 3, 0.9),
        ];
        let context = router.filter_context(&messages, AgentRole::Extractor);

        assert_eq!(context.role, AgentRole::Extractor);
        assert_eq!(context.filtered_content.len(), 3);
        assert!(context.filtered_content[2].is_recent);
        assert!(!context.filtered_content[0].is_recent);
    }

    #[test]
    fn test_get_role_filter() {
        let router = RoleRouter::new();
        let extractor_filters = router.get_role_filter(AgentRole::Extractor);
        assert!(extractor_filters.contains(&"file_deltas".to_string()));
    }
}
