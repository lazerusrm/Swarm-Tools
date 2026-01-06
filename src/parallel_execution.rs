use crate::types::*;
use serde::{Deserialize, Serialize};

pub struct ParallelManager {
    max_parallel: usize,
    #[allow(dead_code)]
    context_budget: usize,
}

impl ParallelManager {
    pub fn new(max_parallel: usize, context_budget: usize) -> Self {
        Self {
            max_parallel,
            context_budget,
        }
    }

    pub fn plan_execution(
        &self,
        tasks: &[AgentTask],
        mode: ExecutionMode,
    ) -> Result<ExecutionPlan> {
        let total_tokens: usize = tasks.iter().map(|t| t.estimated_tokens).sum();

        let (groups, time_estimate) = match mode {
            ExecutionMode::Sequential => {
                let groups: Vec<Vec<AgentTask>> = tasks.iter().map(|t| vec![t.clone()]).collect();
                let time_estimate = total_tokens as f64 / 1000.0;
                (groups, time_estimate)
            }
            ExecutionMode::ParallelSafe | ExecutionMode::ParallelOptimal => {
                let groups = self.group_tasks(tasks);
                let time_estimate = total_tokens as f64 / (self.max_parallel as f64 * 1000.0);
                (groups, time_estimate)
            }
        };

        Ok(ExecutionPlan {
            mode,
            groups,
            time_estimate,
            token_estimate: total_tokens,
        })
    }

    pub fn simulate_execution(
        &self,
        tasks: &[AgentTask],
        mode: ExecutionMode,
    ) -> Result<Vec<ExecutionResult>> {
        let plan = self.plan_execution(tasks, mode)?;
        let mut results = Vec::new();

        for group in &plan.groups {
            let max_time = group
                .iter()
                .map(|t| t.estimated_tokens as f64 / 1000.0)
                .fold(0.0_f64, |acc, x| acc.max(x));

            for task in group {
                let time_taken = max_time;
                let tokens_used = task.estimated_tokens;
                let success = true;

                results.push(ExecutionResult {
                    name: task.name.clone(),
                    time_taken,
                    success,
                    tokens_used,
                });
            }
        }

        Ok(results)
    }

    pub fn compare_modes(&self, tasks: &[AgentTask]) -> Result<ModeComparison> {
        let sequential_results = self.simulate_execution(tasks, ExecutionMode::Sequential)?;
        let parallel_results = self.simulate_execution(tasks, ExecutionMode::ParallelSafe)?;

        let sequential_time = sequential_results.iter().map(|r| r.time_taken).sum();
        let parallel_time = parallel_results.iter().map(|r| r.time_taken).sum();

        let sequential_tokens = sequential_results.iter().map(|r| r.tokens_used).sum();
        let parallel_tokens = parallel_results.iter().map(|r| r.tokens_used).sum();

        let speedup = if parallel_time > 0.0 {
            sequential_time / parallel_time
        } else {
            1.0
        };

        let time_savings = if sequential_time > 0.0 {
            ((sequential_time - parallel_time) / sequential_time) * 100.0
        } else {
            0.0
        };

        let recommendation = if speedup > 1.1 {
            "parallel".to_string()
        } else {
            "sequential".to_string()
        };

        Ok(ModeComparison {
            sequential: ExecutionMetrics {
                time: sequential_time,
                tokens: sequential_tokens,
            },
            parallel: ParallelMetrics {
                time: parallel_time,
                tokens: parallel_tokens,
                speedup,
                time_savings_pct: time_savings,
            },
            recommendation,
            speedup_vs_sequential: format!("{:.2}x", speedup),
            time_savings_pct: format!("{:.1}%", time_savings),
        })
    }

    fn group_tasks(&self, tasks: &[AgentTask]) -> Vec<Vec<AgentTask>> {
        let mut groups = Vec::new();
        for chunk in tasks.chunks(self.max_parallel) {
            groups.push(chunk.to_vec());
        }
        groups
    }
}

impl Default for ParallelManager {
    fn default() -> Self {
        Self::new(3, 200_000)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub name: String,
    pub task_desc: String,
    pub estimated_tokens: usize,
}

impl AgentTask {
    pub fn new(name: String, task_desc: String, estimated_tokens: usize) -> Self {
        Self {
            name,
            task_desc,
            estimated_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub name: String,
    pub time_taken: f64,
    pub success: bool,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeComparison {
    pub sequential: ExecutionMetrics,
    pub parallel: ParallelMetrics,
    pub recommendation: String,
    pub speedup_vs_sequential: String,
    pub time_savings_pct: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub mode: ExecutionMode,
    pub groups: Vec<Vec<AgentTask>>,
    pub time_estimate: f64,
    pub token_estimate: usize,
}
