use swarm_tools::omac_optimizer::OMACOptimizer;
use swarm_tools::types::OMACResult;

#[test]
fn test_new_omac_optimizer() {
    let optimizer = OMACOptimizer::new(3, 200000);
    assert_eq!(optimizer.max_parallel, 3);
}

#[test]
fn test_optimize_execution_basic() {
    let mut optimizer = OMACOptimizer::new(3, 200000);

    let tasks = vec![
        ("task1", 10000, 0.9),
        ("task2", 8000, 0.8),
        ("task3", 12000, 0.95),
    ];

    let result = optimizer.optimize_execution(&tasks).unwrap();
    assert!(result.tasks_to_execute.len() > 0);
}

#[test]
fn test_context_budget_adherence() {
    let mut optimizer = OMACOptimizer::new(3, 100000);

    let tasks = vec![("large_task", 90000, 0.9), ("medium_task", 5000, 0.8)];

    let result = optimizer.optimize_execution(&tasks).unwrap();
    assert!(result.total_tokens <= 100000);
}

#[test]
fn test_priority_sorting() {
    let mut optimizer = OMACOptimizer::new(3, 200000);

    let tasks = vec![
        ("low_priority", 5000, 0.5),
        ("high_priority", 5000, 0.95),
        ("medium_priority", 5000, 0.8),
    ];

    let result = optimizer.optimize_execution(&tasks).unwrap();
    assert!(result.tasks_to_execute.len() <= 3);
}

#[test]
fn test_empty_tasks() {
    let mut optimizer = OMACOptimizer::new(3, 200000);
    let tasks: Vec<(&str, usize, f64)> = vec![];

    let result = optimizer.optimize_execution(&tasks).unwrap();
    assert_eq!(result.tasks_to_execute.len(), 0);
}
