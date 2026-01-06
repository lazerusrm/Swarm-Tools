use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl TaskComplexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskComplexity::Simple => "simple",
            TaskComplexity::Moderate => "moderate",
            TaskComplexity::Complex => "complex",
            TaskComplexity::VeryComplex => "very_complex",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AgentRole {
    Analyzer,
    Reviewer,
    Tester,
    Documenter,
    Optimizer,
    Specialist,
}

impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentRole::Analyzer => "analyzer",
            AgentRole::Reviewer => "reviewer",
            AgentRole::Tester => "tester",
            AgentRole::Documenter => "documenter",
            AgentRole::Optimizer => "optimizer",
            AgentRole::Specialist => "specialist",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    Sequential,
    ParallelSafe,
    ParallelOptimal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LoopType {
    ExactLoop,
    SemanticLoop,
    StateOscillation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CommunicationPriority {
    Critical = 1,
    High = 2,
    Medium = 3,
    Low = 4,
    Redundant = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    pub complexity: TaskComplexity,
    pub task_type: String,
    pub subtasks: Vec<String>,
    pub estimated_effort: f64,
    pub required_roles: Vec<AgentRole>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamComposition {
    pub team_size: usize,
    pub roles: Vec<RoleAllocation>,
    pub workload_distribution: std::collections::HashMap<String, Workload>,
    pub estimated_completion_time: f64,
    pub cost_estimate: usize,
    pub efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAllocation {
    pub agent_id: String,
    pub role: String,
    pub efficiency: f64,
    pub cost_per_hour: usize,
    pub max_concurrent_tasks: usize,
    pub primary_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub hours: f64,
    pub tasks_assigned: usize,
    pub utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original: String,
    pub original_analysis: PromptAnalysis,
    pub optimized: String,
    pub optimization_strategy: String,
    pub token_reduction_pct: f64,
    pub clarity_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAnalysis {
    pub estimated_tokens: usize,
    pub has_redundancy: bool,
    pub has_ambiguity: bool,
    pub has_long_explanations: bool,
    pub has_in_context_files: bool,
    pub clarity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopDetection {
    pub detection_type: LoopType,
    pub agent_id: String,
    pub loop_count: usize,
    pub prompt_hash: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub name: String,
    pub task_desc: String,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub name: String,
    pub time_taken: f64,
    pub success: bool,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBenefitResult {
    pub decision: String,
    pub message: String,
    pub cost: f64,
    pub benefit: f64,
    pub ratio: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionStats {
    pub total_decisions: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub execute_pct: f64,
    pub adjust_scope_pct: f64,
    pub request_assistance_pct: f64,
    pub skip_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionResult {
    pub status: String,
    pub message: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub max_parallel_agents: usize,
    pub context_budget: usize,
    pub context_threshold: f64,
    pub loop_exact_threshold: usize,
    pub loop_semantic_threshold: usize,
    pub loop_state_oscillation_threshold: usize,
    pub semantic_similarity_threshold: f64,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_parallel_agents: 3,
            context_budget: 200_000,
            context_threshold: 0.7,
            loop_exact_threshold: 3,
            loop_semantic_threshold: 5,
            loop_state_oscillation_threshold: 3,
            semantic_similarity_threshold: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationAnalysis {
    pub priority: CommunicationPriority,
    pub redundancy_score: f64,
    pub relevance_score: f64,
    pub should_include: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub mode: ExecutionMode,
    pub groups: Vec<Vec<AgentTask>>,
    pub time_estimate: f64,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub time: f64,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelMetrics {
    pub time: f64,
    pub tokens: usize,
    pub speedup: f64,
    pub time_savings_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeComparison {
    pub sequential: ExecutionMetrics,
    pub parallel: ParallelMetrics,
    pub recommendation: String,
    pub speedup_vs_sequential: String,
    pub time_savings_pct: String,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
