//! Integration layer between agent runtime and toka-tools registry.
//!
//! This module provides the integration between the agent execution system
//! and the enhanced tool registry with external tool support.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, info, warn};

use toka_llm_gateway::LlmGateway;
use toka_runtime::Runtime;
use toka_tools::{ToolRegistry, ToolParams, ToolResult, ToolRegistryExt, ToolDiscoveryConfig};
use toka_types::{AgentConfig, TaskConfig, EntityId};

use crate::{
    AgentContext, TaskExecutor, ProgressReporter, TaskResult as AgentTaskResult,
    AgentRuntimeError, AgentRuntimeResult,
};

/// Enhanced task executor with tool registry integration
pub struct ToolRegistryTaskExecutor {
    /// Tool registry for executing tools
    tool_registry: Arc<ToolRegistry>,
    /// LLM gateway for intelligent task interpretation
    llm_gateway: Arc<LlmGateway>,
    /// Runtime for kernel operations
    runtime: Arc<Runtime>,
    /// Progress reporter
    progress_reporter: Arc<ProgressReporter>,
}

impl ToolRegistryTaskExecutor {
    /// Create a new tool registry task executor
    pub async fn new(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        let tool_registry = Arc::new(ToolRegistry::new_empty());
        
        // Auto-register existing tools
        let count = tool_registry.auto_register_tools().await?;
        info!("Auto-registered {} tools for agent runtime", count);
        
        let progress_reporter = Arc::new(ProgressReporter::new(runtime.clone()));
        
        Ok(Self {
            tool_registry,
            llm_gateway,
            runtime,
            progress_reporter,
        })
    }
    
    /// Create with custom tool registry
    pub fn with_registry(
        tool_registry: Arc<ToolRegistry>,
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Self {
        let progress_reporter = Arc::new(ProgressReporter::new(runtime.clone()));
        
        Self {
            tool_registry,
            llm_gateway,
            runtime,
            progress_reporter,
        }
    }
    
    /// Execute a task using the tool registry
    pub async fn execute_task(
        &self,
        task: &TaskConfig,
        context: &AgentContext,
    ) -> AgentRuntimeResult<AgentTaskResult> {
        info!("Executing task '{}' using tool registry", task.description);
        
        // Parse task to determine tool and parameters
        let (tool_name, tool_params) = self.parse_task_for_tools(task, context).await?;
        
        // Validate tool exists in registry
        if self.tool_registry.get_tool(&tool_name).await.is_none() {
            return Err(AgentRuntimeError::ExecutionFailed(
                format!("Tool not found in registry: {}", tool_name)
            ));
        }
        
        // Execute tool through registry
        let tool_result = self.tool_registry.execute_tool(&tool_name, &tool_params).await
            .map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;
        
        // Convert tool result to agent task result
        let agent_result = self.convert_tool_result(tool_result, task)?;
        
        // Report progress
        self.progress_reporter.report_task_completion(
            context.agent_id,
            task.description.clone(),
            agent_result.success,
        ).await.map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;
        
        Ok(agent_result)
    }
    
    /// Parse a task description to determine tool and parameters using LLM
    async fn parse_task_for_tools(
        &self,
        task: &TaskConfig,
        context: &AgentContext,
    ) -> AgentRuntimeResult<(String, ToolParams)> {
        // Get available tools
        let available_tools = self.tool_registry.list_tools().await;
        
        // Use LLM to interpret task and map to tool
        let prompt = format!(
            r#"
You are an intelligent task executor for Toka OS. Parse the following task and determine which tool to use.

Task: {}
Available Tools: {:?}
Agent Capabilities: {:?}

Respond with JSON in this format:
{{
    "tool_name": "tool-name",
    "parameters": {{
        "param1": "value1",
        "param2": "value2"
    }}
}}

Choose the most appropriate tool based on the task description and available tools.
If the task mentions validation, use validation tools.
If the task mentions building, use build tools.
If the task mentions monitoring, use monitoring tools.
"#,
            task.description,
            available_tools,
            context.config.security.capabilities_required
        );
        
        let llm_request = toka_llm_gateway::LlmRequest::new(prompt)?;
        let llm_response = self.llm_gateway.complete(llm_request).await
            .map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;
        
        // Parse LLM response
        let parsed: serde_json::Value = serde_json::from_str(&llm_response.content)
            .map_err(|e| AgentRuntimeError::ExecutionFailed(
                format!("Failed to parse LLM response: {}", e)
            ))?;
        
        let tool_name = parsed["tool_name"]
            .as_str()
            .ok_or_else(|| AgentRuntimeError::ExecutionFailed(
                "LLM response missing tool_name".to_string()
            ))?
            .to_string();
        
        let mut parameters = HashMap::new();
        if let Some(params_obj) = parsed["parameters"].as_object() {
            for (key, value) in params_obj {
                if let Some(str_value) = value.as_str() {
                    parameters.insert(key.clone(), str_value.to_string());
                }
            }
        }
        
        let tool_params = ToolParams {
            name: tool_name.clone(),
            args: parameters,
        };
        
        debug!("Task '{}' mapped to tool '{}' with params: {:?}", 
               task.description, tool_name, tool_params.args);
        
        Ok((tool_name, tool_params))
    }
    
    /// Convert a tool result to an agent task result
    fn convert_tool_result(
        &self,
        tool_result: ToolResult,
        task: &TaskConfig,
    ) -> AgentRuntimeResult<AgentTaskResult> {
        Ok(AgentTaskResult {
            task_id: task.description.clone(), // Using description as ID for simplicity
            success: tool_result.success,
            output: tool_result.output,
            execution_time: std::time::Duration::from_millis(tool_result.metadata.execution_time_ms),
            timestamp: chrono::DateTime::from_timestamp(tool_result.metadata.timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now),
            metadata: serde_json::json!({
                "tool_version": tool_result.metadata.tool_version,
                "tool_execution_time_ms": tool_result.metadata.execution_time_ms,
            }),
        })
    }
    
    /// Get the tool registry
    pub fn tool_registry(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }
    
    /// List available tools
    pub async fn list_available_tools(&self) -> Vec<String> {
        self.tool_registry.list_tools().await
    }
    
    /// Register additional tools
    pub async fn register_tool(&self, tool: Arc<dyn toka_tools::Tool + Send + Sync>) -> Result<()> {
        self.tool_registry.register_tool(tool).await
    }
    
    /// Auto-discover and register tools with custom configuration
    pub async fn auto_register_tools_with_config(&self, config: ToolDiscoveryConfig) -> Result<usize> {
        self.tool_registry.auto_register_tools_with_config(config).await
    }
}

/// Extension trait for AgentExecutor to support tool registry integration
pub trait AgentExecutorExt {
    /// Execute tasks using the tool registry
    async fn execute_with_tool_registry(
        &self,
        task: &TaskConfig,
        tool_executor: &ToolRegistryTaskExecutor,
    ) -> AgentRuntimeResult<AgentTaskResult>;
}

impl AgentExecutorExt for crate::AgentExecutor {
    async fn execute_with_tool_registry(
        &self,
        task: &TaskConfig,
        tool_executor: &ToolRegistryTaskExecutor,
    ) -> AgentRuntimeResult<AgentTaskResult> {
        // Get agent context (this would need to be exposed from AgentExecutor)
        // For now, we'll create a minimal context
        let context = self.get_context().await?;
        
        tool_executor.execute_task(task, &context).await
    }
}

/// Tool registry factory for different environments
pub struct ToolRegistryFactory;

impl ToolRegistryFactory {
    /// Create a tool registry for development environment
    pub async fn create_development_registry() -> Result<Arc<ToolRegistry>> {
        let registry = Arc::new(ToolRegistry::new_empty());
        
        // Register built-in tools
        registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
        
        // Auto-discover workspace tools with permissive settings
        let config = ToolDiscoveryConfig {
            search_directories: vec![
                std::path::PathBuf::from("scripts"),
                std::path::PathBuf::from("tools"),
                std::path::PathBuf::from("bin"),
            ],
            include_patterns: vec![
                "*.py".to_string(),
                "*.sh".to_string(),
                "*.bash".to_string(),
            ],
            exclude_patterns: vec![
                "*test*".to_string(),
            ],
            follow_symlinks: true,
            max_depth: 3,
        };
        
        let count = registry.auto_register_tools_with_config(config).await?;
        info!("Development registry auto-registered {} tools", count);
        
        Ok(registry)
    }
    
    /// Create a tool registry for production environment
    pub async fn create_production_registry() -> Result<Arc<ToolRegistry>> {
        let registry = Arc::new(ToolRegistry::new_empty());
        
        // Register only essential built-in tools
        registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
        
        // More restrictive auto-discovery for production
        let config = ToolDiscoveryConfig {
            search_directories: vec![
                std::path::PathBuf::from("tools/production"),
            ],
            include_patterns: vec![
                "*.py".to_string(),
                "*.sh".to_string(),
            ],
            exclude_patterns: vec![
                "*test*".to_string(),
                "*debug*".to_string(),
                "*dev*".to_string(),
            ],
            follow_symlinks: false,
            max_depth: 2,
        };
        
        let count = registry.auto_register_tools_with_config(config).await?;
        info!("Production registry auto-registered {} tools", count);
        
        Ok(registry)
    }
    
    /// Create an empty registry for testing
    pub fn create_test_registry() -> Arc<ToolRegistry> {
        Arc::new(ToolRegistry::new_empty())
    }
}

/// Integration helper for connecting agent runtime with tool registry
pub struct AgentRuntimeToolIntegration {
    tool_executor: ToolRegistryTaskExecutor,
}

impl AgentRuntimeToolIntegration {
    /// Create a new integration with development settings
    pub async fn new_development(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        let tool_registry = ToolRegistryFactory::create_development_registry().await?;
        let tool_executor = ToolRegistryTaskExecutor::with_registry(
            tool_registry,
            runtime,
            llm_gateway,
        );
        
        Ok(Self { tool_executor })
    }
    
    /// Create a new integration with production settings
    pub async fn new_production(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        let tool_registry = ToolRegistryFactory::create_production_registry().await?;
        let tool_executor = ToolRegistryTaskExecutor::with_registry(
            tool_registry,
            runtime,
            llm_gateway,
        );
        
        Ok(Self { tool_executor })
    }
    
    /// Get the tool executor
    pub fn tool_executor(&self) -> &ToolRegistryTaskExecutor {
        &self.tool_executor
    }
    
    /// Execute a task through the integrated system
    pub async fn execute_task(
        &self,
        task: &TaskConfig,
        context: &AgentContext,
    ) -> AgentRuntimeResult<AgentTaskResult> {
        self.tool_executor.execute_task(task, context).await
    }
}