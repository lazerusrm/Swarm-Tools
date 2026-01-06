use swarm_tools::communication_optimizer::CommunicationOptimizer;
use swarm_tools::types::CommunicationPriority;

#[test]
fn test_new_communication_optimizer() {
    let optimizer = CommunicationOptimizer::new();
    assert!(optimizer.cache_capacity > 0);
}

#[test]
fn test_analyze_communication_critical() {
    let optimizer = CommunicationOptimizer::new();

    let analysis = optimizer
        .analyze_communication("URGENT: System crash detected", "agent_1", "agent_2")
        .unwrap();

    assert_eq!(analysis.priority, CommunicationPriority::Critical);
}

#[test]
fn test_analyze_communication_redundant() {
    let optimizer = CommunicationOptimizer::new();

    let message1 = "Please review authentication module";
    let message2 = "Please review authentication module";

    optimizer
        .analyze_communication(message1, "agent_1", "agent_2")
        .unwrap();
    let analysis2 = optimizer
        .analyze_communication(message2, "agent_1", "agent_2")
        .unwrap();

    assert_eq!(analysis2.priority, CommunicationPriority::Redundant);
}

#[test]
fn test_filter_redundant_messages() {
    let mut optimizer = CommunicationOptimizer::new();

    let communications = vec![
        ("Same message", "agent_1", "agent_2"),
        ("Same message", "agent_1", "agent_2"),
        ("Different message", "agent_1", "agent_2"),
    ];

    let filtered = optimizer.filter_redundant(&communications);

    assert!(filtered.len() < communications.len());
}

#[test]
fn test_get_relevance_score() {
    let optimizer = CommunicationOptimizer::new();

    let score1 = optimizer.get_relevance_score(
        "Authentication module review needed",
        "Review authentication for security issues",
    );

    let score2 = optimizer.get_relevance_score(
        "Authentication module review needed",
        "Fix UI bug in login page",
    );

    assert!(score1 > score2);
}
