use crate::types::{Plan, PlanStep, StepStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodifiedReasoningConfig {
    pub urgency_source: UrgencySource,
    pub contribution_weight: f64,
    pub urgency_weight: f64,
    pub impact_weight: f64,
    pub default_step_tokens: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UrgencySource {
    Position,
    Deadline,
    Custom,
}

impl Default for CodifiedReasoningConfig {
    fn default() -> Self {
        Self {
            urgency_source: UrgencySource::Position,
            contribution_weight: 0.7,
            urgency_weight: 0.3,
            impact_weight: 0.3,
            default_step_tokens: 500,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodifiedReasoning {
    config: CodifiedReasoningConfig,
    role_impact_map: HashMap<String, f64>,
}

impl CodifiedReasoning {
    pub fn new() -> Self {
        let mut role_impact_map = HashMap::new();
        role_impact_map.insert("extractor".to_string(), 0.8);
        role_impact_map.insert("analyzer".to_string(), 0.9);
        role_impact_map.insert("writer".to_string(), 0.7);
        role_impact_map.insert("reviewer".to_string(), 0.85);
        role_impact_map.insert("synthesizer".to_string(), 0.75);
        role_impact_map.insert("general".to_string(), 0.5);

        Self {
            config: CodifiedReasoningConfig::default(),
            role_impact_map,
        }
    }

    pub fn with_config(config: CodifiedReasoningConfig) -> Self {
        let mut role_impact_map = HashMap::new();
        role_impact_map.insert("extractor".to_string(), 0.8);
        role_impact_map.insert("analyzer".to_string(), 0.9);
        role_impact_map.insert("writer".to_string(), 0.7);
        role_impact_map.insert("reviewer".to_string(), 0.85);
        role_impact_map.insert("synthesizer".to_string(), 0.75);
        role_impact_map.insert("general".to_string(), 0.5);

        Self {
            config,
            role_impact_map,
        }
    }

    pub fn codify_prompt(&self, free_form_plan: &str, target_role: &str) -> Plan {
        let steps = self.parse_into_steps(free_form_plan);
        let total_steps = steps.len() as u32;
        let mut total_tokens = 0;

        let role_impact = self
            .role_impact_map
            .get(&target_role.to_lowercase())
            .copied()
            .unwrap_or(0.5);

        let processed_steps: Vec<PlanStep> = steps
            .into_iter()
            .enumerate()
            .map(|(idx, (action, target, outcome))| {
                let step_number = (idx + 1) as u32;
                let contribution_score =
                    self.calculate_contribution(action.clone(), target.clone());
                let urgency = self.calculate_urgency(step_number, total_steps);
                let impact_score =
                    self.calculate_impact_score(action.clone(), target.clone(), role_impact);
                let priority = self.calculate_priority(contribution_score, urgency);

                let expected_tokens = self.estimate_step_tokens(&action, &target);

                total_tokens += expected_tokens;

                PlanStep {
                    step_number,
                    action,
                    target,
                    expected_outcome: outcome.unwrap_or_default(),
                    expected_tokens,
                    contribution_score,
                    impact_score,
                    priority,
                    status: if idx == 0 {
                        StepStatus::Active
                    } else {
                        StepStatus::Pending
                    },
                }
            })
            .collect();

        Plan {
            steps: processed_steps,
            total_expected_tokens: total_tokens,
            status: "active".to_string(),
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }

    fn parse_into_steps(&self, plan: &str) -> Vec<(String, String, Option<String>)> {
        let mut steps = Vec::new();
        let lines: Vec<&str> = plan.lines().collect();

        let mut current_action = String::new();
        let mut current_target = String::new();
        let mut current_outcome = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with("step") {
                if !current_action.is_empty() {
                    steps.push((
                        current_action.clone(),
                        current_target.clone(),
                        current_outcome.clone(),
                    ));
                }

                let cleaned = trimmed
                    .trim_start_matches('-')
                    .trim_start_matches('*')
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ':')
                    .trim()
                    .to_string();

                if let Some((action, rest)) = cleaned.split_once(" to ") {
                    current_action = action.trim().to_string();
                    current_target = rest.trim().to_string();
                    current_outcome = None;
                } else if let Some((action, rest)) = cleaned.split_once(" for ") {
                    current_action = action.trim().to_string();
                    current_target = rest.trim().to_string();
                    current_outcome = None;
                } else {
                    current_action = cleaned.clone();
                    current_target = "general".to_string();
                    current_outcome = None;
                }

                if let Some(idx) = cleaned.find("expecting") {
                    let outcome_part = &cleaned[idx + "expecting".len()..];
                    current_outcome = Some(outcome_part.trim().to_string());
                }
            } else if let Some((key, value)) = trimmed.split_once(':') {
                let key_lower = key.trim().to_lowercase();
                if key_lower.contains("action") {
                    current_action = value.trim().to_string();
                } else if key_lower.contains("target") {
                    current_target = value.trim().to_string();
                } else if key_lower.contains("outcome") || key_lower.contains("expect") {
                    current_outcome = Some(value.trim().to_string());
                }
            }
        }

        if !current_action.is_empty() {
            steps.push((current_action, current_target, current_outcome));
        }

        if steps.is_empty() {
            steps.push((
                "execute_task".to_string(),
                "main_objective".to_string(),
                Some("task_completed".to_string()),
            ));
        }

        steps
    }

    fn calculate_contribution(&self, action: String, target: String) -> f64 {
        let action_lower = action.to_lowercase();
        let target_lower = target.to_lowercase();

        let high_contribution_actions = [
            "implement",
            "create",
            "design",
            "analyze",
            "fix",
            "optimize",
            "secure",
        ];
        let medium_contribution_actions =
            ["write", "update", "modify", "test", "review", "document"];
        let low_contribution_actions = ["list", "check", "read", "print", "log", "echo"];

        let action_score: f64 = if high_contribution_actions
            .iter()
            .any(|a| action_lower.contains(a))
        {
            0.9
        } else if medium_contribution_actions
            .iter()
            .any(|a| action_lower.contains(a))
        {
            0.6
        } else if low_contribution_actions
            .iter()
            .any(|a| action_lower.contains(a))
        {
            0.3
        } else {
            0.5
        };

        let target_score: f64 = if target_lower.contains("core")
            || target_lower.contains("main")
            || target_lower.contains("critical")
        {
            1.0
        } else if target_lower.contains("test")
            || target_lower.contains("doc")
            || target_lower.contains("example")
        {
            0.7
        } else {
            0.8
        };

        (action_score * 0.6 + target_score * 0.4).min(1.0_f64)
    }

    fn calculate_urgency(&self, step_number: u32, total_steps: u32) -> f64 {
        match self.config.urgency_source {
            UrgencySource::Position => {
                if total_steps == 0 {
                    1.0
                } else {
                    1.0 - (step_number as f64 / total_steps as f64)
                }
            }
            UrgencySource::Deadline => 0.8,
            UrgencySource::Custom => 0.5,
        }
    }

    fn calculate_impact_score(&self, action: String, _target: String, role_impact: f64) -> f64 {
        let action_lower = action.to_lowercase();

        let has_impact_keywords = action_lower.contains("create")
            || action_lower.contains("implement")
            || action_lower.contains("fix")
            || action_lower.contains("optimize")
            || action_lower.contains("analyze");

        let base_impact = if has_impact_keywords {
            role_impact * 1.1
        } else {
            role_impact * 0.9
        };

        base_impact.min(1.0_f64)
    }

    fn calculate_priority(&self, contribution: f64, urgency: f64) -> f64 {
        (contribution * self.config.contribution_weight) + (urgency * self.config.urgency_weight)
    }

    fn estimate_step_tokens(&self, action: &str, target: &str) -> u32 {
        let action_lower = action.to_lowercase();

        let base_tokens = if action_lower.contains("implement")
            || action_lower.contains("create")
            || action_lower.contains("design")
        {
            800
        } else if action_lower.contains("analyze")
            || action_lower.contains("review")
            || action_lower.contains("test")
        {
            500
        } else if action_lower.contains("write")
            || action_lower.contains("update")
            || action_lower.contains("modify")
        {
            400
        } else {
            self.config.default_step_tokens
        };

        let target_length_factor = (target.len() as f64 / 50.0).min(0.5);

        (base_tokens as f64 * (1.0 + target_length_factor)) as u32
    }

    pub fn summarize_old_plans(&self, plans: &[Plan], max_summarized: usize) -> Vec<String> {
        let mut summaries = Vec::new();

        for plan in plans.iter().take(max_summarized) {
            let completed_steps: Vec<_> = plan
                .steps
                .iter()
                .filter(|s| s.status == StepStatus::Complete)
                .collect();

            if !completed_steps.is_empty() {
                let summary = format!(
                    "Plan completed {} steps: {} → {}",
                    completed_steps.len(),
                    completed_steps
                        .first()
                        .map(|s| s.action.clone())
                        .unwrap_or_else(|| "start".to_string()),
                    completed_steps
                        .last()
                        .map(|s| s.action.clone())
                        .unwrap_or_else(|| "end".to_string())
                );
                summaries.push(summary);
            }
        }

        summaries
    }

    pub fn link_impact_to_routing(&self, plan: &Plan) -> Vec<(u32, f64)> {
        plan.steps
            .iter()
            .map(|step| (step.step_number, step.impact_score))
            .collect()
    }
}

impl Default for CodifiedReasoning {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codified_reasoning_new() {
        let cr = CodifiedReasoning::new();
        assert_eq!(cr.config.urgency_source, UrgencySource::Position);
    }

    #[test]
    fn test_codify_prompt_basic() {
        let cr = CodifiedReasoning::new();
        let plan = cr.codify_prompt(
            "1. Read the main.rs file\n2. Analyze the code\n3. Fix bugs",
            "analyzer",
        );
        assert!(!plan.steps.is_empty());
        assert!(plan.total_expected_tokens > 0);
    }

    #[test]
    fn test_calculate_urgency() {
        let cr = CodifiedReasoning::new();
        assert!((cr.calculate_urgency(1, 5) - 0.8).abs() < 0.001);
        assert!((cr.calculate_urgency(3, 5) - 0.4).abs() < 0.001);
        assert!((cr.calculate_urgency(5, 5) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_priority() {
        let cr = CodifiedReasoning::new();
        let priority = cr.calculate_priority(0.8, 0.9);
        assert!((priority - 0.83).abs() < 0.01);
    }

    #[test]
    fn test_link_impact_to_routing() {
        let cr = CodifiedReasoning::new();
        let plan = cr.codify_prompt("Implement feature X", "analyzer");
        let impact_map = cr.link_impact_to_routing(&plan);
        assert!(!impact_map.is_empty());
        for (step, impact) in &impact_map {
            assert!(*impact >= 0.0 && *impact <= 1.0);
        }
    }

    #[test]
    fn test_summarize_old_plans() {
        let cr = CodifiedReasoning::new();
        let plan = cr.codify_prompt("Read file", "analyzer");
        let summaries = cr.summarize_old_plans(&[plan], 5);
        assert!(summaries.is_empty());
    }
}
