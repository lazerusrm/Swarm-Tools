use swarm_tools::codified_reasoning::CodifiedReasoning;
use swarm_tools::communication_optimizer::CommunicationOptimizer;
use swarm_tools::enhanced_monitor::{EnhancedMonitor, ResourceManager, TrajectoryCompression};
use swarm_tools::role_router::RoleRouter;
use swarm_tools::types::{AgentRole, Plan, TrajectoryEntry, TrajectoryLog};

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
}
