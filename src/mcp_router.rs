use crate::feature_config::McpRoutingConfig;
use crate::types::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum McpRoutingDecision {
    Allow,
    Deny { reason: String },
    ModifyArgs { new_args: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRoutingResult {
    pub decision: McpRoutingDecision,
    pub tool_name: String,
    pub role: AgentRole,
    pub token_savings_estimate: Option<usize>,
}

pub struct McpRouter {
    config: McpRoutingConfig,
    role_tool_filters: HashMap<AgentRole, Vec<String>>,
    default_tools: Vec<String>,
}

impl McpRouter {
    pub fn new() -> Self {
        Self::with_config(McpRoutingConfig::default())
    }

    pub fn with_config(config: McpRoutingConfig) -> Self {
        let role_tool_filters = convert_role_filters(config.role_tool_filters.clone());
        let default_tools = config.default_tools.clone().unwrap_or_default();

        Self {
            config,
            role_tool_filters,
            default_tools,
        }
    }

    pub fn route_tool_call(
        &self,
        role: AgentRole,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> McpRoutingResult {
        if !self.config.enabled {
            return McpRoutingResult {
                decision: McpRoutingDecision::Allow,
                tool_name: tool_name.to_string(),
                role,
                token_savings_estimate: None,
            };
        }

        let allowed_tools = self.role_tool_filters.get(&role);

        if let Some(tools) = allowed_tools {
            if tools.iter().any(|t| tool_name.contains(t)) {
                let modified = self.modify_args_if_needed(tool_name, args, role);
                return McpRoutingResult {
                    decision: modified,
                    tool_name: tool_name.to_string(),
                    role,
                    token_savings_estimate: self.estimate_token_savings(args),
                };
            }
        }

        if self.default_tools.iter().any(|t| tool_name.contains(t)) {
            let modified = self.modify_args_if_needed(tool_name, args, role);
            return McpRoutingResult {
                decision: modified,
                tool_name: tool_name.to_string(),
                role,
                token_savings_estimate: self.estimate_token_savings(args),
            };
        }

        let reason = format!(
            "Tool '{}' not in allowed list for role '{}'",
            tool_name,
            role.as_str()
        );

        McpRoutingResult {
            decision: McpRoutingDecision::Deny { reason },
            tool_name: tool_name.to_string(),
            role,
            token_savings_estimate: None,
        }
    }

    fn modify_args_if_needed(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
        _role: AgentRole,
    ) -> McpRoutingDecision {
        let args_str = args.to_string();
        let original_len = args_str.len();

        let mut modified_args = args.clone();

        if tool_name.contains("read_file") || tool_name.contains("browse_file") {
            if let Some(obj) = modified_args.as_object_mut() {
                if let Some(context) = obj.get("context") {
                    if context.as_str().map(|s| s.len()).unwrap_or(0) > 5000 {
                        obj.remove("context");
                        let savings = original_len.saturating_sub(modified_args.to_string().len());
                        return McpRoutingDecision::ModifyArgs {
                            new_args: modified_args,
                        };
                    }
                }
            }
        }

        if tool_name.contains("search") || tool_name.contains("grep") {
            if let Some(obj) = modified_args.as_object_mut() {
                if let Some(query) = obj.get("query") {
                    if let Some(query_str) = query.as_str() {
                        if query_str.len() > 500 {
                            let trimmed = &query_str[..500];
                            obj["query"] = serde_json::Value::String(trimmed.to_string());
                            return McpRoutingDecision::ModifyArgs {
                                new_args: modified_args,
                            };
                        }
                    }
                }
            }
        }

        McpRoutingDecision::Allow
    }

    fn estimate_token_savings(&self, args: &serde_json::Value) -> Option<usize> {
        let args_str = args.to_string();
        let tokens = args_str.len() / 4;

        if tokens > 100 {
            Some(tokens)
        } else {
            None
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

fn convert_role_filters(
    filters: Option<HashMap<String, Vec<String>>>,
) -> HashMap<AgentRole, Vec<String>> {
    let mut result = HashMap::new();

    if let Some(filters) = filters {
        for (role_str, tools) in filters {
            if let Ok(role) = role_str.parse::<AgentRole>() {
                result.insert(role, tools);
            }
        }
    }

    result
}

impl Default for McpRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskComplexity;

    #[test]
    fn test_mcp_router_allow_relevant_tool() {
        let router = McpRouter::new();
        let args = serde_json::json!({"path": "/test/file.rs"});
        let result = router.route_tool_call(AgentRole::Extractor, "read_file", &args);
        assert_eq!(result.decision, McpRoutingDecision::Allow);
    }

    #[test]
    fn test_mcp_router_deny_irrelevant_tool() {
        let router = McpRouter::new();
        let args = serde_json::json!({"query": "test"});
        let result = router.route_tool_call(AgentRole::Extractor, "web_search", &args);
        match &result.decision {
            McpRoutingDecision::Deny { reason } => {
                assert!(reason.contains("not in allowed list"));
            }
            _ => panic!("Expected Deny decision"),
        }
    }

    #[test]
    fn test_mcp_router_modify_large_context() {
        let large_context = "x".repeat(6000);
        let args = serde_json::json!({
            "path": "/test/file.rs",
            "context": large_context
        });
        let router = McpRouter::new();
        let result = router.route_tool_call(AgentRole::Extractor, "read_file", &args);

        match &result.decision {
            McpRoutingDecision::ModifyArgs { new_args } => {
                assert!(!new_args.get("context").is_some());
            }
            _ => panic!("Expected ModifyArgs decision"),
        }
    }

    #[test]
    fn test_mcp_router_disabled() {
        let mut config = McpRoutingConfig::default();
        config.enabled = false;
        let router = McpRouter::with_config(config);

        let args = serde_json::json!({"query": "test"});
        let result = router.route_tool_call(AgentRole::Extractor, "web_search", &args);
        assert_eq!(result.decision, McpRoutingDecision::Allow);
    }

    #[test]
    fn test_mcp_router_analyzer_tools() {
        let router = McpRouter::new();
        let args = serde_json::json!({"pattern": "fn test", "path": "/src"});
        let result = router.route_tool_call(AgentRole::Analyzer, "search_code", &args);
        assert_eq!(result.decision, McpRoutingDecision::Allow);
    }

    #[test]
    fn test_mcp_router_default_tools() {
        let router = McpRouter::new();
        let args = serde_json::json!({"message": "hello"});
        let result = router.route_tool_call(AgentRole::General, "send_message", &args);
        assert_eq!(result.decision, McpRoutingDecision::Allow);
    }

    #[test]
    fn test_token_savings_estimate() {
        let router = McpRouter::new();
        let args = serde_json::json!({"message": "x".repeat(1000)});
        let result = router.route_tool_call(AgentRole::General, "message", &args);
        assert!(result.token_savings_estimate.is_some());
        assert!((result.token_savings_estimate.unwrap() > 0));
    }
}
