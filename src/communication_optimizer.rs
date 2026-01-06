use crate::role_router::{RoleContext, RoleRouter};
use crate::types::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Analyzes communication content for redundancy and relevance.
///
/// Uses pattern matching to identify:
/// - Redundant content (status updates, acknowledgments)
/// - Irrelevant content (pleasantries, background info)
/// - Priority levels (Critical, High, Medium, Low, Redundant)
pub struct CommunicationAnalyzer {
    /// Patterns that indicate redundant content with severity weights.
    redundancy_patterns: Vec<(Regex, f64)>,
    /// Patterns that indicate irrelevant content with severity weights.
    irrelevance_patterns: Vec<(Regex, f64)>,
}

impl CommunicationAnalyzer {
    /// Creates a new CommunicationAnalyzer with default patterns.
    ///
    /// Initializes patterns for detecting:
    /// - Redundant status updates ("working in progress", "continuing with task")
    /// - Acknowledgments ("ok", "understood", "acknowledged")
    /// - Low-value content ("as requested", "will do", "planning to")
    pub fn new() -> Result<Self> {
        Ok(Self {
            redundancy_patterns: vec![
                (
                    Regex::new(r"status:\s*working|in progress|proceeding")?,
                    0.9,
                ),
                (
                    Regex::new(r"i am|i'm (working|proceeding|continuing)")?,
                    0.8,
                ),
                (Regex::new(r"continuing|proceeding with (task|work)")?, 0.7),
                (Regex::new(r"same (as|above|previous)")?, 0.8),
                (Regex::new(r"duplicate|duplicate copy|copy of")?, 0.9),
                (Regex::new(r"already (done|completed|finished)")?, 0.85),
                (Regex::new(r"no (change|updates|new information)")?, 0.9),
                (Regex::new(r"nothing (new|to report|additional)")?, 0.9),
            ],
            irrelevance_patterns: vec![
                (Regex::new(r"acknowledged|ack|ok|understood|got it")?, 0.95),
                (Regex::new(r"please|kindly|thank you|thanks")?, 0.8),
                (Regex::new(r"as requested|following instruction")?, 0.7),
                (Regex::new(r"will do|planning to|intend to")?, 0.6),
                (Regex::new(r"background|context|history:")?, 0.5),
            ],
        })
    }

    /// Analyzes a single communication for inclusion decisions.
    ///
    /// Evaluates content for:
    /// - Priority level based on critical indicators
    /// - Redundancy score (0.0 to 1.0)
    /// - Relevance score (0.0 to 1.0)
    ///
    /// # Arguments
    /// * `_source_agent` - Agent ID sending the message
    /// * `_target_agent` - Agent ID receiving the message
    /// * `content` - The message content to analyze
    ///
    /// # Returns
    /// CommunicationAnalysis with priority, scores, and inclusion decision.
    pub fn analyze_communication(
        &self,
        _source_agent: &str,
        _target_agent: &str,
        content: &str,
    ) -> Result<CommunicationAnalysis> {
        let _content_len = content.len();

        let priority = self.determine_priority(content);
        let redundancy_score = self.calculate_redundancy(content);
        let relevance_score = self.calculate_relevance(content);

        let should_include = match priority {
            CommunicationPriority::Critical | CommunicationPriority::High => true,
            CommunicationPriority::Medium | CommunicationPriority::Low => redundancy_score < 0.7,
            CommunicationPriority::Redundant => false,
        };

        Ok(CommunicationAnalysis {
            priority,
            redundancy_score,
            relevance_score,
            should_include,
        })
    }

    fn determine_priority(&self, content: &str) -> CommunicationPriority {
        let content_lower = content.to_lowercase();

        let critical_indicators = [
            "error",
            "failed",
            "critical",
            "urgent",
            "immediately",
            "blocker",
        ];
        let high_indicators = [
            "result",
            "output",
            "findings",
            "completed",
            "finished",
            "decision",
        ];
        let low_indicators = [
            "status",
            "working",
            "proceeding",
            "acknowledged",
            "ok",
            "understood",
        ];
        let redundant_indicators = [
            "same as",
            "duplicate",
            "already done",
            "no change",
            "no updates",
            "nothing new",
        ];

        for indicator in critical_indicators.iter() {
            if content_lower.contains(indicator) {
                return CommunicationPriority::Critical;
            }
        }

        for indicator in high_indicators.iter() {
            if content_lower.contains(indicator) {
                return CommunicationPriority::High;
            }
        }

        for indicator in low_indicators.iter() {
            if content_lower.contains(indicator) {
                return CommunicationPriority::Low;
            }
        }

        for indicator in redundant_indicators.iter() {
            if content_lower.contains(indicator) {
                return CommunicationPriority::Redundant;
            }
        }

        CommunicationPriority::Medium
    }

    fn calculate_redundancy(&self, content: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let mut redundancy = 0.0;

        for (pattern, weight) in &self.redundancy_patterns {
            if pattern.is_match(&content_lower) {
                redundancy += weight;
            }
        }

        for (pattern, weight) in &self.irrelevance_patterns {
            if pattern.is_match(&content_lower) {
                redundancy += weight;
            }
        }

        (redundancy / 3.0).min(1.0)
    }

    fn calculate_relevance(&self, content: &str) -> f64 {
        let content_lower = content.to_lowercase();

        let relevant_indicators = [
            "result",
            "finding",
            "conclusion",
            "decision",
            "recommendation",
            "error",
            "issue",
            "solution",
            "fix",
        ];

        let less_relevant_indicators = [
            "status",
            "working",
            "proceeding",
            "acknowledged",
            "background",
            "history",
        ];

        let relevant_count = relevant_indicators
            .iter()
            .filter(|ind| content_lower.contains(*ind))
            .count();
        let less_relevant_count = less_relevant_indicators
            .iter()
            .filter(|ind| content_lower.contains(*ind))
            .count();

        if relevant_count > 0 {
            (0.6 + (relevant_count as f64 * 0.1)).min(1.0)
        } else if less_relevant_count > 0 {
            (0.5 - (less_relevant_count as f64 * 0.05)).max(0.2)
        } else {
            0.5
        }
    }
}

/// Routes communications based on priority and content rules.
///
/// Applies rules to decide whether to:
/// - Include: Pass through unchanged
/// - Exclude: Filter out completely
/// - Summarize: Truncate to max length
///
/// Rules are applied in priority order (Critical > High > Medium > Low > Redundant).
pub struct CommunicationRouter {
    /// Ordered list of routing rules.
    rules: Vec<CommunicationRule>,
}

/// A single routing rule for communication filtering.
#[derive(Debug, Clone)]
struct CommunicationRule {
    /// Regex pattern for matching source agent IDs.
    source_pattern: Regex,
    /// Regex pattern for matching target agent IDs.
    target_pattern: Regex,
    /// Action to take: "include", "exclude", or "summarize".
    action: String,
    /// Maximum content length for summarize action (0 = no limit).
    max_content_length: usize,
    /// Minimum priority threshold for this rule.
    priority_threshold: CommunicationPriority,
}

impl CommunicationRouter {
    /// Creates a new CommunicationRouter with default routing rules.
    ///
    /// Default rules:
    /// - Critical/High priority: include unchanged
    /// - Redundant priority: exclude
    /// - Medium priority: summarize to 1000 chars
    /// - Low priority: summarize to 500 chars
    pub fn new() -> Result<Self> {
        let rules = vec![
            CommunicationRule {
                source_pattern: Regex::new(r".*")?,
                target_pattern: Regex::new(r".*")?,
                action: "include".to_string(),
                max_content_length: 0,
                priority_threshold: CommunicationPriority::Critical,
            },
            CommunicationRule {
                source_pattern: Regex::new(r".*")?,
                target_pattern: Regex::new(r".*")?,
                action: "include".to_string(),
                max_content_length: 0,
                priority_threshold: CommunicationPriority::High,
            },
            CommunicationRule {
                source_pattern: Regex::new(r".*")?,
                target_pattern: Regex::new(r".*")?,
                action: "exclude".to_string(),
                max_content_length: 0,
                priority_threshold: CommunicationPriority::Redundant,
            },
            CommunicationRule {
                source_pattern: Regex::new(r".*")?,
                target_pattern: Regex::new(r".*")?,
                action: "summarize".to_string(),
                max_content_length: 1000,
                priority_threshold: CommunicationPriority::Medium,
            },
            CommunicationRule {
                source_pattern: Regex::new(r".*")?,
                target_pattern: Regex::new(r".*")?,
                action: "summarize".to_string(),
                max_content_length: 500,
                priority_threshold: CommunicationPriority::Low,
            },
        ];

        Ok(Self { rules })
    }

    /// Routes a communication based on priority and rules.
    ///
    /// # Arguments
    /// * `source` - Source agent ID
    /// * `target` - Target agent ID
    /// * `content` - Message content
    /// * `priority` - Communication priority level
    ///
    /// # Returns
    /// RoutingDecision with action, reason, and potentially modified content.
    pub fn route_communication(
        &self,
        source: &str,
        target: &str,
        content: &str,
        priority: CommunicationPriority,
    ) -> RoutingDecision {
        for rule in &self.rules {
            if self.matches_rule(source, target, priority, rule) {
                return self.apply_rule(content, rule);
            }
        }

        RoutingDecision {
            action: "include".to_string(),
            reason: "no matching rules".to_string(),
            modified_content: content.to_string(),
        }
    }

    fn matches_rule(
        &self,
        source: &str,
        target: &str,
        priority: CommunicationPriority,
        rule: &CommunicationRule,
    ) -> bool {
        if !rule.source_pattern.is_match(source) {
            return false;
        }

        if !rule.target_pattern.is_match(target) {
            return false;
        }

        priority as i32 <= rule.priority_threshold as i32
    }

    fn apply_rule(&self, content: &str, rule: &CommunicationRule) -> RoutingDecision {
        match rule.action.as_str() {
            "include" => RoutingDecision {
                action: "include".to_string(),
                reason: "matched include rule".to_string(),
                modified_content: content.to_string(),
            },
            "exclude" => RoutingDecision {
                action: "exclude".to_string(),
                reason: "matched exclude rule".to_string(),
                modified_content: String::new(),
            },
            "summarize" => {
                let summary = self.generate_summary(content, rule.max_content_length);
                RoutingDecision {
                    action: "summarize".to_string(),
                    reason: "matched summarize rule".to_string(),
                    modified_content: summary,
                }
            }
            _ => RoutingDecision {
                action: "include".to_string(),
                reason: "unknown rule action".to_string(),
                modified_content: content.to_string(),
            },
        }
    }

    fn generate_summary(&self, content: &str, max_length: usize) -> String {
        let sentences: Vec<&str> = content.split('.').collect();

        if let Some(first) = sentences.first() {
            if first.len() > max_length {
                format!("{}...", &first[..max_length.saturating_sub(3)])
            } else {
                first.to_string()
            }
        } else {
            "[Summary unavailable]".to_string()
        }
    }
}

/// Result of a routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Action taken: "include", "exclude", or "summarize".
    pub action: String,
    /// Explanation for why this action was taken.
    pub reason: String,
    /// Modified content (empty if excluded, truncated if summarized).
    pub modified_content: String,
}

/// Optimizes agent communications by removing redundancy and routing based on priority.
///
/// Combines:
/// - CommunicationAnalyzer for content quality assessment
/// - CommunicationRouter for priority-based routing
/// - RoleRouter for role-aware context filtering (60-80% token reduction)
///
/// Based on research from RCR-Router (Aug 2025) and Trajectory Reduction (Sep 2025).
pub struct CommunicationOptimizer {
    /// Analyzes content for redundancy and relevance.
    analyzer: CommunicationAnalyzer,
    /// Routes communications based on priority.
    router: CommunicationRouter,
    /// Filters context based on agent roles.
    role_router: RoleRouter,
}

/// A single optimized message ready for transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMessage {
    /// Source agent ID.
    pub source: String,
    /// Target agent ID.
    pub target: String,
    /// Optimized message content.
    pub content: String,
    /// Original content length in characters.
    pub original_length: usize,
    /// Optimized content length in characters.
    pub optimized_length: usize,
    /// Estimated token count (chars / 4).
    pub token_estimate: usize,
    /// Priority level as string.
    pub priority: String,
    /// Reason for optimization decisions.
    pub reason: String,
}

/// Summary of communication optimization results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Number of messages before optimization.
    pub original_count: usize,
    /// Number of messages after optimization.
    pub optimized_count: usize,
    /// Percentage of messages filtered out.
    pub reduction_pct: f64,
    /// Estimated tokens before optimization.
    pub original_tokens: usize,
    /// Estimated tokens after optimization.
    pub optimized_tokens: usize,
    /// Percentage of tokens saved.
    pub token_reduction_pct: f64,
    /// Messages that passed through.
    pub optimized_messages: Vec<OptimizedMessage>,
    /// Messages that were filtered out.
    pub filtered_messages: Vec<serde_json::Value>,
}

/// Result of role-based routing with full context analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBasedRoutingResult {
    /// The agent role this routing was performed for.
    pub target_role: AgentRole,
    /// Full role context with relevance scores.
    pub context: RoleContext,
    /// Messages that passed the relevance threshold.
    pub messages_to_include: Vec<OptimizedMessage>,
    /// Messages that were filtered out.
    pub messages_to_exclude: Vec<serde_json::Value>,
    /// Relevance threshold used for filtering.
    pub relevance_threshold: f64,
    /// Sum of all relevance scores.
    pub total_relevance_score: f64,
}

impl CommunicationOptimizer {
    /// Creates a new CommunicationOptimizer with all components.
    pub fn new() -> Result<Self> {
        Ok(Self {
            analyzer: CommunicationAnalyzer::new()?,
            router: CommunicationRouter::new()?,
            role_router: RoleRouter::new(),
        })
    }

    /// Optimizes communications filtered for a specific agent role.
    ///
    /// Combines role-based context filtering with priority routing to produce
    /// optimized communications relevant to the target agent role.
    ///
    /// # Arguments
    /// * `communications` - Vector of communication JSON objects with "source", "target", "content"
    /// * `target_role` - The agent role to optimize for
    ///
    /// # Returns
    /// OptimizationResult with filtered and optimized messages.
    pub fn optimize_for_role(
        &self,
        communications: &[serde_json::Value],
        target_role: AgentRole,
    ) -> Result<OptimizationResult> {
        let mut optimized_messages = Vec::new();
        let mut filtered_messages = Vec::new();

        let original_count = communications.len();
        let original_tokens: usize = communications
            .iter()
            .map(|c| {
                let content = c.get("content").and_then(|v| v.as_str()).unwrap_or("");
                content.len() / 4
            })
            .sum();

        let messages_with_impact: Vec<(&str, usize, f64)> = communications
            .iter()
            .enumerate()
            .map(|(idx, comm)| {
                let content = comm.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let impact = self.extract_impact_score(comm);
                (content, idx, impact)
            })
            .collect();

        let role_context = self
            .role_router
            .filter_context(&messages_with_impact, target_role);

        for (idx, comm) in communications.iter().enumerate() {
            let source = comm
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let target = comm
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let content = comm.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let relevance = role_context
                .relevance_scores
                .get(idx)
                .copied()
                .unwrap_or(0.0);

            let relevance_threshold = 0.3;
            let analysis = self
                .analyzer
                .analyze_communication(source, target, content)?;
            let priority = analysis.priority;

            let routing = self
                .router
                .route_communication(source, target, content, priority);

            if routing.action == "exclude" || relevance < relevance_threshold {
                filtered_messages.push(comm.clone());
            } else {
                let token_estimate = routing.modified_content.len() / 4;

                optimized_messages.push(OptimizedMessage {
                    source: source.to_string(),
                    target: target.to_string(),
                    content: routing.modified_content.clone(),
                    original_length: content.len(),
                    optimized_length: routing.modified_content.len(),
                    token_estimate,
                    priority: format!("{:?}", priority),
                    reason: format!("{} (role relevance: {:.2})", routing.reason, relevance),
                });
            }
        }

        let optimized_count = optimized_messages.len();
        let optimized_tokens: usize = optimized_messages.iter().map(|m| m.token_estimate).sum();

        let reduction_pct = if original_count > 0 {
            ((original_count - optimized_count) as f64 / original_count as f64) * 100.0
        } else {
            0.0
        };

        let token_reduction_pct = if original_tokens > 0 {
            ((original_tokens - optimized_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        Ok(OptimizationResult {
            original_count,
            optimized_count,
            reduction_pct,
            original_tokens,
            optimized_tokens,
            token_reduction_pct,
            optimized_messages,
            filtered_messages,
        })
    }

    fn extract_impact_score(&self, comm: &serde_json::Value) -> f64 {
        comm.get("impact_score")
            .and_then(|v| v.as_f64())
            .or(comm
                .get("priority")
                .and_then(|v| v.as_str())
                .map(|p| match p {
                    "Critical" => 1.0,
                    "High" => 0.8,
                    "Medium" => 0.5,
                    "Low" => 0.3,
                    "Redundant" => 0.1,
                    _ => 0.5,
                }))
            .unwrap_or(0.5)
    }

    /// Gets role context for a sequence of messages.
    ///
    /// # Arguments
    /// * `communications` - Vector of message content strings
    /// * `role` - The agent role to analyze for
    ///
    /// # Returns
    /// RoleContext with relevance scores for each message.
    pub fn get_role_context(&self, communications: &[String], role: AgentRole) -> RoleContext {
        let messages_with_impact: Vec<(&str, usize, f64)> = communications
            .iter()
            .enumerate()
            .map(|(idx, content)| (content.as_str(), idx, 0.5))
            .collect();
        self.role_router.filter_context(&messages_with_impact, role)
    }

    /// Optimizes all communications without role-based filtering.
    ///
    /// Applies redundancy detection and priority routing to all messages.
    /// Use optimize_for_role() for role-specific filtering.
    ///
    /// # Arguments
    /// * `communications` - Vector of communication JSON objects
    ///
    /// # Returns
    /// OptimizationResult with optimized and filtered messages.
    pub fn optimize_communications(
        &self,
        communications: &[serde_json::Value],
    ) -> Result<OptimizationResult> {
        let mut optimized_messages = Vec::new();
        let mut filtered_messages = Vec::new();

        let original_count = communications.len();
        let original_tokens: usize = communications
            .iter()
            .map(|c| {
                let content = c.get("content").and_then(|v| v.as_str()).unwrap_or("");
                content.len() / 4
            })
            .sum();

        for comm in communications {
            let source = comm
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let target = comm
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let content = comm.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let analysis = self
                .analyzer
                .analyze_communication(source, target, content)?;
            let priority = analysis.priority;

            let routing = self
                .router
                .route_communication(source, target, content, priority);

            if routing.action == "exclude" {
                filtered_messages.push(comm.clone());
            } else {
                let token_estimate = routing.modified_content.len() / 4;

                optimized_messages.push(OptimizedMessage {
                    source: source.to_string(),
                    target: target.to_string(),
                    content: routing.modified_content.clone(),
                    original_length: content.len(),
                    optimized_length: routing.modified_content.len(),
                    token_estimate,
                    priority: format!("{:?}", priority),
                    reason: routing.reason,
                });
            }
        }

        let optimized_count = optimized_messages.len();
        let optimized_tokens: usize = optimized_messages.iter().map(|m| m.token_estimate).sum();

        let reduction_pct = if original_count > 0 {
            ((original_count - optimized_count) as f64 / original_count as f64) * 100.0
        } else {
            0.0
        };

        let token_reduction_pct = if original_tokens > 0 {
            ((original_tokens - optimized_tokens) as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        Ok(OptimizationResult {
            original_count,
            optimized_count,
            reduction_pct,
            original_tokens,
            optimized_tokens,
            token_reduction_pct,
            optimized_messages,
            filtered_messages,
        })
    }

    /// Routes communications for a specific role with relevance threshold.
    ///
    /// Filters out messages that don't meet the relevance threshold for the target role.
    /// Uses role-specific keyword matching and recency weighting.
    ///
    /// # Arguments
    /// * `communications` - Vector of communication JSON objects
    /// * `target_role` - The agent role to route for
    /// * `relevance_threshold` - Minimum relevance score to include (0.0 to 1.0)
    ///
    /// # Returns
    /// RoleBasedRoutingResult with filtered messages and full context analysis.
    pub fn route_for_role(
        &self,
        communications: &[serde_json::Value],
        target_role: AgentRole,
        relevance_threshold: f64,
    ) -> Result<RoleBasedRoutingResult> {
        let messages_with_impact: Vec<(&str, usize, f64)> = communications
            .iter()
            .enumerate()
            .map(|(idx, comm)| {
                let content = comm.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let impact = self.extract_impact_score(comm);
                (content, idx, impact)
            })
            .collect();

        let role_context = self
            .role_router
            .filter_context(&messages_with_impact, target_role);

        let mut messages_to_include = Vec::new();
        let mut messages_to_exclude = Vec::new();

        for (idx, comm) in communications.iter().enumerate() {
            let source = comm
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let target = comm
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let content = comm.get("content").and_then(|v| v.as_str()).unwrap_or("");

            let relevance = role_context
                .relevance_scores
                .get(idx)
                .copied()
                .unwrap_or(0.0);

            let analysis = self
                .analyzer
                .analyze_communication(source, target, content)?;
            let priority = analysis.priority;

            let routing = self
                .router
                .route_communication(source, target, content, priority);

            if relevance >= relevance_threshold && routing.action != "exclude" {
                let token_estimate = routing.modified_content.len() / 4;

                messages_to_include.push(OptimizedMessage {
                    source: source.to_string(),
                    target: target.to_string(),
                    content: routing.modified_content.clone(),
                    original_length: content.len(),
                    optimized_length: routing.modified_content.len(),
                    token_estimate,
                    priority: format!("{:?}", priority),
                    reason: format!("Role relevance: {:.2}", relevance),
                });
            } else {
                messages_to_exclude.push(comm.clone());
            }
        }

        Ok(RoleBasedRoutingResult {
            target_role,
            context: role_context.clone(),
            messages_to_include,
            messages_to_exclude,
            relevance_threshold,
            total_relevance_score: role_context.total_relevance,
        })
    }
}

impl Default for CommunicationOptimizer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
