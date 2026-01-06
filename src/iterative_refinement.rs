use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterationDecision {
    Continue,
    Accept,
    Reject,
    Merge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTrend {
    Improving,
    Stable,
    Declining,
    Oscillating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationState {
    pub iteration_number: usize,
    pub prompt: String,
    pub output: String,
    pub quality_score: f64,
    pub criteria_scores: HashMap<String, f64>,
    pub timestamp: String,
    pub token_cost: usize,
    pub improvement_from_previous: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct IterationLimit {
    pub max_iterations: usize,
    pub min_quality_threshold: f64,
    pub improvement_threshold: f64,
    pub cost_threshold: usize,
    pub time_limit_minutes: usize,
}

impl Default for IterationLimit {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            min_quality_threshold: 0.75,
            improvement_threshold: 0.10,
            cost_threshold: 10_000,
            time_limit_minutes: 0,
        }
    }
}

pub struct IterationAnalyzer {
    limits: IterationLimit,
}

impl IterationAnalyzer {
    pub fn new(limits: IterationLimit) -> Self {
        Self { limits }
    }

    pub fn analyze_iterations(&self, iterations: &[IterationState]) -> AnalysisResult {
        if iterations.is_empty() {
            return AnalysisResult {
                can_continue: false,
                reason: "No iterations to analyze".to_string(),
                recommendation: "REJECT".to_string(),
                quality_trend: QualityTrend::Stable,
                current_score: 0.0,
                best_score: 0.0,
                best_iteration: 0,
                improvement_potential: 0.0,
                convergence_iteration: 0,
            };
        }

        let latest = &iterations[iterations.len() - 1];
        let total_iterations = iterations.len();

        let max_reached = total_iterations >= self.limits.max_iterations;
        let quality_met = latest.quality_score >= self.limits.min_quality_threshold;
        let cost_exceeded =
            iterations.iter().map(|i| i.token_cost).sum::<usize>() >= self.limits.cost_threshold;

        let quality_trend = self.calculate_quality_trend(iterations);
        let convergence = self.check_convergence(iterations, self.limits.improvement_threshold);

        let can_continue = !max_reached
            && !quality_met
            && !cost_exceeded
            && (quality_trend == QualityTrend::Improving || quality_trend == QualityTrend::Stable)
            && convergence == 0;

        let best = iterations.iter().max_by(|a, b| {
            a.quality_score
                .partial_cmp(&b.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_score = best.map(|b| b.quality_score).unwrap_or(0.0);
        let best_iteration = best.map(|b| b.iteration_number).unwrap_or(0);
        let improvement_potential = self.estimate_improvement_potential(iterations);

        let recommendation = self.make_recommendation(
            latest,
            quality_trend,
            convergence,
            max_reached,
            quality_met,
            cost_exceeded,
        );

        AnalysisResult {
            can_continue,
            reason: "".to_string(),
            recommendation,
            quality_trend,
            current_score: latest.quality_score,
            best_score,
            best_iteration,
            improvement_potential,
            convergence_iteration: convergence,
        }
    }

    fn calculate_quality_trend(&self, iterations: &[IterationState]) -> QualityTrend {
        if iterations.len() < 2 {
            return QualityTrend::Stable;
        }

        let mut improvements = Vec::new();
        for i in 1..iterations.len() {
            let improvement = iterations[i].quality_score - iterations[i - 1].quality_score;
            improvements.push(improvement);
        }

        let positive_count = improvements.iter().filter(|x| **x > 0.01).count();
        let negative_count = improvements.iter().filter(|x| **x < -0.01).count();

        if positive_count > improvements.len() * 3 / 5 {
            QualityTrend::Improving
        } else if negative_count > improvements.len() * 3 / 5 {
            QualityTrend::Declining
        } else if positive_count > 0 && negative_count > 0 {
            QualityTrend::Oscillating
        } else {
            QualityTrend::Stable
        }
    }

    fn check_convergence(&self, iterations: &[IterationState], threshold: f64) -> usize {
        if iterations.len() < 2 {
            return 0;
        }

        let window_size = std::cmp::min(3, iterations.len());
        let recent_iterations = &iterations[iterations.len() - window_size..];

        for i in 1..recent_iterations.len() {
            let improvement =
                recent_iterations[i].quality_score - recent_iterations[i - 1].quality_score;
            if improvement.abs() > threshold {
                return 0;
            }
        }

        recent_iterations[0].iteration_number
    }

    fn estimate_improvement_potential(&self, iterations: &[IterationState]) -> f64 {
        if iterations.is_empty() {
            return 0.0;
        }

        let trend = self.calculate_quality_trend(iterations);

        match trend {
            QualityTrend::Improving => {
                if iterations.len() >= 2 {
                    let latest = &iterations[iterations.len() - 1];
                    let last_improvement =
                        latest.quality_score - iterations[iterations.len() - 2].quality_score;
                    (last_improvement * 1.5).min(0.2)
                } else {
                    0.1
                }
            }
            QualityTrend::Stable => 0.02,
            QualityTrend::Declining => 0.0,
            QualityTrend::Oscillating => 0.05,
        }
    }

    fn make_recommendation(
        &self,
        latest: &IterationState,
        trend: QualityTrend,
        convergence: usize,
        max_reached: bool,
        quality_met: bool,
        cost_exceeded: bool,
    ) -> String {
        if quality_met {
            return "ACCEPT - Quality threshold met".to_string();
        }

        if cost_exceeded {
            return "REJECT - Cost limit exceeded".to_string();
        }

        if max_reached {
            if convergence > 0 {
                return format!("ACCEPT - Converged at iteration {}", convergence);
            } else if latest.quality_score >= 0.70 {
                return "ACCEPT - Max iterations reached with acceptable quality".to_string();
            } else {
                return "REJECT - Max iterations reached, quality too low".to_string();
            }
        }

        match trend {
            QualityTrend::Declining => "ACCEPT - Quality declining, use best iteration".to_string(),
            QualityTrend::Oscillating => {
                "ACCEPT - Quality oscillating, use best iteration".to_string()
            }
            QualityTrend::Improving => "CONTINUE - Quality improving".to_string(),
            QualityTrend::Stable => {
                if convergence > 0 {
                    "ACCEPT - Quality converged".to_string()
                } else {
                    "CONTINUE - Default action".to_string()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub can_continue: bool,
    pub reason: String,
    pub recommendation: String,
    pub quality_trend: QualityTrend,
    pub current_score: f64,
    pub best_score: f64,
    pub best_iteration: usize,
    pub improvement_potential: f64,
    pub convergence_iteration: usize,
}

#[derive(Debug, Clone)]
pub struct RefinementStrategy {
    pub strategy_type: String,
    pub focus_area: String,
    pub action: String,
    pub severity: usize,
}

pub struct RefinementGenerator;

impl RefinementGenerator {
    pub fn generate_refinement(
        iterations: &[IterationState],
        analysis: &AnalysisResult,
    ) -> RefinementStrategy {
        if iterations.is_empty() {
            return RefinementStrategy {
                strategy_type: "incremental".to_string(),
                focus_area: "all".to_string(),
                action: "expand".to_string(),
                severity: 3,
            };
        }

        let latest = &iterations[iterations.len() - 1];

        let weakest = latest
            .criteria_scores
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((weakest_criterion, weakest_score)) = weakest {
            match (weakest_criterion.as_str(), *weakest_score) {
                ("completeness", score) if score < 0.5 => RefinementStrategy {
                    strategy_type: "targeted".to_string(),
                    focus_area: "completeness".to_string(),
                    action: "expand".to_string(),
                    severity: 4,
                },
                ("accuracy", score) if score < 0.6 => RefinementStrategy {
                    strategy_type: "targeted".to_string(),
                    focus_area: "accuracy".to_string(),
                    action: "verify".to_string(),
                    severity: 3,
                },
                ("clarity", score) if score < 0.6 => RefinementStrategy {
                    strategy_type: "targeted".to_string(),
                    focus_area: "clarity".to_string(),
                    action: "clarify".to_string(),
                    severity: 2,
                },
                ("relevance", score) if score < 0.6 => RefinementStrategy {
                    strategy_type: "targeted".to_string(),
                    focus_area: "relevance".to_string(),
                    action: "restructure".to_string(),
                    severity: 3,
                },
                ("token_efficiency", score) if score < 0.7 => RefinementStrategy {
                    strategy_type: "targeted".to_string(),
                    focus_area: "token_efficiency".to_string(),
                    action: "condense".to_string(),
                    severity: 2,
                },
                _ => {
                    let improvement = analysis.improvement_potential;

                    if improvement > 0.10 {
                        RefinementStrategy {
                            strategy_type: "complete_rewrite".to_string(),
                            focus_area: "all".to_string(),
                            action: "restructure".to_string(),
                            severity: 3,
                        }
                    } else {
                        RefinementStrategy {
                            strategy_type: "incremental".to_string(),
                            focus_area: "all".to_string(),
                            action: "clarify".to_string(),
                            severity: 2,
                        }
                    }
                }
            }
        } else {
            RefinementStrategy {
                strategy_type: "incremental".to_string(),
                focus_area: "all".to_string(),
                action: "clarify".to_string(),
                severity: 2,
            }
        }
    }
}

pub struct IterativeRefinement {
    analyzer: IterationAnalyzer,
}

impl IterativeRefinement {
    pub fn new(limits: IterationLimit) -> Self {
        Self {
            analyzer: IterationAnalyzer::new(limits),
        }
    }

    pub fn refine_iteratively(
        &self,
        initial_prompt: &str,
        task_requirements: &str,
        limits: Option<IterationLimit>,
    ) -> RefinementResult {
        let effective_limits = limits.unwrap_or_default();

        let mut iterations: Vec<IterationState> = Vec::new();

        let (output, criteria_scores, quality_score) =
            self.generate_output(initial_prompt, task_requirements);

        iterations.push(IterationState {
            iteration_number: 1,
            prompt: initial_prompt.to_string(),
            output,
            quality_score,
            criteria_scores,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            token_cost: initial_prompt.len() / 4,
            improvement_from_previous: 0.0,
        });

        let mut iteration_num = 2;
        let analysis = self.analyzer.analyze_iterations(&iterations);

        while iteration_num <= effective_limits.max_iterations && analysis.can_continue {
            let strategy = RefinementGenerator::generate_refinement(&iterations, &analysis);
            let refined_prompt =
                self.apply_refinement(&iterations.last().unwrap().prompt, &strategy);

            let (output, criteria_scores, quality_score) =
                self.generate_output(&refined_prompt, task_requirements);
            let improvement = quality_score - iterations.last().unwrap().quality_score;

            iterations.push(IterationState {
                iteration_number: iteration_num,
                prompt: refined_prompt.clone(),
                output,
                quality_score,
                criteria_scores,
                timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                token_cost: refined_prompt.len() / 4,
                improvement_from_previous: improvement,
            });

            let _analysis = self.analyzer.analyze_iterations(&iterations);
            iteration_num += 1;
        }

        let best = iterations.iter().max_by(|a, b| {
            a.quality_score
                .partial_cmp(&b.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_token_cost: usize = iterations.iter().map(|i| i.token_cost).sum();
        let quality_trend = self.analyzer.calculate_quality_trend(&iterations);
        let convergence_iteration = self
            .analyzer
            .check_convergence(&iterations, effective_limits.improvement_threshold);

        let final_iteration = iterations.last().unwrap().clone();

        let final_analysis = self.analyzer.analyze_iterations(&iterations);
        let recommendation = if iteration_num > effective_limits.max_iterations {
            final_analysis.recommendation.clone()
        } else {
            format!(
                "Accept iteration {} - {:?} trend",
                final_iteration.iteration_number, quality_trend
            )
        };

        RefinementResult {
            final_iteration,
            total_iterations: iterations.len(),
            total_token_cost,
            quality_trend,
            decision: IterationDecision::Accept,
            best_iteration: best.unwrap().clone(),
            convergence_iteration,
            recommendation,
        }
    }

    fn generate_output(
        &self,
        prompt: &str,
        _task_requirements: &str,
    ) -> (String, HashMap<String, f64>, f64) {
        let prompt_lower = prompt.to_lowercase();

        let (output, completeness, clarity) = if prompt_lower.contains("expand") {
            (
                "Detailed analysis with extensive details and multiple sections covering all aspects of task thoroughly.".to_string(),
                0.9,
                0.8,
            )
        } else if prompt_lower.contains("clarify") {
            (
                "Clear and structured analysis with explicit numbering and specific findings."
                    .to_string(),
                0.8,
                0.95,
            )
        } else if prompt_lower.contains("condense") {
            (
                "Analysis complete. Key findings: security vulnerabilities identified, performance issues noted.".to_string(),
                0.6,
                0.9,
            )
        } else {
            (
                "Analysis completed. Found some issues and provided recommendations.".to_string(),
                0.7,
                0.75,
            )
        };

        let mut criteria_scores = HashMap::new();
        criteria_scores.insert("completeness".to_string(), completeness);
        criteria_scores.insert("accuracy".to_string(), 0.85);
        criteria_scores.insert("clarity".to_string(), clarity);
        criteria_scores.insert("relevance".to_string(), 0.9);
        criteria_scores.insert(
            "token_efficiency".to_string(),
            (100.0 / output.len() as f64).min(1.0),
        );
        criteria_scores.insert("novelty".to_string(), 0.7);

        let quality_score = criteria_scores.values().sum::<f64>() / criteria_scores.len() as f64;

        (output, criteria_scores, quality_score)
    }

    fn apply_refinement(&self, prompt: &str, strategy: &RefinementStrategy) -> String {
        let mut refined = prompt.to_string();

        match strategy.action.as_str() {
            "expand" => {
                refined.push_str("\n\nAdditional Instructions:\n- Provide comprehensive analysis\n- Include all relevant details\n- Cover multiple aspects thoroughly");
            }
            "clarify" => {
                refined.push_str("\n\nClarity Instructions:\n- Use clear, precise language\n- Avoid ambiguity\n- Use structured format with numbered sections");
            }
            "condense" => {
                refined.push_str("\n\nConciseness Instructions:\n- Be brief and to the point\n- Remove redundancy\n- Focus on key information only");
            }
            "restructure" => {
                refined.push_str("\n\nStructure Instructions:\n- Use hierarchical organization\n- Group related information\n- Use clear headings and subheadings");
            }
            "verify" => {
                refined.push_str("\n\nVerification Instructions:\n- Double-check all claims\n- Provide evidence for conclusions\n- Verify accuracy of statements");
            }
            _ => {}
        }

        refined
    }
}

#[derive(Debug, Clone)]
pub struct RefinementResult {
    pub final_iteration: IterationState,
    pub total_iterations: usize,
    pub total_token_cost: usize,
    pub quality_trend: QualityTrend,
    pub decision: IterationDecision,
    pub best_iteration: IterationState,
    pub convergence_iteration: usize,
    pub recommendation: String,
}
