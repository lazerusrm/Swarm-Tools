use crate::config::QualityGateConfig;
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub score: f64,
    pub quality_level: QualityLevel,
    pub refinement_action: RefinementAction,
    pub criteria_scores: Vec<CriterionScore>,
    pub meets_threshold: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QualityLevel {
    Excellent,
    Good,
    Acceptable,
    Poor,
    Unacceptable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RefinementAction {
    None,
    Expand,
    Clarify,
    Focus,
    Rewrite,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionScore {
    pub name: String,
    pub score: f64,
    pub weight: f64,
    pub weighted_score: f64,
    pub max_score: f64,
}

impl From<f64> for QualityLevel {
    fn from(score: f64) -> Self {
        match score {
            s if s >= 90.0 => QualityLevel::Excellent,
            s if s >= 80.0 => QualityLevel::Good,
            s if s >= 70.0 => QualityLevel::Acceptable,
            s if s >= 60.0 => QualityLevel::Poor,
            _ => QualityLevel::Unacceptable,
        }
    }
}

impl From<QualityLevel> for RefinementAction {
    fn from(level: QualityLevel) -> Self {
        match level {
            QualityLevel::Excellent => RefinementAction::None,
            QualityLevel::Good => RefinementAction::Expand,
            QualityLevel::Acceptable => RefinementAction::Clarify,
            QualityLevel::Poor => RefinementAction::Focus,
            QualityLevel::Unacceptable => RefinementAction::Rewrite,
        }
    }
}

impl std::fmt::Display for QualityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityLevel::Excellent => write!(f, "EXCELLENT"),
            QualityLevel::Good => write!(f, "GOOD"),
            QualityLevel::Acceptable => write!(f, "ACCEPTABLE"),
            QualityLevel::Poor => write!(f, "POOR"),
            QualityLevel::Unacceptable => write!(f, "UNACCEPTABLE"),
        }
    }
}

impl std::fmt::Display for RefinementAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefinementAction::None => write!(f, "none"),
            RefinementAction::Expand => write!(f, "expand"),
            RefinementAction::Clarify => write!(f, "clarify"),
            RefinementAction::Focus => write!(f, "focus"),
            RefinementAction::Rewrite => write!(f, "rewrite"),
            RefinementAction::Review => write!(f, "review"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualityGate {
    config: QualityGateConfig,
}

impl QualityGate {
    pub fn new() -> Self {
        Self::with_config(QualityGateConfig::default())
    }

    pub fn with_config(config: QualityGateConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(
        &self,
        output: &str,
        impact_score: f64,
        contribution_score: f64,
    ) -> QualityGateResult {
        let criteria = self.evaluate_criteria(output, impact_score, contribution_score);
        let total_weighted: f64 = criteria.iter().map(|c| c.weighted_score).sum();
        let max_possible: f64 = criteria.iter().map(|c| c.max_score * c.weight).sum();

        let score = if max_possible > 0.0 {
            (total_weighted / max_possible) * 100.0
        } else {
            0.0
        };

        let quality_level = QualityLevel::from(score);
        let refinement_action = RefinementAction::from(quality_level.clone());
        let meets_threshold = score >= self.config.minimum_threshold;

        QualityGateResult {
            score: score.min(100.0),
            quality_level,
            refinement_action,
            criteria_scores: criteria,
            meets_threshold,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }

    fn evaluate_criteria(
        &self,
        output: &str,
        impact_score: f64,
        contribution_score: f64,
    ) -> Vec<CriterionScore> {
        let mut criteria = Vec::new();

        if self.config.impact_weight > 0.0 {
            let impact_raw = impact_score * 100.0;
            criteria.push(CriterionScore {
                name: "impact".to_string(),
                score: impact_raw,
                weight: self.config.impact_weight,
                weighted_score: impact_raw * self.config.impact_weight,
                max_score: 100.0,
            });
        }

        if self.config.contribution_weight > 0.0 {
            let contrib_raw = contribution_score * 100.0;
            criteria.push(CriterionScore {
                name: "contribution".to_string(),
                score: contrib_raw,
                weight: self.config.contribution_weight,
                weighted_score: contrib_raw * self.config.contribution_weight,
                max_score: 100.0,
            });
        }

        if self.config.completeness_weight > 0.0 {
            let completeness = self.evaluate_completeness(output);
            criteria.push(CriterionScore {
                name: "completeness".to_string(),
                score: completeness,
                weight: self.config.completeness_weight,
                weighted_score: completeness * self.config.completeness_weight,
                max_score: 100.0,
            });
        }

        if self.config.coherence_weight > 0.0 {
            let coherence = self.evaluate_coherence(output);
            criteria.push(CriterionScore {
                name: "coherence".to_string(),
                score: coherence,
                weight: self.config.coherence_weight,
                weighted_score: coherence * self.config.coherence_weight,
                max_score: 100.0,
            });
        }

        if self.config.todo_penalty_weight > 0.0 {
            let todo_penalty = self.evaluate_todo_penalty(output);
            criteria.push(CriterionScore {
                name: "todo_penalty".to_string(),
                score: todo_penalty,
                weight: self.config.todo_penalty_weight,
                weighted_score: todo_penalty * self.config.todo_penalty_weight,
                max_score: 100.0,
            });
        }

        criteria
    }

    fn evaluate_completeness(&self, output: &str) -> f64 {
        let lines: Vec<&str> = output.lines().collect();
        let line_count = lines.len();

        let base_score = match line_count {
            l if l >= 20 => 100.0,
            l if l >= 10 => 80.0,
            l if l >= 5 => 60.0,
            l if l >= 1 => 40.0,
            _ => 0.0,
        };

        let has_sections = output.contains("##") || output.contains("###");
        let has_list = output.contains("- ") || output.contains("* ") || output.contains("1.");

        let mut score: f64 = base_score;
        if has_sections {
            score += 10.0;
        }
        if has_list {
            score += 10.0;
        }

        score.min(100.0)
    }

    fn evaluate_coherence(&self, output: &str) -> f64 {
        let paragraphs: Vec<&str> = output.split("\n\n").collect();
        if paragraphs.len() <= 1 {
            return 70.0;
        }

        let coherence_indicators = [
            "however",
            "therefore",
            "furthermore",
            "additionally",
            "consequently",
        ];
        let text_lower = output.to_lowercase();

        let mut score: f64 = 60.0;
        for indicator in &coherence_indicators {
            if text_lower.contains(indicator) {
                score += 8.0;
            }
        }

        let sentences: Vec<&str> = output.split(|c| c == '.' || c == '!' || c == '?').collect();
        if sentences.len() > 3 {
            score += 5.0;
        }

        score.min(100.0)
    }

    fn evaluate_todo_penalty(&self, output: &str) -> f64 {
        let text_lower = output.to_lowercase();
        let todo_count = text_lower.matches("todo").count() + text_lower.matches("fixme").count();
        let hack_count = text_lower.matches("hack").count();

        let penalty = (todo_count as f64 * 15.0 + hack_count as f64 * 20.0).min(100.0);
        -penalty
    }

    pub fn should_continue_refinement(&self, result: &QualityGateResult) -> bool {
        !result.meets_threshold
            && result.quality_level != QualityLevel::Unacceptable
            && result.refinement_action != RefinementAction::Rewrite
    }

    pub fn decide_next_action(&self, result: &QualityGateResult) -> RefinementAction {
        result.refinement_action.clone()
    }
}

impl Default for QualityGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_excellent() {
        let gate = QualityGate::new();
        let result = gate.evaluate("This is excellent content with multiple sections\n## Introduction\n### Details\n- Point 1\n- Point 2\n- Point 3\nTherefore we conclude...", 0.95, 0.90);

        assert!(result.score >= 70.0);
        assert!(result.meets_threshold);
    }

    #[test]
    fn test_quality_gate_poor_with_todos() {
        let gate = QualityGate::new();
        let result = gate.evaluate("TODO: fix this later\nTODO: also fix this", 0.3, 0.2);

        assert!(result.score < 70.0);
    }

    #[test]
    fn test_quality_gate_with_impact_and_contribution() {
        let gate = QualityGate::new();
        let result = gate.evaluate("Quality content", 0.85, 0.75);

        let has_impact = result.criteria_scores.iter().any(|c| c.name == "impact");
        let has_contribution = result
            .criteria_scores
            .iter()
            .any(|c| c.name == "contribution");
        assert!(has_impact);
        assert!(has_contribution);
    }

    #[test]
    fn test_should_continue_refinement() {
        let gate = QualityGate::new();
        let poor = gate.evaluate(
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10",
            0.7,
            0.7,
        );
        let good = gate.evaluate("This is excellent content with multiple sections\n## Introduction\n### Details\n- Point 1\n- Point 2\n- Point 3\nTherefore we conclude...", 0.95, 0.95);

        assert!(gate.should_continue_refinement(&poor));
        assert!(!gate.should_continue_refinement(&good));
    }

    #[test]
    fn test_quality_level_from_score() {
        assert_eq!(QualityLevel::from(95.0), QualityLevel::Excellent);
        assert_eq!(QualityLevel::from(85.0), QualityLevel::Good);
        assert_eq!(QualityLevel::from(75.0), QualityLevel::Acceptable);
        assert_eq!(QualityLevel::from(65.0), QualityLevel::Poor);
        assert_eq!(QualityLevel::from(50.0), QualityLevel::Unacceptable);
    }
}
