use crate::config::RoleRouterKeywordsConfig;
use crate::semantic_engine::{RoleEmbeddingStore, SemanticEngine};
use crate::types::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
/// Uses embedding-based semantic matching when available, with keyword matching as fallback.
/// Position-based recency weighting and impact scores are combined for final relevance.
#[derive(Debug, Clone)]
pub struct RoleRouter {
    /// Role-specific configurations.
    role_configs: HashMap<AgentRole, RoleConfig>,
    /// Custom role configurations by name.
    custom_configs: HashMap<String, RoleConfig>,
    /// Default filters for each role.
    default_filters: HashMap<AgentRole, Vec<String>>,
    /// Semantic engine for embedding-based routing.
    semantic_engine: Option<Arc<SemanticEngine>>,
    /// Pre-computed role embeddings.
    role_embeddings: Option<Arc<RoleEmbeddingStore>>,
    /// Whether to use semantic routing.
    use_semantic: bool,
}

impl RoleRouter {
    /// Creates a new RoleRouter with default configurations for all agent roles.
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
            semantic_engine: None,
            role_embeddings: None,
            use_semantic: false,
        }
    }

    /// Creates a RoleRouter with semantic embedding support.
    pub fn with_semantic_engine(semantic_engine: Arc<SemanticEngine>) -> Self {
        let mut router = Self::with_config(RoleRouterKeywordsConfig::default());
        router.semantic_engine = Some(semantic_engine.clone());
        router.role_embeddings = Some(Arc::new(RoleEmbeddingStore::new(semantic_engine)));
        router.use_semantic = true;
        router
    }

    /// Routes a task to the most appropriate agent role based on semantic similarity.
    ///
    /// Uses embedding cosine similarity between the task description and role definitions.
    /// Falls back to keyword matching if embeddings are not available.
    ///
    /// # Arguments
    /// * `task_description` - The task/user prompt to route
    ///
    /// # Returns
    /// The most appropriate AgentRole for this task.
    pub fn route_task(&self, task_description: &str) -> AgentRole {
        if self.use_semantic {
            if let Some(store) = &self.role_embeddings {
                return store.route_task(task_description);
            }
        }

        // Fallback to keyword-based routing
        self.route_task_keyword(task_description)
    }

    /// Fallback keyword-based task routing.
    fn route_task_keyword(&self, task: &str) -> AgentRole {
        let task_lower = task.to_lowercase();

        // Score each role based on keyword matching
        let mut scores: Vec<(AgentRole, f64)> = self
            .default_filters
            .iter()
            .map(|(role, keywords)| {
                let score = self.task_keyword_score(&task_lower, keywords);
                (*role, score)
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        eprintln!(
            "DEBUG get_all_routing_scores: task='{}', scores={:?}",
            task, scores
        );

        // Return the highest scoring role
        scores
            .first()
            .map(|(r, _)| *r)
            .unwrap_or(AgentRole::General)
    }

    fn task_keyword_score(&self, task: &str, keywords: &[String]) -> f64 {
        if keywords.is_empty() {
            return 0.0;
        }

        // "all" is a special catch-all keyword with low priority
        let has_all_only = keywords.len() == 1 && keywords.iter().any(|k| k == "all");
        if has_all_only {
            return 0.1;
        }

        let task_normalized = task.replace(' ', "_");
        let task_words: Vec<&str> = task_normalized
            .split('_')
            .filter(|w| !w.is_empty())
            .collect();
        let mut exact_match_count = 0;
        let mut partial_match_count = 0;

        for keyword in keywords {
            if keyword == "all" {
                continue;
            }
            let keyword_lower = keyword.to_lowercase();
            let keyword_words: Vec<&str> =
                keyword_lower.split('_').filter(|w| !w.is_empty()).collect();

            // Check if keyword is a substring of task (handles underscores vs spaces)
            if task_normalized.contains(&keyword_lower) {
                exact_match_count += 1;
            } else {
                // Check bidirectional partial matching (minimum 3 chars to avoid spurious matches):
                // 1. Any task word starts with keyword word (e.g., "security" matches "security_issues")
                // 2. Any keyword word starts with task word (e.g., "security_issues" matches "security")
                let mut matched = false;
                for task_word in &task_words {
                    if task_word.len() < 3 {
                        continue; // Skip very short task words
                    }
                    for kw_word in &keyword_words {
                        if kw_word.len() < 3 {
                            continue; // Skip very short keyword words
                        }
                        if task_word.starts_with(kw_word) || kw_word.starts_with(task_word) {
                            matched = true;
                            break;
                        }
                    }
                    if matched {
                        break;
                    }
                }
                if matched {
                    partial_match_count += 1;
                }
            }
        }

        // Weight exact matches more heavily than partial matches
        let total_score = exact_match_count as f64 + partial_match_count as f64 * 0.5;
        total_score / keywords.len() as f64
    }

    /// Calculates a relevance score for content based on agent role.
    ///
    /// The score combines:
    /// - Keyword matching or semantic similarity (base relevance)
    /// - Recency weighting (2.0x multiplier for last 10% of messages)
    /// - Impact boost (30% bonus based on impact_score)
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
    pub fn add_custom_config(&mut self, name: String, config: RoleConfig) {
        self.custom_configs.insert(name, config);
    }

    /// Gets the filter keywords for a specific agent role.
    pub fn get_role_filter(&self, role: AgentRole) -> Vec<String> {
        self.custom_configs
            .values()
            .find(|c| c.role == role)
            .map(|c| c.filters.clone())
            .or(self.default_filters.get(&role).cloned())
            .unwrap_or_else(|| vec!["all".to_string()])
    }

    /// Returns whether semantic routing is enabled.
    pub fn is_using_semantic(&self) -> bool {
        self.use_semantic
    }

    /// Gets routing scores for all roles (useful for debugging/transparency).
    pub fn get_all_routing_scores(&self, task: &str) -> Vec<(AgentRole, f64)> {
        if self.use_semantic {
            if let Some(store) = &self.role_embeddings {
                return store
                    .get_all_scores(task)
                    .into_iter()
                    .map(|(r, s)| (r, s as f64))
                    .collect();
            }
        }

        // Fallback to keyword scores
        let task_lower = task.to_lowercase();
        let mut scores: Vec<(AgentRole, f64)> = self
            .default_filters
            .iter()
            .map(|(role, keywords)| {
                let score = self.task_keyword_score(&task_lower, keywords);
                (*role, score)
            })
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
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

    #[test]
    fn test_route_task_keyword() {
        let router = RoleRouter::new();

        // Code extraction task
        let role = router.route_task("Show me the git diff for the recent changes");
        assert_eq!(role, AgentRole::Extractor);

        // Code analysis task
        let role = router.route_task("Analyze the performance metrics and identify bottlenecks");
        assert_eq!(role, AgentRole::Analyzer);

        // Code review task
        let role = router.route_task("Review this code for security vulnerabilities");
        assert_eq!(role, AgentRole::Reviewer);
    }

    #[test]
    fn test_get_all_routing_scores() {
        let router = RoleRouter::new();
        let scores = router.get_all_routing_scores("Analyze code metrics");

        assert!(!scores.is_empty());
        // Analyzer should be highest
        assert_eq!(scores[0].0, AgentRole::Analyzer);
    }
}
