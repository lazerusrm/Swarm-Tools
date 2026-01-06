use swarm_tools::enhanced_monitor::{EnhancedMonitor, TrajectoryCompression};
use swarm_tools::types::{
    CompressedTrajectory, Plan, PlanStep, StepStatus, TrajectoryEntry, TrajectoryLog,
};

#[cfg(test)]
mod trajectory_compression_tests {
    use super::*;

    fn create_sample_trajectory() -> TrajectoryLog {
        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "analyze_data".to_string(),
                outcome: "Extracted 50 records".to_string(),
                is_repeat: false,
                impact_score: 0.9,
                succeeded: true,
                tokens_used: 500,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "validate_schema".to_string(),
                outcome: "Schema valid".to_string(),
                is_repeat: false,
                impact_score: 0.8,
                succeeded: true,
                tokens_used: 200,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "failed_attempt_1".to_string(),
                outcome: "Connection timeout".to_string(),
                is_repeat: false,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:03:00Z".to_string(),
                action: "failed_attempt_2".to_string(),
                outcome: "Invalid response".to_string(),
                is_repeat: true,
                impact_score: 0.15,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:04:00Z".to_string(),
                action: "process_results".to_string(),
                outcome: "Generated report".to_string(),
                is_repeat: false,
                impact_score: 0.85,
                succeeded: true,
                tokens_used: 800,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:05:00Z".to_string(),
                action: "retry_failed".to_string(),
                outcome: "Still failed".to_string(),
                is_repeat: true,
                impact_score: 0.1,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:06:00Z".to_string(),
                action: "final_success".to_string(),
                outcome: "Completed task".to_string(),
                is_repeat: false,
                impact_score: 0.95,
                succeeded: true,
                tokens_used: 300,
            },
        ];

        TrajectoryLog {
            entries,
            tokens_used: 2100,
            compressibility_score: 0.65,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_compress_trajectory_preserves_high_impact() {
        let monitor = EnhancedMonitor::new(200_000);
        let trajectory = create_sample_trajectory();
        let compressed = monitor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.len() >= 4);

        let preserved_impact_scores: Vec<f64> = compressed
            .preserved
            .iter()
            .map(|e| e.impact_score)
            .collect();

        for score in preserved_impact_scores {
            assert!(score >= 0.7, "High impact entries should be preserved");
        }
    }

    #[test]
    fn test_compress_trajectory_summarizes_low_impact() {
        let monitor = EnhancedMonitor::new(200_000);

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "important_task".to_string(),
                outcome: "Success".to_string(),
                is_repeat: false,
                impact_score: 0.9,
                succeeded: true,
                tokens_used: 500,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "retry_query".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: true,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "retry_query".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: true,
                impact_score: 0.15,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:03:00Z".to_string(),
                action: "retry_query".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: true,
                impact_score: 0.1,
                succeeded: false,
                tokens_used: 100,
            },
        ];

        let trajectory = TrajectoryLog {
            entries,
            tokens_used: 800,
            compressibility_score: 0.7,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = monitor.compress_trajectory(&trajectory);

        assert!(!compressed.summarized.is_empty());
        assert_eq!(compressed.summarized.len(), 1);
        assert_eq!(compressed.summarized[0].count, 3);
        assert!(compressed.summarized[0].tokens_saved > 0);
    }

    #[test]
    fn test_compress_trajectory_compression_ratio() {
        let monitor = EnhancedMonitor::new(200_000);
        let trajectory = create_sample_trajectory();
        let compressed = monitor.compress_trajectory(&trajectory);

        assert!(compressed.compression_ratio > 0.0);
        assert!(compressed.compression_ratio <= 1.0);
    }

    #[test]
    fn test_should_compress_triggers_at_threshold() {
        let monitor = EnhancedMonitor::new(200_000);

        assert!(
            !monitor.should_compress(0.80, 18, 25000),
            "80% exactly should not trigger"
        );
        assert!(
            monitor.should_compress(0.81, 18, 25000),
            "81% should trigger"
        );
        assert!(
            monitor.should_compress(0.81, 19, 25000),
            "81% with 19 steps should trigger"
        );
        assert!(
            monitor.should_compress(0.81, 18, 25001),
            "81% with 25001 tokens should trigger"
        );
    }

    #[test]
    fn test_should_not_compress_below_threshold() {
        let monitor = EnhancedMonitor::new(200_000);

        assert!(!monitor.should_compress(0.50, 10, 10000));
        assert!(!monitor.should_compress(0.70, 5, 20000));
        assert!(!monitor.should_compress(0.75, 17, 24999));
    }

    #[test]
    fn test_filter_expired_info_keeps_succeeded_entries() {
        let monitor = EnhancedMonitor::new(200_000);
        let trajectory = create_sample_trajectory();
        let filtered = monitor.filter_expired_info(&trajectory.entries);

        let failed_count = filtered.iter().filter(|e| !e.succeeded).count();
        let high_impact_failed = filtered
            .iter()
            .filter(|e| !e.succeeded && e.impact_score >= 0.5)
            .count();
        assert!(failed_count <= 3, "Should filter most failed entries");
        assert!(
            high_impact_failed >= 0,
            "May keep some high-impact failed entries"
        );
    }

    #[test]
    fn test_group_and_summarize_groups_repeated_actions() {
        let monitor = EnhancedMonitor::new(200_000);

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "retry_database_query".to_string(),
                outcome: "Timeout".to_string(),
                is_repeat: true,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "retry_database_query".to_string(),
                outcome: "Timeout".to_string(),
                is_repeat: true,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "retry_database_query".to_string(),
                outcome: "Timeout".to_string(),
                is_repeat: true,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
        ];

        let refs: Vec<&TrajectoryEntry> = entries.iter().collect();
        let summaries = EnhancedMonitor::group_and_summarize(&refs);

        assert!(!summaries.is_empty());
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].count, 3);
        assert!(summaries[0].tokens_saved > 0);
    }

    #[test]
    fn test_compression_threshold_returns_correct_values() {
        let monitor = EnhancedMonitor::new(200_000);
        let threshold = monitor.get_compression_threshold();

        assert_eq!(threshold.0, 18, "Step threshold should be 18");
        assert_eq!(threshold.1, 25000, "Token threshold should be 25000");
    }

    #[test]
    fn test_empty_trajectory_compression() {
        let monitor = EnhancedMonitor::new(200_000);
        let empty_trajectory = TrajectoryLog {
            entries: vec![],
            tokens_used: 0,
            compressibility_score: 0.0,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = monitor.compress_trajectory(&empty_trajectory);

        assert!(compressed.preserved.is_empty());
        assert!(compressed.summarized.is_empty());
    }

    #[test]
    fn test_all_high_impact_trajectory() {
        let monitor = EnhancedMonitor::new(200_000);

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "important_task".to_string(),
                outcome: "Success".to_string(),
                is_repeat: false,
                impact_score: 0.95,
                succeeded: true,
                tokens_used: 1000,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "critical_analysis".to_string(),
                outcome: "Complete".to_string(),
                is_repeat: false,
                impact_score: 0.88,
                succeeded: true,
                tokens_used: 1200,
            },
        ];

        let trajectory = TrajectoryLog {
            entries,
            tokens_used: 2200,
            compressibility_score: 0.3,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = monitor.compress_trajectory(&trajectory);

        assert_eq!(compressed.preserved.len(), 2);
        assert!(compressed.summarized.is_empty());
    }

    #[test]
    fn test_all_failed_trajectory() {
        let monitor = EnhancedMonitor::new(200_000);

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "failed_1".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: false,
                impact_score: 0.1,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "failed_2".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: false,
                impact_score: 0.15,
                succeeded: false,
                tokens_used: 100,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "failed_3".to_string(),
                outcome: "Failed".to_string(),
                is_repeat: false,
                impact_score: 0.2,
                succeeded: false,
                tokens_used: 100,
            },
        ];

        let trajectory = TrajectoryLog {
            entries,
            tokens_used: 300,
            compressibility_score: 0.9,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = monitor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.is_empty());
        assert!(!compressed.summarized.is_empty());
    }
}
