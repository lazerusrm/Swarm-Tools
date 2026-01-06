use swarm_tools::enhanced_monitor::EnhancedMonitor;

#[test]
fn test_new_enhanced_monitor() {
    let monitor = EnhancedMonitor::new(3, 200000);
    assert_eq!(monitor.max_parallel, 3);
}

#[test]
fn test_track_token_usage() {
    let mut monitor = EnhancedMonitor::new(3, 200000);

    monitor.track_token_usage("agent_1", 1000);
    monitor.track_token_usage("agent_1", 2000);

    let stats = monitor.get_agent_stats("agent_1").unwrap();
    assert_eq!(stats.total_tokens, 3000);
}

#[test]
fn test_check_context_budget() {
    let mut monitor = EnhancedMonitor::new(3, 200000);

    monitor.track_token_usage("agent_1", 150000);

    let should_trigger = monitor.check_context_budget("agent_1").unwrap();
    assert!(should_trigger);
}

#[test]
fn test_get_system_stats() {
    let mut monitor = EnhancedMonitor::new(3, 200000);

    monitor.track_token_usage("agent_1", 10000);
    monitor.track_token_usage("agent_2", 20000);

    let system_stats = monitor.get_system_stats().unwrap();
    assert_eq!(system_stats.total_agents, 2);
}

#[test]
fn test_reset_stats() {
    let mut monitor = EnhancedMonitor::new(3, 200000);

    monitor.track_token_usage("agent_1", 10000);
    monitor.reset_agent_stats("agent_1");

    let stats = monitor.get_agent_stats("agent_1").unwrap();
    assert_eq!(stats.total_tokens, 0);
}
