use swarm_tools::team_optimizer::TeamOptimizer;
use swarm_tools::types::TaskComplexity;

#[test]
fn test_new_team_optimizer() {
    let optimizer = TeamOptimizer::new(3, 200000);
    assert_eq!(optimizer.max_parallel, 3);
}

#[test]
fn test_analyze_task_complexity() {
    let optimizer = TeamOptimizer::new(3, 200000);

    let simple_task = "Write a simple function";
    let complex_task = "Design and implement a complex distributed system";

    let simple_analysis = optimizer.analyze_task(simple_task).unwrap();
    let complex_analysis = optimizer.analyze_task(complex_task).unwrap();

    assert!(simple_analysis.complexity <= complex_analysis.complexity);
}

#[test]
fn test_compose_team() {
    let optimizer = TeamOptimizer::new(3, 200000);

    let task = "Design authentication system for web application";
    let composition = optimizer.compose_team(task).unwrap();

    assert!(composition.team_size > 0);
    assert!(composition.roles.len() > 0);
}

#[test]
fn test_distribute_workload() {
    let optimizer = TeamOptimizer::new(3, 200000);

    let task = "Design authentication system";
    let composition = optimizer.compose_team(task).unwrap();

    let workloads = &composition.workload_distribution;
    assert!(workloads.len() > 0);
}

#[test]
fn test_team_size_constraint() {
    let optimizer = TeamOptimizer::new(2, 200000);

    let task = "Simple code review";
    let composition = optimizer.compose_team(task).unwrap();

    assert!(composition.team_size <= 2);
}
