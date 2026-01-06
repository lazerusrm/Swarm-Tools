#[cfg(test)]
mod resource_allocation_tests {
    use super::*;
    use swarm_tools::enhanced_monitor::{EnhancedMonitor, ResourceManager};

    #[test]
    fn test_new_resource_manager_safety_reserve_calculation() {
        let total = 200_000u32;
        let monitor = EnhancedMonitor::new_resource_manager(total);

        let budget = monitor.get_budget().expect("Budget should be set");
        let expected_reserve = (total as f64 * 0.15) as u32;
        assert_eq!(budget.safety_reserve, expected_reserve);
    }

    #[test]
    fn test_track_usage_records_turn_stats() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("agent_1", 500, 0.8, 2);

        let history = monitor
            .agent_usage_history
            .get("agent_1")
            .expect("Should have history");
        assert_eq!(history.len(), 1);

        let turn = &history[0];
        assert_eq!(turn.turn_number, 0);
        assert_eq!(turn.tokens_used, 500);
        assert!((turn.contribution - 0.8).abs() < 0.001);
        assert_eq!(turn.tasks_completed, 2);
    }

    #[test]
    fn test_track_usage_multiple_turns() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("agent_1", 100, 0.5, 1);
        monitor.track_usage("agent_1", 200, 0.6, 2);
        monitor.track_usage("agent_1", 300, 0.7, 3);

        let history = monitor
            .agent_usage_history
            .get("agent_1")
            .expect("Should have history");
        assert_eq!(history.len(), 3);

        assert_eq!(history[0].turn_number, 0);
        assert_eq!(history[1].turn_number, 1);
        assert_eq!(history[2].turn_number, 2);
    }

    #[test]
    fn test_track_usage_limits_history_to_10_turns() {
        let mut monitor = EnhancedMonitor::new(200_000);

        for i in 0..15 {
            monitor.track_usage(&format!("agent_{}", i), 100, 0.5, 1);
        }

        for i in 0..15 {
            let history = monitor.agent_usage_history.get(&format!("agent_{}", i));
            assert!(history.is_none() || history.unwrap().len() <= 10);
        }
    }

    #[test]
    fn test_reallocate_budget_calculates_per_agent() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("agent_1", 100, 0.9, 2);
        monitor.track_usage("agent_2", 100, 0.3, 1);

        let allocation = monitor.reallocate_budget(100_000);

        assert!(allocation.per_agent > 0);
        assert!(allocation.safety_reserve > 0);
        assert!(!allocation.adjustments.is_empty());
    }

    #[test]
    fn test_reallocate_budget_high_contributor_flag() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("super_agent", 500, 0.85, 10);

        let allocation = monitor.reallocate_budget(100_000);

        let has_high_contributor = allocation
            .adjustments
            .iter()
            .any(|a| a.contains("High contributor"));
        assert!(has_high_contributor);
    }

    #[test]
    fn test_reallocate_budget_low_contributor_flag() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("lazy_agent", 50, 0.2, 0);

        let allocation = monitor.reallocate_budget(100_000);

        let has_low_contributor = allocation
            .adjustments
            .iter()
            .any(|a| a.contains("Potential prune"));
        assert!(has_low_contributor);
    }

    #[test]
    fn test_check_imbalance_returns_false_with_single_agent() {
        let monitor = EnhancedMonitor::new(200_000);

        assert!(!monitor.check_imbalance());
    }

    #[test]
    fn test_check_imbalance_returns_false_with_balanced_agents() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("agent_1", 100, 0.6, 2);
        monitor.track_usage("agent_2", 100, 0.62, 2);
        monitor.track_usage("agent_3", 100, 0.58, 2);

        assert!(!monitor.check_imbalance());
    }

    #[test]
    fn test_check_imbalance_returns_true_with_unbalanced_agents() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("agent_1", 100, 0.9, 5);
        monitor.track_usage("agent_2", 100, 0.1, 0);

        assert!(monitor.check_imbalance());
    }

    #[test]
    fn test_check_pruning_candidate_returns_none_for_new_agent() {
        let monitor = EnhancedMonitor::new(200_000);

        let result = monitor.check_pruning_candidate("new_agent");
        assert!(result.is_none());
    }

    #[test]
    fn test_check_pruning_candidate_returns_none_for_contributing_agent() {
        let mut monitor = EnhancedMonitor::new(200_000);

        for _ in 0..5 {
            monitor.track_usage("contributor", 500, 0.6, 2);
        }

        let result = monitor.check_pruning_candidate("contributor");
        assert!(result.is_none());
    }

    #[test]
    fn test_check_pruning_candidate_returns_some_for_low_contribution() {
        let mut monitor = EnhancedMonitor::new(200_000);

        for _ in 0..5 {
            monitor.track_usage("underperformer", 100, 0.2, 0);
        }

        let result = monitor.check_pruning_candidate("underperformer");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Potential topology change"));
    }

    #[test]
    fn test_check_pruning_candidate_checks_usage_rate() {
        let mut monitor = EnhancedMonitor::new(200_000);

        for _ in 0..5 {
            monitor.track_usage("low_usage", 100, 0.25, 1);
        }

        let result = monitor.check_pruning_candidate("low_usage");
        assert!(result.is_some());
    }

    #[test]
    fn test_budget_allocation_timestamp_is_valid() {
        let mut monitor = EnhancedMonitor::new(200_000);
        monitor.track_usage("agent_1", 100, 0.5, 1);

        let allocation = monitor.reallocate_budget(100_000);

        assert!(!allocation.timestamp.is_empty());
        assert!(allocation.timestamp.contains("T"));
    }

    #[test]
    fn test_contribution_sorted_allocation() {
        let mut monitor = EnhancedMonitor::new(200_000);

        monitor.track_usage("high_performer", 500, 0.9, 10);
        monitor.track_usage("mid_performer", 300, 0.6, 3);
        monitor.track_usage("low_performer", 100, 0.2, 0);

        let allocation = monitor.reallocate_budget(100_000);

        assert!(allocation.per_agent > 0);

        let has_high_note = allocation
            .adjustments
            .iter()
            .any(|a| a.contains("high_performer") && a.contains("High contributor"));
        let has_low_note = allocation
            .adjustments
            .iter()
            .any(|a| a.contains("low_performer") && a.contains("Potential prune"));

        assert!(has_high_note, "High performer should be flagged positively");
        assert!(
            has_low_note,
            "Low performer should be flagged for potential pruning"
        );
    }

    #[test]
    fn test_no_agents_allocation() {
        let mut monitor = EnhancedMonitor::new(200_000);
        let allocation = monitor.reallocate_budget(100_000);

        assert!(allocation.per_agent > 0);
        assert!(allocation.adjustments.is_empty());
    }
}
