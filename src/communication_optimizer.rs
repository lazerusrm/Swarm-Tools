use crate::types::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct CommunicationAnalyzer {
    redundancy_patterns: Vec<(Regex, f64)>,
    irrelevance_patterns: Vec<(Regex, f64)>,
}

impl CommunicationAnalyzer {
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

pub struct CommunicationRouter {
    rules: Vec<CommunicationRule>,
}

#[derive(Debug, Clone)]
struct CommunicationRule {
    source_pattern: Regex,
    target_pattern: Regex,
    action: String,
    max_content_length: usize,
    priority_threshold: CommunicationPriority,
}

impl CommunicationRouter {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub action: String,
    pub reason: String,
    pub modified_content: String,
}

pub struct CommunicationOptimizer {
    analyzer: CommunicationAnalyzer,
    router: CommunicationRouter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMessage {
    pub source: String,
    pub target: String,
    pub content: String,
    pub original_length: usize,
    pub optimized_length: usize,
    pub token_estimate: usize,
    pub priority: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_count: usize,
    pub optimized_count: usize,
    pub reduction_pct: f64,
    pub original_tokens: usize,
    pub optimized_tokens: usize,
    pub token_reduction_pct: f64,
    pub optimized_messages: Vec<OptimizedMessage>,
    pub filtered_messages: Vec<serde_json::Value>,
}

impl CommunicationOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            analyzer: CommunicationAnalyzer::new()?,
            router: CommunicationRouter::new()?,
        })
    }

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
}

impl Default for CommunicationOptimizer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
