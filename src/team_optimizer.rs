pub use crate::types::*;
use regex::Regex;
use std::collections::HashMap;

pub struct TaskAnalyzer {
    complexity_indicators: HashMap<TaskComplexity, Vec<String>>,
}

impl TaskAnalyzer {
    pub fn new() -> Self {
        let mut complexity_indicators = HashMap::new();

        complexity_indicators.insert(
            TaskComplexity::Simple,
            vec![
                "single task".to_string(),
                "one file".to_string(),
                "simple".to_string(),
                "basic".to_string(),
                "quick".to_string(),
                "straightforward".to_string(),
            ],
        );

        complexity_indicators.insert(
            TaskComplexity::Moderate,
            vec![
                "multiple files".to_string(),
                "several tasks".to_string(),
                "analyze and".to_string(),
                "test and".to_string(),
                "review and".to_string(),
            ],
        );

        complexity_indicators.insert(
            TaskComplexity::Complex,
            vec![
                "multiple objectives".to_string(),
                "several components".to_string(),
                "integrated system".to_string(),
                "comprehensive".to_string(),
                "thorough analysis".to_string(),
            ],
        );

        complexity_indicators.insert(
            TaskComplexity::VeryComplex,
            vec![
                "large codebase".to_string(),
                "multiple systems".to_string(),
                "architecture review".to_string(),
                "performance optimization".to_string(),
                "security audit".to_string(),
            ],
        );

        Self {
            complexity_indicators,
        }
    }

    pub fn analyze_task(&self, task_description: &str) -> Result<TaskAnalysis> {
        let complexity = self.determine_complexity(task_description);
        let task_type = self.determine_task_type(task_description);
        let subtasks = self.extract_subtasks(task_description);
        let estimated_effort = self.estimate_effort(complexity, &subtasks);
        let required_roles = self.determine_roles(task_description, &task_type);
        let priority = self.determine_priority(task_description);

        Ok(TaskAnalysis {
            complexity,
            task_type,
            subtasks,
            estimated_effort,
            required_roles,
            priority,
        })
    }

    fn determine_complexity(&self, task_description: &str) -> TaskComplexity {
        let text_lower = task_description.to_lowercase();

        let mut scores: HashMap<TaskComplexity, i32> = HashMap::new();

        for (complexity, indicators) in &self.complexity_indicators {
            let mut score = 0;
            for indicator in indicators {
                if text_lower.contains(indicator) {
                    score += 1;
                }
            }
            scores.insert(*complexity, score);
        }

        if task_description.split_whitespace().count() > 100 {
            *scores.get_mut(&TaskComplexity::VeryComplex).unwrap() += 2;
        }

        let task_verbs_re =
            Regex::new(r"\b(analyze|review|test|write|implement|optimize|refactor)\b").unwrap();
        let task_verbs = task_verbs_re.find_iter(task_description).count();

        if task_verbs > 3 {
            *scores.get_mut(&TaskComplexity::Complex).unwrap() += 2;
        } else if task_verbs > 1 {
            *scores.get_mut(&TaskComplexity::Moderate).unwrap() += 1;
        }

        let best_complexity = scores
            .iter()
            .max_by_key(|&(_, score)| score)
            .map(|(&complexity, _)| complexity)
            .unwrap_or(TaskComplexity::Simple);

        best_complexity
    }

    fn determine_task_type(&self, task_description: &str) -> String {
        let task_patterns: HashMap<&str, Vec<&str>> = vec![
            (
                "code_review",
                vec!["review", "code review", "pr review", "pull request"],
            ),
            (
                "testing",
                vec!["test", "testing", "test suite", "unit tests"],
            ),
            (
                "documentation",
                vec!["document", "documentation", "docs", "readme"],
            ),
            (
                "analysis",
                vec!["analyze", "analysis", "investigate", "examine"],
            ),
            (
                "implementation",
                vec!["implement", "write", "create", "build"],
            ),
            (
                "optimization",
                vec!["optimize", "optimize", "refactor", "improve performance"],
            ),
            (
                "security",
                vec!["security", "audit", "vulnerability", "penetration test"],
            ),
        ]
        .into_iter()
        .collect();

        let text_lower = task_description.to_lowercase();
        let mut scores: HashMap<&str, i32> = HashMap::new();

        for (task_type, patterns) in &task_patterns {
            let mut score = 0;
            for pattern in patterns {
                if text_lower.contains(pattern) {
                    score += 1;
                }
            }
            scores.insert(task_type, score);
        }

        scores
            .iter()
            .max_by_key(|&(_, score)| score)
            .map(|(&task_type, _)| task_type.to_string())
            .unwrap_or_else(|| "analysis".to_string())
    }

    fn extract_subtasks(&self, task_description: &str) -> Vec<String> {
        let numbered_re = Regex::new(r"\d+\.\s+([^.]+\.?)").unwrap();
        let numbered_items: Vec<&str> = numbered_re
            .find_iter(task_description)
            .map(|m| m.as_str())
            .collect();

        if !numbered_items.is_empty() {
            return numbered_items
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
        }

        let task_verbs_re = Regex::new(
            r"(?:analyze|review|test|write|implement|optimize|refactor|document)\s+([^.]+\.?)",
        )
        .unwrap();
        let task_verbs: Vec<&str> = task_verbs_re
            .find_iter(task_description)
            .map(|m| m.as_str())
            .collect();

        if !task_verbs.is_empty() {
            return task_verbs
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
        }

        let parts: Vec<&str> = Regex::new(r"\b(and|also|additionally|furthermore|moreover)\b")
            .unwrap()
            .split(task_description)
            .collect();

        if parts.len() > 1 {
            return parts
                .into_iter()
                .filter(|s| s.trim().len() > 10)
                .map(|s| s.trim().to_string())
                .collect();
        }

        vec![task_description.trim().to_string()]
    }

    fn estimate_effort(&self, complexity: TaskComplexity, subtasks: &[String]) -> f64 {
        let base_effort = match complexity {
            TaskComplexity::Simple => 0.5,
            TaskComplexity::Moderate => 2.0,
            TaskComplexity::Complex => 8.0,
            TaskComplexity::VeryComplex => 24.0,
        };

        let base = base_effort;
        let subtask_multiplier = 1.0 + (subtasks.len().saturating_sub(1) as f64 * 0.2);

        base * subtask_multiplier
    }

    fn determine_roles(&self, task_description: &str, task_type: &str) -> Vec<AgentRole> {
        let role_mappings: HashMap<&str, Vec<AgentRole>> = vec![
            (
                "code_review",
                vec![AgentRole::Analyzer, AgentRole::Reviewer],
            ),
            ("testing", vec![AgentRole::Tester, AgentRole::Analyzer]),
            (
                "documentation",
                vec![AgentRole::Documenter, AgentRole::Analyzer],
            ),
            ("analysis", vec![AgentRole::Analyzer, AgentRole::Documenter]),
            (
                "implementation",
                vec![AgentRole::Analyzer, AgentRole::Specialist],
            ),
            (
                "optimization",
                vec![AgentRole::Optimizer, AgentRole::Analyzer, AgentRole::Tester],
            ),
            ("security", vec![AgentRole::Analyzer, AgentRole::Specialist]),
        ]
        .into_iter()
        .collect();

        let mut roles = role_mappings
            .get(&task_type)
            .cloned()
            .unwrap_or_else(|| vec![AgentRole::Analyzer]);

        let text_lower = task_description.to_lowercase();
        if text_lower.contains("review") && !roles.contains(&AgentRole::Reviewer) {
            roles.push(AgentRole::Reviewer);
        }
        if text_lower.contains("test") && !roles.contains(&AgentRole::Tester) {
            roles.push(AgentRole::Tester);
        }
        if text_lower.contains("document") && !roles.contains(&AgentRole::Documenter) {
            roles.push(AgentRole::Documenter);
        }
        if text_lower.contains("optimize") && !roles.contains(&AgentRole::Optimizer) {
            roles.push(AgentRole::Optimizer);
        }

        roles.sort();
        roles.dedup();
        roles
    }

    fn determine_priority(&self, task_description: &str) -> String {
        let text_lower = task_description.to_lowercase();

        let high_priority_indicators = [
            "urgent",
            "critical",
            "security",
            "immediately",
            "asap",
            "blocker",
            "emergency",
        ];
        let low_priority_indicators = [
            "if possible",
            "when convenient",
            "nice to have",
            "someday",
            "eventually",
        ];

        for indicator in high_priority_indicators.iter() {
            if text_lower.contains(indicator) {
                return "high".to_string();
            }
        }

        for indicator in low_priority_indicators.iter() {
            if text_lower.contains(indicator) {
                return "low".to_string();
            }
        }

        "medium".to_string()
    }
}

pub struct TeamOptimizer {
    #[allow(dead_code)]
    max_parallel: usize,
    #[allow(dead_code)]
    context_budget: usize,
    role_capabilities: HashMap<AgentRole, RoleCapabilities>,
}

#[derive(Debug, Clone)]
struct RoleCapabilities {
    efficiency: f64,
    cost_per_hour: usize,
    max_concurrent_tasks: usize,
}

impl TeamOptimizer {
    pub fn new() -> Self {
        let mut role_capabilities = HashMap::new();

        role_capabilities.insert(
            AgentRole::Analyzer,
            RoleCapabilities {
                efficiency: 1.0,
                cost_per_hour: 5000,
                max_concurrent_tasks: 3,
            },
        );

        role_capabilities.insert(
            AgentRole::Reviewer,
            RoleCapabilities {
                efficiency: 0.8,
                cost_per_hour: 4000,
                max_concurrent_tasks: 5,
            },
        );

        role_capabilities.insert(
            AgentRole::Tester,
            RoleCapabilities {
                efficiency: 1.2,
                cost_per_hour: 6000,
                max_concurrent_tasks: 2,
            },
        );

        role_capabilities.insert(
            AgentRole::Documenter,
            RoleCapabilities {
                efficiency: 1.0,
                cost_per_hour: 4000,
                max_concurrent_tasks: 4,
            },
        );

        role_capabilities.insert(
            AgentRole::Optimizer,
            RoleCapabilities {
                efficiency: 0.9,
                cost_per_hour: 7000,
                max_concurrent_tasks: 2,
            },
        );

        role_capabilities.insert(
            AgentRole::Specialist,
            RoleCapabilities {
                efficiency: 1.5,
                cost_per_hour: 8000,
                max_concurrent_tasks: 1,
            },
        );

        Self {
            max_parallel: 3,
            context_budget: 200_000,
            role_capabilities,
        }
    }

    pub fn optimize_team(&self, task_analysis: &TaskAnalysis) -> Result<TeamComposition> {
        let team_size = self.determine_team_size(task_analysis);
        let roles = self.allocate_roles(task_analysis, team_size)?;
        let workload_distribution = self.distribute_workload(task_analysis, &roles)?;
        let completion_time = self.estimate_completion_time(task_analysis, &roles);
        let cost = self.estimate_cost(task_analysis, &roles, completion_time);
        let efficiency =
            self.calculate_efficiency_score(task_analysis, team_size, completion_time, cost);

        Ok(TeamComposition {
            team_size,
            roles,
            workload_distribution,
            estimated_completion_time: completion_time,
            cost_estimate: cost,
            efficiency_score: efficiency,
        })
    }

    fn determine_team_size(&self, task_analysis: &TaskAnalysis) -> usize {
        let base_sizes: HashMap<TaskComplexity, usize> = vec![
            (TaskComplexity::Simple, 1),
            (TaskComplexity::Moderate, 2),
            (TaskComplexity::Complex, 3),
            (TaskComplexity::VeryComplex, 5),
        ]
        .into_iter()
        .collect();

        let base_size = *base_sizes.get(&task_analysis.complexity).unwrap_or(&1);
        let required_count = task_analysis.required_roles.len();
        let mut team_size = base_size.max(required_count);

        if task_analysis.subtasks.len() > 3 {
            team_size += 1;
        }

        if task_analysis.priority == "high" && team_size < 3 {
            team_size += 1;
        }

        team_size.min(8)
    }

    fn allocate_roles(
        &self,
        task_analysis: &TaskAnalysis,
        team_size: usize,
    ) -> Result<Vec<RoleAllocation>> {
        let mut required_roles = task_analysis.required_roles.clone();

        while required_roles.len() < team_size {
            required_roles.push(AgentRole::Analyzer);
        }

        required_roles.truncate(team_size);

        let mut allocations = Vec::new();
        for (i, role) in required_roles.iter().enumerate() {
            let capabilities = self
                .role_capabilities
                .get(role)
                .ok_or_else(|| format!("Unknown role: {:?}", role))?;

            allocations.push(RoleAllocation {
                agent_id: format!("agent_{}", i + 1),
                role: role.as_str().to_string(),
                efficiency: capabilities.efficiency,
                cost_per_hour: capabilities.cost_per_hour,
                max_concurrent_tasks: capabilities.max_concurrent_tasks,
                primary_tasks: vec![],
            });
        }

        Ok(allocations)
    }

    fn distribute_workload(
        &self,
        task_analysis: &TaskAnalysis,
        roles: &[RoleAllocation],
    ) -> Result<HashMap<String, Workload>> {
        let mut distribution = HashMap::new();
        let total_workload = task_analysis.estimated_effort;

        let total_efficiency: f64 = roles.iter().map(|r| r.efficiency).sum();

        for role in roles {
            let share = (role.efficiency / total_efficiency) * total_workload;

            distribution.insert(
                role.agent_id.clone(),
                Workload {
                    hours: share,
                    tasks_assigned: 0,
                    utilization: 0.0,
                },
            );
        }

        let mut available_agents: Vec<RoleAllocation> = roles.to_vec();
        available_agents.sort_by(|a, b| {
            b.efficiency
                .partial_cmp(&a.efficiency)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.max_concurrent_tasks.cmp(&a.max_concurrent_tasks))
        });

        for (i, _subtask) in task_analysis.subtasks.iter().enumerate() {
            if i < available_agents.len() {
                let agent_id = &available_agents[i].agent_id;
                if let Some(workload) = distribution.get_mut(agent_id) {
                    workload.tasks_assigned += 1;
                }
            }
        }

        for role in roles {
            if let Some(workload) = distribution.get_mut(&role.agent_id) {
                let max_hours = 8.0;
                workload.utilization = (workload.hours / max_hours).min(1.0);
            }
        }

        Ok(distribution)
    }

    fn estimate_completion_time(
        &self,
        task_analysis: &TaskAnalysis,
        roles: &[RoleAllocation],
    ) -> f64 {
        let base_time = task_analysis.estimated_effort;

        let total_efficiency: f64 = roles.iter().map(|r| r.efficiency).sum();

        let parallelizable = 0.8;
        let serial = 0.2;

        let speedup = 1.0 / (serial + (parallelizable / total_efficiency));

        let mut completion_time = base_time / speedup;

        let coordination_overhead = roles.len().saturating_sub(2) as f64 * 0.1;
        completion_time *= 1.0 + coordination_overhead;

        completion_time
    }

    fn estimate_cost(
        &self,
        _task_analysis: &TaskAnalysis,
        roles: &[RoleAllocation],
        completion_time: f64,
    ) -> usize {
        let mut total_cost = 0.0;

        for role in roles {
            let agent_hours = completion_time / roles.len() as f64;
            let cost = role.cost_per_hour as f64 * agent_hours;
            total_cost += cost;
        }

        total_cost *= 1.1;

        total_cost.round() as usize
    }

    fn calculate_efficiency_score(
        &self,
        task_analysis: &TaskAnalysis,
        team_size: usize,
        completion_time: f64,
        cost: usize,
    ) -> f64 {
        let target_time = task_analysis.estimated_effort / 2.0;
        let time_score = (target_time / completion_time).min(1.0);

        let base_cost = 5000.0 * task_analysis.estimated_effort;
        let cost_score = (base_cost / cost as f64).min(1.0);

        let optimal_sizes: HashMap<TaskComplexity, usize> = vec![
            (TaskComplexity::Simple, 1),
            (TaskComplexity::Moderate, 2),
            (TaskComplexity::Complex, 3),
            (TaskComplexity::VeryComplex, 5),
        ]
        .into_iter()
        .collect();

        let optimal_size = *optimal_sizes.get(&task_analysis.complexity).unwrap_or(&3);
        let size_diff = (team_size as isize - optimal_size as isize).abs() as f64;
        let size_score = (1.0 - (size_diff * 0.2)).max(0.0);

        let efficiency = (time_score * 0.4) + (cost_score * 0.3) + (size_score * 0.3);

        (efficiency * 100.0).round() / 100.0
    }
}

impl Default for TaskAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TeamOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
