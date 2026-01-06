pub use crate::types::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum OptimizationPriority {
    High,
    Medium,
    Low,
}

pub trait OptimizationStrategy: Send + Sync {
    fn optimize(&self, prompt: &str, role: &AgentRole) -> Result<OptimizationResult>;
    fn priority(&self) -> OptimizationPriority;
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
pub struct OptimizationResult {
    pub original: String,
    pub original_analysis: PromptAnalysis,
    pub optimized: String,
    pub optimization_strategy: String,
    pub token_reduction_pct: f64,
    pub clarity_improvement: f64,
}

pub struct PromptOptimizer {
    redundant_patterns: Vec<(Regex, f64)>,
    ambiguity_patterns: Vec<(Regex, f64)>,
}

impl PromptOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            redundant_patterns: vec![
                (Regex::new(r"\bplease\b")?, 0.9),
                (Regex::new(r"\bkindly\b")?, 0.8),
                (Regex::new(r"\bas previously mentioned\b")?, 0.85),
                (Regex::new(r"\bin conclusion\b")?, 0.85),
            ],
            ambiguity_patterns: vec![
                (Regex::new(r"\b(some|a few|several|approximately)\b")?, 0.8),
                (
                    Regex::new(r"\b(if necessary|if possible|as appropriate)\b")?,
                    0.7,
                ),
                (Regex::new(r"\b(and|or|maybe)\s+(then|also)\b")?, 0.75),
            ],
        })
    }

    pub fn optimize_prompt(
        &self,
        original_prompt: &str,
        _agent_role: &AgentRole,
    ) -> Result<OptimizationResult> {
        let original_analysis = self.analyze_prompt(original_prompt)?;

        let optimized = self.apply_conciseness(original_prompt);
        let optimized = self.apply_clarity(&optimized);
        let optimized = self.apply_context_awareness(&optimized);

        let token_reduction = ((original_analysis.estimated_tokens as f64
            - (optimized.len() / 4) as f64)
            / original_analysis.estimated_tokens as f64)
            * 100.0;

        let clarity_improvement: f64 = if original_analysis.has_redundancy {
            0.2
        } else if original_analysis.has_ambiguity {
            0.15
        } else if original_analysis.has_long_explanations {
            0.1
        } else {
            0.0
        };

        Ok(OptimizationResult {
            original: original_prompt.to_string(),
            original_analysis,
            optimized,
            optimization_strategy: "conciseness".to_string(),
            token_reduction_pct: token_reduction.clamp(0.0, 50.0),
            clarity_improvement: clarity_improvement.min(1.0),
        })
    }

    fn analyze_prompt(&self, prompt: &str) -> Result<PromptAnalysis> {
        let estimated_tokens = prompt.len() / 4;

        let has_redundancy = self
            .redundant_patterns
            .iter()
            .any(|(pattern, _)| pattern.is_match(prompt));
        let has_ambiguity = self
            .ambiguity_patterns
            .iter()
            .any(|(pattern, _)| pattern.is_match(prompt));

        let sentences: Vec<&str> = prompt.split('.').collect();
        let long_sentences = sentences
            .iter()
            .filter(|s| s.split_whitespace().count() > 20)
            .count();
        let has_long_explanations = long_sentences > 2;

        let has_in_context_files = prompt.contains("```") || prompt.contains("file:");

        let clarity_issues = [has_redundancy, has_ambiguity, has_long_explanations]
            .iter()
            .filter(|&&x| x)
            .count() as f64;

        let clarity_score = (1.0 - (clarity_issues * 0.2)).clamp(0.0, 1.0);

        Ok(PromptAnalysis {
            estimated_tokens,
            has_redundancy,
            has_ambiguity,
            has_long_explanations,
            has_in_context_files,
            clarity_score,
        })
    }

    fn apply_conciseness(&self, prompt: &str) -> String {
        let polite_re = Regex::new(r"\b(please|kindly|thank you|in conclusion|finally)\b").unwrap();
        let redundant_re =
            Regex::new(r"\b(as previously mentioned|as stated above|as you know)\b").unwrap();

        let mut result = polite_re.replace_all(prompt, "").to_string();
        result = redundant_re.replace_all(&result, "").to_string();

        result
    }

    fn apply_clarity(&self, prompt: &str) -> String {
        let mut result = prompt.to_string();

        if !Regex::new(r"\b(limit|constraint|max|should not)\b")
            .unwrap()
            .is_match(&result)
        {
            result.push_str(
                "\n\nConstraints:\n- Use file-based operations\n- Return 200-word summary",
            );
        }

        if !Regex::new(r"\b(success criteria|complete|finish|output)\b")
            .unwrap()
            .is_match(&result)
        {
            result.push_str("\n\nSuccess Criteria:\n- Analysis complete\n- Results written to file\n- Summary returned");
        }

        result
    }

    fn apply_context_awareness(&self, prompt: &str) -> String {
        if prompt.contains("```") || prompt.len() > 2000 {
            format!("Use Read tool to access files. Task:\n\n{}", prompt)
        } else {
            prompt.to_string()
        }
    }
}

pub struct RoleOptimizer {
    role_templates: HashMap<String, RoleTemplate>,
}

#[derive(Debug, Clone)]
struct RoleTemplate {
    capabilities: Vec<String>,
    tools: Vec<String>,
    responsibilities: String,
    scope: String,
}

impl RoleOptimizer {
    pub fn new() -> Self {
        let mut role_templates = HashMap::new();

        role_templates.insert(
            "code_analyzer".to_string(),
            RoleTemplate {
                capabilities: vec![
                    "read".to_string(),
                    "analyze".to_string(),
                    "identify_issues".to_string(),
                ],
                tools: vec!["Read".to_string(), "Write".to_string()],
                responsibilities: "Analyze code, identify issues, report findings".to_string(),
                scope: "Static analysis only, no code modifications".to_string(),
            },
        );

        role_templates.insert(
            "tester".to_string(),
            RoleTemplate {
                capabilities: vec![
                    "run_tests".to_string(),
                    "identify_failures".to_string(),
                    "generate_report".to_string(),
                ],
                tools: vec!["Bash".to_string(), "Write".to_string()],
                responsibilities: "Run tests, check results, generate report".to_string(),
                scope: "Testing only, no test modifications".to_string(),
            },
        );

        role_templates.insert(
            "code_reviewer".to_string(),
            RoleTemplate {
                capabilities: vec![
                    "review_code".to_string(),
                    "check_quality".to_string(),
                    "provide_feedback".to_string(),
                ],
                tools: vec!["Read".to_string(), "Write".to_string()],
                responsibilities: "Review code, check quality, provide feedback".to_string(),
                scope: "Code review only, no modifications".to_string(),
            },
        );

        Self { role_templates }
    }

    pub fn optimize_role(
        &self,
        current_role: &str,
        agent_name: &str,
    ) -> Result<OptimizationResult> {
        let original_analysis = self.analyze_role(current_role)?;

        let optimized_role = if let Some(template) = self.role_templates.get(agent_name) {
            self.format_role_from_template(agent_name, template)
        } else {
            self.create_custom_role(agent_name, current_role)
        };

        let token_reduction = ((original_analysis.estimated_tokens as f64
            - (optimized_role.len() / 4) as f64)
            / original_analysis.estimated_tokens as f64)
            * 100.0;

        let specificity_improvement: f64 = if original_analysis.has_vague_capabilities
            || original_analysis.has_over_capabilities
            || original_analysis.has_ambiguous_scope
            || original_analysis.has_missing_protocols
        {
            0.25
        } else {
            0.0
        };

        Ok(OptimizationResult {
            original: current_role.to_string(),
            original_analysis: PromptAnalysis {
                estimated_tokens: original_analysis.estimated_tokens,
                has_redundancy: false,
                has_ambiguity: false,
                has_long_explanations: false,
                has_in_context_files: false,
                clarity_score: original_analysis.specificity_score,
            },
            optimized: optimized_role,
            optimization_strategy: "role_template".to_string(),
            token_reduction_pct: token_reduction.clamp(0.0, 50.0),
            clarity_improvement: specificity_improvement.min(1.0),
        })
    }

    fn analyze_role(&self, role: &str) -> Result<RoleAnalysis> {
        let estimated_tokens = role.len() / 4;

        let vague_re =
            Regex::new(r"\b(do various things|handle multiple tasks|perform analysis)\b")?;
        let has_vague_capabilities = vague_re.is_match(role);

        let capability_re = Regex::new(r"\b(can|able to|capable of)\b")?;
        let capability_count = capability_re.find_iter(role).count();
        let has_over_capabilities = capability_count > 5;

        let ambiguous_re = Regex::new(r"\b(as needed|when appropriate|if necessary)\b")?;
        let has_ambiguous_scope = ambiguous_re.is_match(role);

        let has_output_format =
            Regex::new(r"\b(output|format|return|communicate)\b")?.is_match(role);
        let has_tools_specified = Regex::new(r"\b(tools|using|access|with)\b")?.is_match(role);
        let has_missing_protocols = !(has_output_format || has_tools_specified);

        let specificity_issues = [
            has_vague_capabilities,
            has_over_capabilities,
            has_ambiguous_scope,
            has_missing_protocols,
        ]
        .iter()
        .filter(|&&x| x)
        .count() as f64;

        let specificity_score = (1.0 - (specificity_issues * 0.2)).clamp(0.0, 1.0);

        Ok(RoleAnalysis {
            estimated_tokens,
            has_vague_capabilities,
            has_over_capabilities,
            has_ambiguous_scope,
            has_missing_protocols,
            specificity_score,
        })
    }

    fn format_role_from_template(&self, agent_name: &str, template: &RoleTemplate) -> String {
        format!(
            "Role: {}\n\nType: {}\n\nCapabilities:\n{}\n\nTools: {}\n\nResponsibilities:\n{}\n\nScope:\n{}\n\nOutput Format:\n- Detailed analysis in results/[filename].md\n- 200-word summary in response\n- File paths for references",
            agent_name,
            agent_name.replace("_", " "),
            template.capabilities.iter().map(|c| format!("- {}", c)).collect::<Vec<_>>().join("\n"),
            template.tools.join(", "),
            template.responsibilities,
            template.scope
        )
    }

    fn create_custom_role(&self, agent_name: &str, _current_role: &str) -> String {
        format!(
            "Role: {}\n\nType: Custom Specialized Agent\n\nCapabilities:\n- perform_tasks\n\nTools: Read, Write\n\nResponsibilities:\n- complete_assigned_task\n\nScope:\n- Specialized tasks based on capabilities\n- File-based outputs for large content\n- Return 200-word summaries\n\nOutput Format:\n- Detailed analysis in results/[filename].md\n- 200-word summary in response\n- File paths for references",
            agent_name
        )
    }
}

impl Default for RoleOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct RoleAnalysis {
    estimated_tokens: usize,
    has_vague_capabilities: bool,
    has_over_capabilities: bool,
    has_ambiguous_scope: bool,
    has_missing_protocols: bool,
    specificity_score: f64,
}

pub struct OmackOptimizer {
    prompt_optimizer: PromptOptimizer,
    role_optimizer: RoleOptimizer,
}

impl OmackOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            prompt_optimizer: PromptOptimizer::new()?,
            role_optimizer: RoleOptimizer::new(),
        })
    }

    pub fn optimize_agent_configuration(
        &self,
        prompt: &str,
        role: &str,
        agent_name: &str,
    ) -> Result<OptimizationResult> {
        let prompt_result = self
            .prompt_optimizer
            .optimize_prompt(prompt, &AgentRole::Analyzer)?;
        let role_result = self.role_optimizer.optimize_role(role, agent_name)?;

        let overall_token_reduction =
            (prompt_result.token_reduction_pct + role_result.token_reduction_pct) / 2.0;
        let overall_improvement =
            (prompt_result.clarity_improvement + role_result.clarity_improvement) / 2.0;

        Ok(OptimizationResult {
            original: prompt.to_string(),
            original_analysis: prompt_result.original_analysis,
            optimized: prompt_result.optimized,
            optimization_strategy: format!(
                "{} + {}",
                prompt_result.optimization_strategy, role_result.optimization_strategy
            ),
            token_reduction_pct: overall_token_reduction,
            clarity_improvement: overall_improvement,
        })
    }
}

impl Default for OmackOptimizer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
