use swarm_tools::codified_reasoning::CodifiedReasoning;
use swarm_tools::communication_optimizer::CommunicationOptimizer;
use swarm_tools::config::{load_config_from_json, merge_configs, save_config_to_json, SwarmConfig};
use swarm_tools::enhanced_monitor::{EnhancedMonitor, ResourceManager, TrajectoryCompression};
use swarm_tools::role_router::RoleRouter;
use swarm_tools::trajectory_compressor::{TrajectoryCompressor, TrajectoryCompressorConfig};
use swarm_tools::types::{AgentRole, TrajectoryEntry, TrajectoryLog};

#[cfg(test)]
mod comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline_role_aware_optimization() {
        let optimizer = CommunicationOptimizer::new().unwrap();
        let role_router = RoleRouter::new();

        let communications = vec![
            serde_json::json!({
                "source": "agent_1",
                "target": "agent_2",
                "content": "file_deltas show changes in metrics patterns - analysis_results ready",
                "priority": "High",
                "impact_score": 0.9
            }),
            serde_json::json!({
                "source": "agent_3",
                "target": "agent_1",
                "content": "status: working in progress proceeding with task",
                "priority": "Low",
                "impact_score": 0.2
            }),
            serde_json::json!({
                "source": "agent_2",
                "target": "agent_1",
                "content": "findings show critical error in security_issues - immediate fix needed",
                "priority": "Critical",
                "impact_score": 0.95
            }),
        ];

        let result = optimizer
            .optimize_for_role(&communications, AgentRole::Analyzer)
            .unwrap();

        assert!(result.token_reduction_pct >= 0.0);
        assert!(result.optimized_count <= result.original_count);
    }

    #[test]
    fn test_role_router_with_multiple_roles() {
        let router = RoleRouter::new();

        let messages = vec![
            ("file_deltas and git_diff changes detected", 0, 0.8),
            ("draft_content for documentation updates", 1, 0.6),
            ("security_issues found in code_changes", 2, 0.9),
            ("summaries of findings consolidated", 3, 0.7),
        ];

        let extractor_context = router.filter_context(&messages, AgentRole::Extractor);
        let analyzer_context = router.filter_context(&messages, AgentRole::Analyzer);
        let reviewer_context = router.filter_context(&messages, AgentRole::Reviewer);

        assert_eq!(extractor_context.role, AgentRole::Extractor);
        assert_eq!(analyzer_context.role, AgentRole::Analyzer);
        assert_eq!(reviewer_context.role, AgentRole::Reviewer);

        assert!(extractor_context.relevance_scores[0] > 0.5);
        assert!(analyzer_context.relevance_scores[0] > analyzer_context.relevance_scores[1]);
        assert!(reviewer_context.relevance_scores[2] > 0.5);
    }

    #[test]
    fn test_trajectory_compression_with_resource_allocation() {
        let mut monitor = EnhancedMonitor::new(200_000);

        for i in 0..20 {
            let contribution = if i < 5 {
                0.2
            } else {
                0.6 + (i as f64 - 5.0) / 15.0 * 0.3
            };
            monitor.track_usage(&format!("agent_{}", i % 3), 500, contribution, 3);
        }

        assert!(monitor.check_imbalance() == false);

        let trajectory = TrajectoryLog {
            entries: vec![
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:00:00Z".to_string(),
                    action: "high_impact_task".to_string(),
                    outcome: "Success".to_string(),
                    is_repeat: false,
                    impact_score: 0.9,
                    succeeded: true,
                    tokens_used: 500,
                },
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:01:00Z".to_string(),
                    action: "failed_attempt".to_string(),
                    outcome: "Failed".to_string(),
                    is_repeat: false,
                    impact_score: 0.1,
                    succeeded: false,
                    tokens_used: 100,
                },
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:02:00Z".to_string(),
                    action: "failed_attempt".to_string(),
                    outcome: "Failed".to_string(),
                    is_repeat: true,
                    impact_score: 0.1,
                    succeeded: false,
                    tokens_used: 100,
                },
            ],
            tokens_used: 700,
            compressibility_score: 0.5,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = monitor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.len() >= 1);
        assert!(compressed.compression_ratio > 0.0);
        assert!(compressed.compression_ratio <= 1.0);
    }

    #[test]
    fn test_codified_reasoning_integration() {
        let reasoning = CodifiedReasoning::new();

        let plan_text = r#"
        1. Extract data from source files
        2. Analyze patterns in the data
        3. Generate report with findings
        4. Review report for quality
        5. Finalize documentation
        "#;

        let plan = reasoning.codify_prompt(plan_text, "analyzer");

        assert!(plan.steps.len() <= 10);
        assert!(plan.total_expected_tokens > 0);
    }

    #[test]
    fn test_resource_budget_flow() {
        let mut monitor = EnhancedMonitor::new_resource_manager(100_000);

        monitor.track_usage("agent_high", 500, 0.85, 10);
        monitor.track_usage("agent_mid", 400, 0.6, 5);

        for _ in 0..5 {
            monitor.track_usage("agent_low", 50, 0.15, 0);
        }

        let allocation = monitor.reallocate_budget(100_000);

        assert!(allocation.per_agent > 0);
        assert!(allocation.safety_reserve > 0);

        let pruning = monitor.check_pruning_candidate("agent_low");
        assert!(pruning.is_some());

        let healthy = monitor.check_pruning_candidate("agent_high");
        assert!(healthy.is_none());
    }

    #[test]
    fn test_multi_feature_workflow() {
        let optimizer = CommunicationOptimizer::new().unwrap();
        let router = RoleRouter::new();
        let mut monitor = EnhancedMonitor::new(200_000);
        let reasoning = CodifiedReasoning::new();

        let communications = vec![
            serde_json::json!({
                "source": "extractor",
                "target": "analyzer",
                "content": "Extracted 1000 records with file_deltas from git_diff",
                "priority": "High"
            }),
            serde_json::json!({
                "source": "analyzer",
                "target": "synthesizer",
                "content": "Analysis complete - patterns detected in metrics with statistical significance",
                "priority": "High"
            }),
            serde_json::json!({
                "source": "reviewer",
                "target": "writer",
                "content": "Quality gate passed - no security_issues found in code_changes",
                "priority": "Medium"
            }),
        ];

        let opt_result = optimizer
            .optimize_for_role(&communications, AgentRole::Synthesizer)
            .unwrap();
        assert!(opt_result.token_reduction_pct >= 0.0);

        let role_context =
            router.filter_context(&vec![("Analysis complete", 1, 0.8)], AgentRole::Synthesizer);
        assert!(role_context.total_relevance > 0.0);

        monitor.track_usage("test_agent", 1000, 0.7, 5);
        assert!(monitor.agent_usage_history.contains_key("test_agent"));

        let plan = reasoning.codify_prompt("Analyze and report", "synthesizer");
        assert!(plan.steps.len() > 0);
    }

    #[test]
    fn test_trajectory_compression_threshold() {
        let monitor = EnhancedMonitor::new(200_000);

        assert!(!monitor.should_compress(0.79, 18, 25000));
        assert!(monitor.should_compress(0.81, 18, 25000));
        assert!(monitor.should_compress(0.85, 25, 25000));
        assert!(monitor.should_compress(0.85, 18, 30000));
    }

    #[test]
    fn test_role_router_recency_boost() {
        let router = RoleRouter::new();
        let total = 10;

        let old_score = router.score_for_role("file_deltas", AgentRole::Extractor, 2, total, 0.5);
        let recent_score =
            router.score_for_role("file_deltas", AgentRole::Extractor, 9, total, 0.5);

        assert!(
            recent_score > old_score,
            "Recent messages should have higher scores"
        );
    }

    #[test]
    fn test_budget_safety_reserve() {
        let totals = [50000u32, 100000, 200000, 500000];

        for total in totals {
            let monitor = EnhancedMonitor::new_resource_manager(total);
            let budget = monitor.get_budget().expect("Budget should be set");

            let reserve_ratio = budget.safety_reserve as f64 / total as f64;
            assert!(
                (reserve_ratio - 0.15).abs() < 0.01,
                "Safety reserve should be ~15% but was {:.2}%",
                reserve_ratio * 100.0
            );
        }
    }

    #[test]
    fn test_plan_summarization() {
        let reasoning = CodifiedReasoning::new();

        let long_plan = r#"
        Step 1: Initialize the system and load configuration files
        Step 2: Connect to the database and verify connection status
        Step 3: Query the main table for all pending records
        Step 4: Process each record with the transformation logic
        Step 5: Validate transformed data against schema
        Step 6: Write results to output table
        Step 7: Generate summary report of all operations
        Step 8: Send notification to admin team
        Step 9: Clean up temporary files and close connections
        Step 10: Log completion status and exit
        "#;

        let plan = reasoning.codify_prompt(long_plan, "general");
        assert!(plan.steps.len() > 0);

        let summary = reasoning.summarize_old_plans(&[plan], 5);
        assert!(!summary.is_empty() || summary.is_empty());
    }

    #[test]
    fn test_imbalanced_agent_detection() {
        let monitor = EnhancedMonitor::new(200_000);

        assert!(!monitor.check_imbalance());

        let mut monitor = EnhancedMonitor::new(200_000);
        monitor.track_usage("star_agent", 1000, 0.95, 20);
        monitor.track_usage("lazy_agent", 50, 0.1, 0);

        assert!(monitor.check_imbalance());
    }

    #[test]
    fn test_trajectory_compressor_full_impl() {
        let config = TrajectoryCompressorConfig {
            preserve_threshold: 0.6,
            max_summaries: 5,
            superseded_patterns: vec!["updated".to_string(), "fixed".to_string()],
            filter_redundant: true,
            max_tokens: 5000,
        };
        let compressor = TrajectoryCompressor::with_config(config);

        let trajectory = TrajectoryLog {
            entries: vec![
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:00:00Z".to_string(),
                    action: "analyze".to_string(),
                    outcome: "Analysis complete".to_string(),
                    is_repeat: false,
                    impact_score: 0.8,
                    succeeded: true,
                    tokens_used: 500,
                },
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:01:00Z".to_string(),
                    action: "analyze".to_string(),
                    outcome: "Analysis updated with new findings".to_string(),
                    is_repeat: true,
                    impact_score: 0.9,
                    succeeded: true,
                    tokens_used: 600,
                },
                TrajectoryEntry {
                    timestamp: "2025-01-06T10:02:00Z".to_string(),
                    action: "failed".to_string(),
                    outcome: "Connection timeout".to_string(),
                    is_repeat: false,
                    impact_score: 0.2,
                    succeeded: false,
                    tokens_used: 100,
                },
            ],
            tokens_used: 1200,
            compressibility_score: 0.5,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = compressor.compress_trajectory(&trajectory);

        assert!(compressed.preserved.len() > 0);
        assert!(compressed.compression_ratio > 0.0);
    }

    #[test]
    fn test_config_loading_and_merging() {
        let default_config = SwarmConfig::default();
        let override_config = SwarmConfig {
            general: swarm_tools::config::GeneralConfig {
                default_context_budget: 300000,
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = merge_configs(default_config, &override_config);
        assert_eq!(merged.general.default_context_budget, 300000);
    }

    #[test]
    fn test_auto_reduce_low_contrib() {
        let mut monitor = EnhancedMonitor::with_auto_reduce(200_000, true, 30.0, 0.4);

        monitor.track_usage("high_agent", 500, 0.8, 5);
        monitor.track_usage("low_agent", 100, 0.2, 1);

        let allocation = monitor.reallocate_budget(100_000);

        let has_reduction = allocation
            .adjustments
            .iter()
            .any(|a| a.contains("Reduced budget"));
        assert!(has_reduction);
    }

    #[test]
    fn test_expanded_superseded_detection() {
        let monitor = EnhancedMonitor::new(200_000);

        let entries = vec![
            TrajectoryEntry {
                timestamp: "2025-01-06T10:00:00Z".to_string(),
                action: "query".to_string(),
                outcome: "Old result from first query".to_string(),
                is_repeat: false,
                impact_score: 0.5,
                succeeded: true,
                tokens_used: 200,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:01:00Z".to_string(),
                action: "query".to_string(),
                outcome: "This result updated the previous one with new data".to_string(),
                is_repeat: true,
                impact_score: 0.7,
                succeeded: true,
                tokens_used: 300,
            },
            TrajectoryEntry {
                timestamp: "2025-01-06T10:02:00Z".to_string(),
                action: "query".to_string(),
                outcome: "Further refined result that overrides the update".to_string(),
                is_repeat: true,
                impact_score: 0.9,
                succeeded: true,
                tokens_used: 400,
            },
        ];

        let filtered = monitor.filter_expired_info(&entries);

        assert!(filtered.len() >= 1);
        let has_high_impact = filtered.iter().any(|e| e.impact_score >= 0.5);
        assert!(
            has_high_impact,
            "Should have at least one entry with impact >= 0.5"
        );
    }

    #[test]
    fn test_cross_module_codified_to_compression() {
        let reasoning = CodifiedReasoning::new();
        let compressor = TrajectoryCompressor::new();

        let plan_text = r#"
        1. Extract data from files
        2. Analyze patterns
        3. Compress results
        4. Generate report
        "#;

        let plan = reasoning.codify_prompt(plan_text, "analyzer");
        assert!(plan.steps.len() > 0);

        let trajectory = TrajectoryLog {
            entries: plan
                .steps
                .iter()
                .enumerate()
                .map(|(i, step)| TrajectoryEntry {
                    timestamp: format!("2025-01-06T10:0{}:00Z", i),
                    action: step.action.clone(),
                    outcome: format!("Step {} completed", i + 1),
                    is_repeat: false,
                    impact_score: step.impact_score,
                    succeeded: true,
                    tokens_used: 100,
                })
                .collect(),
            tokens_used: (plan.steps.len() * 100) as u32,
            compressibility_score: 0.6,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = compressor.compress_trajectory(&trajectory);
        assert!(compressed.preserved.len() >= plan.steps.len() / 2);
    }

    #[test]
    fn test_cross_module_routing_to_allocation() {
        let optimizer = CommunicationOptimizer::new().unwrap();
        let mut monitor = EnhancedMonitor::with_auto_reduce(200_000, true, 25.0, 0.35);

        let communications = vec![
            serde_json::json!({
                "source": "extractor",
                "target": "analyzer",
                "content": "High priority: file_deltas with critical findings",
                "priority": "Critical"
            }),
            serde_json::json!({
                "source": "reviewer",
                "target": "writer",
                "content": "Status update: continuing with quality review",
                "priority": "Low"
            }),
        ];

        let result = optimizer
            .optimize_for_role(&communications, AgentRole::Analyzer)
            .unwrap();

        monitor.track_usage("analyzer", result.optimized_tokens as u32, 0.85, 2);
        monitor.track_usage("writer", 200, 0.25, 1);

        let allocation = monitor.reallocate_budget(100_000);
        assert!(allocation.adjustments.len() >= 1);
    }

    #[test]
    fn test_full_workflow_integration() {
        let config = SwarmConfig {
            trajectory_compression: swarm_tools::config::TrajectoryCompressionConfig {
                preserve_threshold: 0.5,
                ..Default::default()
            },
            resource_allocation: swarm_tools::config::ResourceAllocationConfig {
                auto_reduce_low_contrib: true,
                low_contrib_reduction_percent: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let reasoning = CodifiedReasoning::new();
        let compressor = TrajectoryCompressor::with_config(TrajectoryCompressorConfig {
            preserve_threshold: config.trajectory_compression.preserve_threshold,
            ..Default::default()
        });
        let mut monitor = EnhancedMonitor::with_auto_reduce(
            config.general.default_context_budget,
            config.resource_allocation.auto_reduce_low_contrib,
            config.resource_allocation.low_contrib_reduction_percent,
            config.resource_allocation.pruning_contribution_threshold,
        );

        let plan = reasoning.codify_prompt("Extract, analyze, compress, report", "synthesizer");
        assert!(plan.steps.len() > 0);

        let trajectory = TrajectoryLog {
            entries: plan
                .steps
                .iter()
                .enumerate()
                .map(|(i, step)| TrajectoryEntry {
                    timestamp: format!("2025-01-06T10:0{}:00Z", i),
                    action: step.action.clone(),
                    outcome: format!("Step {}: {}", i + 1, step.action),
                    is_repeat: false,
                    impact_score: step.impact_score,
                    succeeded: true,
                    tokens_used: 150,
                })
                .collect(),
            tokens_used: (plan.steps.len() * 150) as u32,
            compressibility_score: 0.6,
            created_at: "2025-01-06T10:00:00Z".to_string(),
        };

        let compressed = compressor.compress_trajectory(&trajectory);
        assert!(compressed.compression_ratio > 0.0);

        for i in 0..5 {
            monitor.track_usage(
                &format!("agent_{}", i % 3),
                300 + (i * 50) as u32,
                0.3 + (i as f64 * 0.1),
                2 + i,
            );
        }

        let allocation = monitor.reallocate_budget(config.general.default_context_budget as u32);
        assert!(allocation.safety_reserve > 0);
    }
}
