use swarm_tools::iterative_refinement::IterativeRefinement;
use swarm_tools::types::QualityTrend;

#[test]
fn test_new_iterative_refinement() {
    let refinement = IterativeRefinement::new(3, 200000);
    assert_eq!(refinement.max_iterations, 3);
}

#[test]
fn test_refine_prompt_improvement() {
    let mut refinement = IterativeRefinement::new(5, 200000);

    let initial_prompt = "Analyze the code";
    let result = refinement.refine_prompt(initial_prompt).unwrap();

    assert_eq!(result.iterations.len(), 1);
    assert!(result.quality_score > 0.0);
}

#[test]
fn test_quality_trend_improving() {
    let refinement = IterativeRefinement::new(5, 200000);

    let improvements = vec![0.1, 0.2, 0.15, 0.25, 0.3];
    let trend = refinement.analyze_quality_trend(&improvements);

    assert_eq!(trend, QualityTrend::Improving);
}

#[test]
fn test_quality_trend_declining() {
    let refinement = IterativeRefinement::new(5, 200000);

    let improvements = vec![0.3, 0.2, 0.15, 0.1, 0.05];
    let trend = refinement.analyze_quality_trend(&improvements);

    assert_eq!(trend, QualityTrend::Declining);
}

#[test]
fn test_quality_trend_stable() {
    let refinement = IterativeRefinement::new(5, 200000);

    let improvements = vec![0.15, 0.16, 0.15, 0.15, 0.16];
    let trend = refinement.analyze_quality_trend(&improvements);

    assert_eq!(trend, QualityTrend::Stable);
}

#[test]
fn test_max_iterations_limit() {
    let mut refinement = IterativeRefinement::new(2, 200000);

    let initial_prompt = "Analyze the code";
    let result = refinement.refine_prompt(initial_prompt).unwrap();

    assert_eq!(result.iterations.len(), 2);
}
