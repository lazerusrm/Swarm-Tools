use swarm_tools::parallel_execution::ParallelManager;

#[test]
fn test_new_parallel_manager() {
    let manager = ParallelManager::new(3, 200000);
    assert_eq!(manager.max_parallel, 3);
}

#[test]
fn test_execute_tasks_parallel() {
    let mut manager = ParallelManager::new(3, 200000);

    let tasks = vec![("task1", 10000), ("task2", 8000), ("task3", 12000)];

    let results = manager.execute_tasks_parallel(&tasks).unwrap();

    assert_eq!(results.len(), 3);
}

#[test]
fn test_context_budget_enforcement() {
    let mut manager = ParallelManager::new(3, 100000);

    let tasks = vec![("large_task", 90000), ("small_task", 5000)];

    let results = manager.execute_tasks_parallel(&tasks).unwrap();

    let total_tokens: usize = results.iter().map(|r| r.tokens_used).sum();
    assert!(total_tokens <= 100000);
}

#[test]
fn test_max_parallel_limit() {
    let mut manager = ParallelManager::new(2, 200000);

    let tasks = vec![
        ("task1", 10000),
        ("task2", 8000),
        ("task3", 12000),
        ("task4", 15000),
    ];

    let results = manager.execute_tasks_parallel(&tasks).unwrap();

    assert!(results.len() <= 4);
}

#[test]
fn test_empty_tasks() {
    let mut manager = ParallelManager::new(3, 200000);

    let tasks: Vec<(&str, usize)> = vec![];

    let results = manager.execute_tasks_parallel(&tasks).unwrap();

    assert_eq!(results.len(), 0);
}
