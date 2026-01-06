use serde_json::json;
use swarm_tools::cost_benefit::CostBenefitAnalyzer;
use swarm_tools::types::CostBenefitResult;

#[test]
fn test_new_analyzer() {
    let analyzer = CostBenefitAnalyzer::new();
    let stats = analyzer.get_decision_stats();

    assert_eq!(stats.total_decisions, 0);
    assert_eq!(stats.by_type.len(), 0);
}

#[test]
fn test_estimate_cost_low_complexity() {
    let analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "complexity": "low",
        "tokens": 1000,
        "time_estimated": 60
    });

    let cost = analyzer.estimate_cost(&action).unwrap();
    assert!(cost > 0.0);
    assert!(cost < 1000.0);
}

#[test]
fn test_estimate_cost_high_complexity() {
    let analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "complexity": "high",
        "tokens": 10000,
        "time_estimated": 600
    });

    let cost = analyzer.estimate_cost(&action).unwrap();
    assert!(cost > 0.0);
}

#[test]
fn test_make_decision_skip() {
    let mut analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "complexity": "high",
        "tokens": 50000,
        "time_estimated": 1800,
        "accuracy": 0.5,
        "time_saved": 100,
        "completion_rate": 0.7,
        "information_gain": 0.5
    });

    let result = analyzer.make_decision(action).unwrap();
    assert!(result.decision.len() > 0);
    assert!(result.message.len() > 0);
    assert!(result.cost > 0.0);
    assert!(result.benefit > 0.0);
    assert!(result.ratio > 0.0);
}

#[test]
fn test_estimate_benefit_with_accuracy() {
    let analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "accuracy": 0.95,
        "time_saved": 120,
        "completion_rate": 1.0
    });

    let benefit = analyzer.estimate_benefit(&action).unwrap();
    assert!(benefit > 0.0);
}

#[test]
fn test_make_decision_execute() {
    let mut analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "complexity": "low",
        "tokens": 1000,
        "time_estimated": 60,
        "accuracy": 0.95,
        "time_saved": 300,
        "completion_rate": 1.0,
        "information_gain": 0.8
    });

    let result = analyzer.make_decision(action).unwrap();
    assert_eq!(result.decision, "execute");
    assert!(result.ratio > 1.0);
    assert!(result.message.len() > 0);
}

#[test]
fn test_record_actual() {
    let mut analyzer = CostBenefitAnalyzer::new();

    let action = json!({
        "complexity": "low",
        "tokens": 1000,
        "time_estimated": 60,
        "accuracy": 0.95,
        "time_saved": 300,
        "completion_rate": 1.0,
        "information_gain": 0.8
    });

    let result = analyzer.make_decision(action).unwrap();

    analyzer.record_actual(
        format!("action_{}", chrono::Utc::now().timestamp()),
        50.0,
        100.0,
    );

    let stats = analyzer.get_decision_stats();
    assert_eq!(stats.total_decisions, 1);
}

#[test]
fn test_multiple_decisions() {
    let mut analyzer = CostBenefitAnalyzer::new();

    for i in 0..5 {
        let action = json!({
            "complexity": "low",
            "tokens": 1000 + i * 100,
            "time_estimated": 60,
            "accuracy": 0.95,
            "time_saved": 300,
            "completion_rate": 1.0,
            "information_gain": 0.8
        });

        let _ = analyzer.make_decision(action);
        analyzer.record_actual(format!("action_{}", i), 50.0, 100.0);
    }

    let stats = analyzer.get_decision_stats();
    assert_eq!(stats.total_decisions, 5);
}

#[test]
fn test_decision_serialization() {
    let mut analyzer = CostBenefitAnalyzer::new();
    let action = json!({
        "complexity": "low",
        "tokens": 1000,
        "time_estimated": 60,
        "accuracy": 0.95,
        "time_saved": 300,
        "completion_rate": 1.0,
        "information_gain": 0.8
    });

    let result = analyzer.make_decision(action).unwrap();

    let json_str = serde_json::to_string(&result).unwrap();
    assert!(json_str.len() > 0);

    let deserialized: CostBenefitResult = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.decision, result.decision);
    assert!((deserialized.cost - result.cost).abs() < 0.001);
    assert!((deserialized.benefit - result.benefit).abs() < 0.001);
}

#[test]
fn test_cost_benefit_ratio() {
    let mut analyzer = CostBenefitAnalyzer::new();

    let low_cost = json!({
        "complexity": "low",
        "tokens": 1000,
        "time_estimated": 60,
        "accuracy": 0.95,
        "time_saved": 300,
        "completion_rate": 1.0,
        "information_gain": 0.8
    });

    let low_cost_result = analyzer.make_decision(low_cost).unwrap();

    assert!(low_cost_result.ratio > 0.0);
    assert!(low_cost_result.cost > 0.0);
    assert!(low_cost_result.benefit > 0.0);
}
