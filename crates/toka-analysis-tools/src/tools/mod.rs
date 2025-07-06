//! Individual analysis tool implementations

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;

use toka_tools::{ToolParams, ToolResult};

use crate::{
    AnalysisTool, AnalysisContext, PythonExecutor, ResultCache, MetricsCollector
};

/// Control flow analysis tool
pub struct ControlFlowAnalysisTool {
    executor: Arc<PythonExecutor>,
    cache: Arc<RwLock<ResultCache>>,
    metrics: Arc<MetricsCollector>,
}

impl ControlFlowAnalysisTool {
    /// Create a new control flow analysis tool
    pub fn new(
        executor: Arc<PythonExecutor>,
        cache: Arc<RwLock<ResultCache>>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self { executor, cache, metrics }
    }
}

#[async_trait]
impl AnalysisTool for ControlFlowAnalysisTool {
    fn name(&self) -> &str {
        "control-flow-analysis"
    }
    
    fn description(&self) -> &str {
        "Analyzes function control flow patterns and complexity metrics"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        vec![
            "filesystem-read".to_string(),
            "filesystem-write".to_string(),
            "process-spawn".to_string(),
        ]
    }
    
    fn validate_input(&self, params: &ToolParams) -> Result<()> {
        // Validate required parameters
        if !params.args.contains_key("target_function") && !params.args.contains_key("target_file") {
            return Err(anyhow::anyhow!("Either 'target_function' or 'target_file' must be specified"));
        }
        Ok(())
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        // Create execution context
        let context = AnalysisContext::new(
            self.name().to_string(),
            params.clone(),
            &crate::AnalysisConfig::default(), // Would get from executor
        );
        
        // Execute the tool
        let result = self.executor.execute_tool(&context).await?;
        
        // Convert to ToolResult
        Ok(self.executor.to_tool_result(result))
    }
    
    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "function_analysis": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "complexity": {"type": "number"},
                        "control_flow": {"type": "object"},
                        "mermaid_diagram": {"type": "string"}
                    }
                },
                "summary": {
                    "type": "object",
                    "properties": {
                        "total_functions": {"type": "number"},
                        "average_complexity": {"type": "number"}
                    }
                }
            }
        })
    }
}

/// Dependency analysis tool
pub struct DependencyAnalysisTool {
    executor: Arc<PythonExecutor>,
    cache: Arc<RwLock<ResultCache>>,
    metrics: Arc<MetricsCollector>,
}

impl DependencyAnalysisTool {
    /// Create a new dependency analysis tool
    pub fn new(
        executor: Arc<PythonExecutor>,
        cache: Arc<RwLock<ResultCache>>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self { executor, cache, metrics }
    }
}

#[async_trait]
impl AnalysisTool for DependencyAnalysisTool {
    fn name(&self) -> &str {
        "dependency-analysis"
    }
    
    fn description(&self) -> &str {
        "Analyzes crate dependencies and workspace architecture"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        vec![
            "filesystem-read".to_string(),
            "filesystem-write".to_string(),
            "process-spawn".to_string(),
        ]
    }
    
    fn validate_input(&self, params: &ToolParams) -> Result<()> {
        // Basic validation - workspace root should be provided
        if let Some(workspace_root) = params.args.get("workspace_root") {
            if workspace_root.is_empty() {
                return Err(anyhow::anyhow!("workspace_root cannot be empty"));
            }
        }
        Ok(())
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let context = AnalysisContext::new(
            self.name().to_string(),
            params.clone(),
            &crate::AnalysisConfig::default(),
        );
        
        let result = self.executor.execute_tool(&context).await?;
        Ok(self.executor.to_tool_result(result))
    }
    
    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "crates": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "version": {"type": "string"},
                            "dependencies": {"type": "array"}
                        }
                    }
                },
                "dependency_graph": {"type": "string"},
                "mermaid_diagram": {"type": "string"}
            }
        })
    }
}

/// Combined analysis tool
pub struct CombinedAnalysisTool {
    executor: Arc<PythonExecutor>,
    cache: Arc<RwLock<ResultCache>>,
    metrics: Arc<MetricsCollector>,
}

impl CombinedAnalysisTool {
    /// Create a new combined analysis tool
    pub fn new(
        executor: Arc<PythonExecutor>,
        cache: Arc<RwLock<ResultCache>>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self { executor, cache, metrics }
    }
}

#[async_trait]
impl AnalysisTool for CombinedAnalysisTool {
    fn name(&self) -> &str {
        "combined-analysis"
    }
    
    fn description(&self) -> &str {
        "Runs multiple analysis tools together for comprehensive code analysis"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn required_capabilities(&self) -> Vec<String> {
        vec![
            "filesystem-read".to_string(),
            "filesystem-write".to_string(),
            "process-spawn".to_string(),
        ]
    }
    
    fn validate_input(&self, _params: &ToolParams) -> Result<()> {
        // Combined analysis doesn't require specific parameters
        Ok(())
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let context = AnalysisContext::new(
            self.name().to_string(),
            params.clone(),
            &crate::AnalysisConfig::default(),
        );
        
        let result = self.executor.execute_tool(&context).await?;
        Ok(self.executor.to_tool_result(result))
    }
    
    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "control_flow_analysis": {"type": "object"},
                "dependency_analysis": {"type": "object"},
                "summary": {
                    "type": "object",
                    "properties": {
                        "analyses_completed": {"type": "number"},
                        "total_execution_time": {"type": "string"}
                    }
                }
            }
        })
    }
}