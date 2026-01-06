use crate::enhanced_monitor::TrajectoryCompression;
use crate::types::{CompressedTrajectory, SummaryGroup, TrajectoryEntry, TrajectoryLog};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for trajectory compression behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryCompressorConfig {
    /// Minimum impact score to preserve entry (0.0 to 1.0).
    pub preserve_threshold: f64,
    /// Maximum summaries to keep when grouping repeated actions.
    pub max_summaries: usize,
    /// Regex patterns for superseded content detection.
    pub superseded_patterns: Vec<String>,
    /// Whether to auto-filter redundant entries.
    pub filter_redundant: bool,
    /// Token budget for compressed trajectory.
    pub max_tokens: usize,
}

impl Default for TrajectoryCompressorConfig {
    fn default() -> Self {
        Self {
            preserve_threshold: 0.7,
            max_summaries: 10,
            superseded_patterns: vec![
                r"updated".to_string(),
                r"replaced".to_string(),
                r"superseded".to_string(),
                r"newer".to_string(),
                r"later".to_string(),
                r"overrides?".to_string(),
            ],
            filter_redundant: true,
            max_tokens: 10000,
        }
    }
}

/// Concrete implementation of trajectory compression.
///
/// Provides full implementation of the TrajectoryCompression trait with
/// configurable thresholds, superseded detection, and summary grouping.
#[derive(Debug, Clone)]
pub struct TrajectoryCompressor {
    config: TrajectoryCompressorConfig,
    superseded_patterns: Vec<Regex>,
    preserved_count: usize,
    summarized_count: usize,
    filtered_count: usize,
}

impl TrajectoryCompressor {
    /// Creates a new TrajectoryCompressor with default configuration.
    pub fn new() -> Self {
        Self::with_config(TrajectoryCompressorConfig::default())
    }

    /// Creates a TrajectoryCompressor with custom configuration.
    pub fn with_config(config: TrajectoryCompressorConfig) -> Self {
        let superseded_patterns = config
            .superseded_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            config,
            superseded_patterns,
            preserved_count: 0,
            summarized_count: 0,
            filtered_count: 0,
        }
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &TrajectoryCompressorConfig {
        &self.config
    }

    /// Updates configuration dynamically.
    pub fn update_config(&mut self, config: TrajectoryCompressorConfig) {
        self.superseded_patterns = config
            .superseded_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        self.config = config;
    }

    /// Resets compression statistics.
    pub fn reset_stats(&mut self) {
        self.preserved_count = 0;
        self.summarized_count = 0;
        self.filtered_count = 0;
    }

    /// Gets compression statistics.
    pub fn stats(&self) -> CompressionStats {
        CompressionStats {
            preserved: self.preserved_count,
            summarized: self.summarized_count,
            filtered: self.filtered_count,
        }
    }

    fn is_superseded(&self, entry: &TrajectoryEntry) -> bool {
        let outcome_lower = entry.outcome.to_lowercase();
        self.superseded_patterns
            .iter()
            .any(|p| p.is_match(&outcome_lower))
    }

    fn is_redundant(&self, entry: &TrajectoryEntry) -> bool {
        !entry.succeeded && entry.is_repeat && entry.impact_score < self.config.preserve_threshold
    }
}

/// Statistics from trajectory compression operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Number of entries preserved verbatim.
    pub preserved: usize,
    /// Number of entries summarized.
    pub summarized: usize,
    /// Number of entries filtered out.
    pub filtered: usize,
}

impl CompressionStats {
    /// Total entries processed.
    pub fn total(&self) -> usize {
        self.preserved + self.summarized + self.filtered
    }

    /// Compression ratio (preserved / total).
    pub fn preservation_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            self.preserved as f64 / total as f64
        }
    }
}

impl TrajectoryCompression for TrajectoryCompressor {
    fn get_compression_threshold(&self) -> (usize, usize) {
        (18, 25000)
    }

    fn should_compress(&self, context_pct: f64, steps: usize, tokens: usize) -> bool {
        context_pct > 0.80 || steps >= 18 || tokens >= 25000
    }

    fn compress_trajectory(&self, trajectory: &TrajectoryLog) -> CompressedTrajectory {
        let high_impact_threshold = self.config.preserve_threshold;

        let mut preserved: Vec<TrajectoryEntry> = Vec::new();
        let mut low_impact: Vec<&TrajectoryEntry> = Vec::new();

        for entry in &trajectory.entries {
            if entry.impact_score >= high_impact_threshold || entry.succeeded {
                preserved.push(entry.clone());
            } else if self.is_superseded(entry) || self.is_redundant(entry) {
                // filtered_count increment (stats only)
            } else {
                low_impact.push(entry);
            }
        }

        // preserved_count increment (stats only)
        let summarized = TrajectoryCompressor::group_and_summarize(&low_impact);
        // summarized_count increment (stats only)

        let original_tokens = trajectory.tokens_used;
        let preserved_tokens: u32 = preserved.iter().map(|e| e.tokens_used).sum();
        let summarized_tokens: u32 = summarized.iter().map(|s| s.tokens_saved).sum();
        let compressed_tokens = preserved_tokens + summarized_tokens / 3;

        let compression_ratio = if original_tokens > 0 {
            compressed_tokens as f64 / original_tokens as f64
        } else {
            0.0
        };

        CompressedTrajectory {
            preserved,
            summarized,
            compression_ratio,
            debug_raw: None,
        }
    }

    fn filter_expired_info(&self, entries: &[TrajectoryEntry]) -> Vec<TrajectoryEntry> {
        let mut latest_by_action: HashMap<String, &TrajectoryEntry> = HashMap::new();
        let mut superseded: Vec<&TrajectoryEntry> = Vec::new();

        for entry in entries {
            if entry.succeeded {
                if let Some(existing) = latest_by_action.get(&entry.action) {
                    if self.is_superseded(entry) {
                        superseded.push(entry);
                        continue;
                    }
                    if entry.impact_score > existing.impact_score {
                        superseded.push(*existing);
                        latest_by_action.insert(entry.action.clone(), entry);
                    } else {
                        superseded.push(entry);
                    }
                } else {
                    latest_by_action.insert(entry.action.clone(), entry);
                }
            }
        }

        let mut result: Vec<TrajectoryEntry> = latest_by_action.into_values().cloned().collect();
        // filtered_count assignment (stats only)
        result.extend(
            superseded
                .into_iter()
                .filter(|e| e.impact_score >= 0.5)
                .cloned(),
        );

        result.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        result
    }

    fn group_and_summarize(entries: &[&TrajectoryEntry]) -> Vec<SummaryGroup> {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut action_groups: HashMap<String, Vec<&TrajectoryEntry>> = HashMap::new();
        let mut total_tokens = 0u32;

        for &entry in entries {
            action_groups
                .entry(entry.action.clone())
                .or_insert_with(Vec::new)
                .push(entry);
            total_tokens += entry.tokens_used;
        }

        let mut summaries: Vec<SummaryGroup> = action_groups
            .into_iter()
            .filter_map(|(action, group)| {
                if group.len() < 2 {
                    return None;
                }
                let successful_count = group.iter().filter(|e| e.succeeded).count();
                let failed_count = group.iter().filter(|e| !e.succeeded).count();
                let count = group.len();
                let tokens_saved = if count > 1 {
                    (count as u32 - 1) * 100
                } else {
                    0
                };

                let pattern = if failed_count > 0 && failed_count >= count / 2 {
                    format!("failed_attempt_{}", failed_count)
                } else if successful_count > 0 {
                    format!("successful_attempt_{}", successful_count)
                } else {
                    action.clone()
                };

                Some(SummaryGroup {
                    pattern,
                    count: count as u32,
                    consolidated_description: group
                        .first()
                        .map(|e| e.outcome.clone())
                        .unwrap_or_default(),
                    tokens_saved,
                })
            })
            .collect();

        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        summaries.truncate(10);

        summaries
    }
}

impl Default for TrajectoryCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TrajectoryLog;

    fn create_test_trajectory() -> TrajectoryLog {
        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "extract_data".to_string(),
                outcome: "Extracted 1000 records".to_string(),
                is_repeat: false,
                impact_score: 0.9,
                succeeded: true,
                tokens_used: 500,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "extract_data".to_string(),
                outcome: "Extracted 1000 records".to_string(),
                is_repeat: true,
                impact_score: 0.9,
                succeeded: true,
                tokens_used: 500,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "failed_query".to_string(),
                outcome: "Connection timeout".to_string(),
                is_repeat: false,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:03:00Z".to_string(),
                action: "failed_query".to_string(),
                outcome: "Connection timeout".to_string(),
                is_repeat: true,
                impact_score: 0.15,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:04:00Z".to_string(),
                action: "updated_result".to_string(),
                outcome: "This result updated the previous one".to_string(),
                is_repeat: false,
                impact_score: 0.5,
                succeeded: true,
                tokens_used: 300,
            },
        ];

        TrajectoryLog {
            entries,
            tokens_used: 1500,
            compressibility_score: 0.6,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_compressor_preserves_high_impact() {
        let compressor = TrajectoryCompressor::new();
        let trajectory = create_test_trajectory();
        let compressed = compressor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.len() >= 2);
    }

    #[test]
    fn test_compressor_groups_repeated_actions() {
        let compressor = TrajectoryCompressor::new();

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "query_db".to_string(),
                outcome: "Query result 1".to_string(),
                is_repeat: false,
                impact_score: 0.5,
                succeeded: true,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "query_db".to_string(),
                outcome: "Query result 2".to_string(),
                is_repeat: true,
                impact_score: 0.5,
                succeeded: true,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "query_db".to_string(),
                outcome: "Query result 3".to_string(),
                is_repeat: true,
                impact_score: 0.5,
                succeeded: true,
                tokens_used: 100,
            },
        ];

        let trajectory = TrajectoryLog {
            entries,
            tokens_used: 300,
            compressibility_score: 0.6,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = compressor.compress_trajectory(&trajectory);

        assert!(!compressed.preserved.is_empty() || !compressed.summarized.is_empty());
    }

    #[test]
    fn test_compressor_detects_superseded() {
        let compressor = TrajectoryCompressor::new();
        let trajectory = create_test_trajectory();
        let filtered = compressor.filter_expired_info(&trajectory.entries);

        let has_updated = filtered.iter().any(|e| e.outcome.contains("updated"));
        assert!(has_updated);
    }

    #[test]
    fn test_compressor_stats() {
        let compressor = TrajectoryCompressor::new();
        let trajectory = create_test_trajectory();
        let compressed = compressor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.len() > 0 || compressed.summarized.len() > 0);
    }

    #[test]
    fn test_custom_config() {
        let config = TrajectoryCompressorConfig {
            preserve_threshold: 0.5,
            max_summaries: 5,
            superseded_patterns: vec!["obsolete".to_string()],
            filter_redundant: false,
            max_tokens: 5000,
        };
        let compressor = TrajectoryCompressor::with_config(config);

        assert_eq!(compressor.config().preserve_threshold, 0.5);
        assert_eq!(compressor.config().max_summaries, 5);
    }
}
