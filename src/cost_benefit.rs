use crate::config::CostBenefitWeightsConfig;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Weights {
    tokens: f64,
    time: f64,
    accuracy: f64,
    completion: f64,
    information: f64,
    strategy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecisionRecord {
    action: serde_json::Value,
    estimated_cost: f64,
    estimated_benefit: f64,
    ratio: f64,
    decision: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActualRecord {
    action_id: String,
    actual_cost: f64,
    actual_benefit: f64,
    timestamp: String,
}

pub struct CostBenefitAnalyzer {
    weights: Weights,
    token_scale: f64,
    time_scale: f64,
    history: History,
}

#[derive(Debug, Clone, Default)]
struct History {
    estimates: Vec<DecisionRecord>,
    actuals: Vec<ActualRecord>,
}

impl CostBenefitAnalyzer {
    pub fn new() -> Self {
        Self::with_config(CostBenefitWeightsConfig::default())
    }

    pub fn with_config(config: CostBenefitWeightsConfig) -> Self {
        Self {
            weights: Weights {
                tokens: config.tokens_weight,
                time: config.time_weight,
                accuracy: config.quality_weight,
                completion: config.contribution_weight,
                information: 1.5,
                strategy: 1.0,
            },
            token_scale: 1.0 / 5000.0,
            time_scale: 1.0 / 60.0,
            history: History::default(),
        }
    }

    pub fn estimate_cost(&self, action: &serde_json::Value) -> Result<f64> {
        let tokens_required = action
            .get("tokens_required")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000) as f64;

        let time_required = action
            .get("time_required")
            .and_then(|v| v.as_u64())
            .unwrap_or(60) as f64;

        let accuracy_impact = action
            .get("accuracy_impact")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let normalized_tokens = tokens_required * self.token_scale;
        let normalized_time = time_required * self.time_scale;
        let normalized_accuracy = accuracy_impact;

        let cost = (normalized_tokens * self.weights.tokens)
            + (normalized_time * self.weights.time)
            + (normalized_accuracy * self.weights.accuracy);

        Ok(cost)
    }

    pub fn estimate_benefit(&self, action: &serde_json::Value) -> Result<f64> {
        let task_completion_value = action
            .get("task_completion_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);

        let new_information_value = action
            .get("new_information_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);

        let strategic_value = action
            .get("strategic_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let benefit = (task_completion_value * self.weights.completion)
            + (new_information_value * self.weights.information)
            + (strategic_value * self.weights.strategy);

        Ok(benefit)
    }

    pub fn make_decision(&mut self, action: serde_json::Value) -> Result<CostBenefitResult> {
        let cost = self.estimate_cost(&action)?;
        let benefit = self.estimate_benefit(&action)?;
        let ratio = if cost > 0.0 {
            benefit / cost
        } else {
            f64::INFINITY
        };

        let (decision_type, message) = if ratio > 1.0 {
            ("execute".to_string(), "Benefit exceeds cost".to_string())
        } else if ratio > 0.8 {
            (
                "adjust_scope".to_string(),
                "Benefit slightly lower than cost, adjusting scope".to_string(),
            )
        } else if ratio > 0.5 {
            (
                "request_assistance".to_string(),
                "Cost moderate, requesting assistance".to_string(),
            )
        } else {
            ("skip".to_string(), "Cost exceeds benefit".to_string())
        };

        let record = DecisionRecord {
            action: action.clone(),
            estimated_cost: cost,
            estimated_benefit: benefit,
            ratio,
            decision: decision_type.clone(),
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };

        self.history.estimates.push(record);

        Ok(CostBenefitResult {
            decision: decision_type,
            message,
            cost,
            benefit,
            ratio,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        })
    }

    pub fn record_actual(&mut self, action_id: String, actual_cost: f64, actual_benefit: f64) {
        let record = ActualRecord {
            action_id,
            actual_cost,
            actual_benefit,
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };
        self.history.actuals.push(record);
    }

    pub fn get_decision_stats(&self) -> DecisionStats {
        let total = self.history.estimates.len();

        if total == 0 {
            return DecisionStats {
                total_decisions: 0,
                by_type: HashMap::new(),
                execute_pct: 0.0,
                adjust_scope_pct: 0.0,
                request_assistance_pct: 0.0,
                skip_pct: 0.0,
            };
        }

        let mut by_type: HashMap<String, usize> = HashMap::new();
        for record in &self.history.estimates {
            let count = by_type.entry(record.decision.clone()).or_insert(0);
            *count += 1;
        }

        let execute_pct = (*by_type.get("execute").unwrap_or(&0) as f64 / total as f64) * 100.0;
        let adjust_scope_pct =
            (*by_type.get("adjust_scope").unwrap_or(&0) as f64 / total as f64) * 100.0;
        let request_assistance_pct =
            (*by_type.get("request_assistance").unwrap_or(&0) as f64 / total as f64) * 100.0;
        let skip_pct = (*by_type.get("skip").unwrap_or(&0) as f64 / total as f64) * 100.0;

        DecisionStats {
            total_decisions: total,
            by_type,
            execute_pct,
            adjust_scope_pct,
            request_assistance_pct,
            skip_pct,
        }
    }

    #[allow(clippy::needless_return)]
    pub fn adapt_weights(&mut self) {
        if self.history.estimates.len() < 10 {
            return;
        }
    }
}

impl Default for CostBenefitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
