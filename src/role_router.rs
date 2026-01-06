use crate::types::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    pub role: AgentRole,
    pub filters: Vec<String>,
    pub keywords: Vec<String>,
    pub recency_multiplier_max: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleContext {
    pub role: AgentRole,
    pub relevance_scores: Vec<f64>,
    pub filtered_content: Vec<FilteredContent>,
    pub total_relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredContent {
    pub original_index: usize,
    pub content: String,
    pub relevance_score: f64,
    pub is_recent: bool,
    pub impact_score: f64,
}

#[derive(Debug, Clone)]
pub struct RoleRouter {
    role_configs: HashMap<AgentRole, RoleConfig>,
    custom_configs: HashMap<String, RoleConfig>,
    default_filters: HashMap<AgentRole, Vec<String>>,
}

impl RoleRouter {
    pub fn new() -> Self {
        let mut default_filters = HashMap::new();
        default_filters.insert(
            AgentRole::Extractor,
            vec![
                "file_deltas".to_string(),
                "git_diff".to_string(),
                "changed_files".to_string(),
                "new_content".to_string(),
                "additions".to_string(),
                "modifications".to_string(),
            ],
        );
        default_filters.insert(
            AgentRole::Analyzer,
            vec![
                "metrics".to_string(),
                "patterns".to_string(),
                "analysis_results".to_string(),
                "findings".to_string(),
                "statistics".to_string(),
                "trends".to_string(),
            ],
        );
        default_filters.insert(
            AgentRole::Writer,
            vec![
                "draft_content".to_string(),
                "updates".to_string(),
                "modifications".to_string(),
                "revisions".to_string(),
                "text".to_string(),
                "documentation".to_string(),
            ],
        );
        default_filters.insert(
            AgentRole::Reviewer,
            vec![
                "code_changes".to_string(),
                "security_issues".to_string(),
                "quality_gate".to_string(),
                "bugs".to_string(),
                "errors".to_string(),
                "violations".to_string(),
            ],
        );
        default_filters.insert(
            AgentRole::Synthesizer,
            vec![
                "summaries".to_string(),
                "findings".to_string(),
                "consolidations".to_string(),
                "conclusions".to_string(),
                "recommendations".to_string(),
                "overview".to_string(),
            ],
        );
        default_filters.insert(
            AgentRole::General,
            vec![
                "all".to_string(),
                "message".to_string(),
                "communication".to_string(),
                "update".to_string(),
            ],
        );

        let mut role_configs = HashMap::new();
        for (role, filters) in &default_filters {
            role_configs.insert(
                *role,
                RoleConfig {
                    role: *role,
                    filters: filters.clone(),
                    keywords: filters.clone(),
                    recency_multiplier_max: 2.0,
                },
            );
        }

        Self {
            role_configs,
            custom_configs: HashMap::new(),
            default_filters,
        }
    }

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

    pub fn add_custom_config(&mut self, name: String, config: RoleConfig) {
        self.custom_configs.insert(name, config);
    }

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
