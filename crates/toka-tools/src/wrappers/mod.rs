//! Tool wrappers for external executables with unified security model
//!
//! This module provides secure wrappers for external tools including Python scripts,
//! shell scripts, and other executables. It combines the flexibility of generic
//! external tool integration with the security and analysis capabilities needed
//! for the Toka agent OS.

pub mod external;
pub mod python;
pub mod shell;
pub mod discovery;
pub mod security;

// Re-export common types
pub use external::{ExternalTool, ExternalToolBuilder};
pub use python::{PythonTool, PythonConfig, PythonToolBuilder};
pub use shell::{ShellTool, ShellConfig, ShellToolBuilder};
pub use discovery::{ToolDiscovery, DiscoveryConfig, ToolDiscoveryBuilder};
pub use security::{
    SecurityConfig, SandboxConfig, CapabilityValidator, 
    ResourceLimits, SecurityLevel, ToolSecurityClassification
};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::{Tool, ToolParams, ToolResult};
use crate::manifest::ToolManifest;

/// Unified tool registry that combines all tool types with consistent security
pub struct UnifiedToolRegistry {
    /// Main tool registry
    registry: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    /// Security validator
    security_validator: Arc<CapabilityValidator>,
    /// Discovery system
    discovery: Arc<ToolDiscovery>,
    /// Security classifications
    security_classifications: Arc<RwLock<HashMap<String, ToolSecurityClassification>>>,
    /// Execution metrics
    execution_metrics: Arc<RwLock<HashMap<String, ToolExecutionMetrics>>>,
}

impl UnifiedToolRegistry {
    /// Create a new unified tool registry
    pub async fn new() -> Result<Self> {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let security_validator = Arc::new(CapabilityValidator::new());
        let discovery = Arc::new(ToolDiscovery::new_with_security_defaults());
        let security_classifications = Arc::new(RwLock::new(HashMap::new()));
        let execution_metrics = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            registry,
            security_validator,
            discovery,
            security_classifications,
            execution_metrics,
        })
    }
    
    /// Auto-discover and register all tools with appropriate security levels
    pub async fn auto_register_tools(&self) -> Result<usize> {
        let discovered_tools = self.discovery.discover_all_tools().await?;
        let mut count = 0;
        
        for tool_spec in discovered_tools {
            let security_level = self.classify_tool_security(&tool_spec);
            self.register_tool_with_security(tool_spec, security_level).await?;
            count += 1;
        }
        
        Ok(count)
    }
    
    /// Register a tool with specific security classification
    pub async fn register_tool_with_security(
        &self,
        tool_spec: DiscoveredTool,
        security_level: SecurityLevel,
    ) -> Result<()> {
        // Create appropriate wrapper based on tool type
        let capabilities_clone = tool_spec.capabilities.clone();
        let tool: Arc<dyn Tool> = match tool_spec.tool_type {
            ToolType::Python => {
                let python_tool = PythonTool::new_with_security(
                    tool_spec.path,
                    &tool_spec.name,
                    &tool_spec.description,
                    tool_spec.capabilities.clone(),
                    security_level,
                ).await?;
                Arc::new(python_tool)
            },
            ToolType::Shell => {
                let shell_tool = ShellTool::new_with_security(
                    tool_spec.path,
                    &tool_spec.name,
                    &tool_spec.description,
                    tool_spec.capabilities.clone(),
                    security_level,
                ).await?;
                Arc::new(shell_tool)
            },
            ToolType::External => {
                let external_tool = ExternalTool::new_with_security(
                    tool_spec.path,
                    &tool_spec.name,
                    &tool_spec.description,
                    tool_spec.capabilities.clone(),
                    security_level,
                ).await?;
                Arc::new(external_tool)
            },
        };
        
        // Register the tool
        let tool_name = tool.name().to_string();
        self.registry.write().await.insert(tool_name.clone(), tool);
        
        // Store security classification
        let classification = ToolSecurityClassification {
            security_level,
            capabilities: capabilities_clone,
            resource_limits: security_level.default_resource_limits(),
            sandbox_config: security_level.default_sandbox_config(),
        };
        self.security_classifications.write().await.insert(tool_name, classification);
        
        Ok(())
    }
    
    /// Execute a tool with security validation
    pub async fn execute_tool_secure(
        &self,
        tool_name: &str,
        params: &ToolParams,
        agent_capabilities: &[String],
    ) -> Result<ToolResult> {
        // Get tool and security classification
        let tool = self.registry.read().await.get(tool_name).cloned()
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;
        
        let classification = self.security_classifications.read().await.get(tool_name).cloned()
            .ok_or_else(|| anyhow::anyhow!("Security classification not found for: {}", tool_name))?;
        
        // Validate capabilities
        self.security_validator.validate_tool_execution(
            &classification.capabilities,
            agent_capabilities,
        )?;
        
        // Execute with monitoring
        let start_time = std::time::Instant::now();
        let result = tool.execute(params).await;
        let execution_time = start_time.elapsed();
        
        // Record metrics
        let metrics = ToolExecutionMetrics {
            tool_name: tool_name.to_string(),
            execution_time,
            success: result.is_ok(),
            timestamp: std::time::SystemTime::now(),
        };
        self.execution_metrics.write().await.insert(tool_name.to_string(), metrics);
        
        result
    }
    
    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        self.registry.read().await.len()
    }
    
    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.registry.read().await.keys().cloned().collect()
    }
    
    /// Get tool security classification
    pub async fn get_tool_security(&self, tool_name: &str) -> Option<ToolSecurityClassification> {
        self.security_classifications.read().await.get(tool_name).cloned()
    }
    
    /// Get execution metrics
    pub async fn get_execution_metrics(&self, tool_name: &str) -> Option<ToolExecutionMetrics> {
        self.execution_metrics.read().await.get(tool_name).cloned()
    }
    
    /// Classify tool security level based on tool specification
    fn classify_tool_security(&self, tool_spec: &DiscoveredTool) -> SecurityLevel {
        // Analysis tools get high security
        if tool_spec.capabilities.iter().any(|cap| cap.contains("analysis") || cap.contains("visualization")) {
            return SecurityLevel::High;
        }
        
        // System tools get medium security
        if tool_spec.capabilities.iter().any(|cap| cap.contains("system") || cap.contains("build")) {
            return SecurityLevel::Medium;
        }
        
        // Utility tools get basic security
        SecurityLevel::Basic
    }
}

/// Discovered tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub capabilities: Vec<String>,
    pub tool_type: ToolType,
}

/// Tool type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Python,
    Shell,
    External,
}

/// Tool execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionMetrics {
    pub tool_name: String,
    pub execution_time: std::time::Duration,
    pub success: bool,
    pub timestamp: std::time::SystemTime,
}



#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_unified_registry_creation() -> Result<()> {
        let registry = UnifiedToolRegistry::new().await?;
        assert_eq!(registry.tool_count().await, 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_tool_registration() -> Result<()> {
        let registry = UnifiedToolRegistry::new().await?;
        
        // Create a test Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "print('Test tool')")?;
        
        let tool_spec = DiscoveredTool {
            name: "test-tool".to_string(),
            path: script_file.path().to_path_buf(),
            description: "Test tool".to_string(),
            capabilities: vec!["testing".to_string()],
            tool_type: ToolType::Python,
        };
        
        registry.register_tool_with_security(tool_spec, SecurityLevel::Basic).await?;
        
        assert_eq!(registry.tool_count().await, 1);
        let tools = registry.list_tools().await;
        assert!(tools.contains(&"test-tool".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_security_classification() -> Result<()> {
        let registry = UnifiedToolRegistry::new().await?;
        
        // Create analysis tool
        let analysis_tool = DiscoveredTool {
            name: "analyzer".to_string(),
            path: PathBuf::from("analyzer.py"),
            description: "Analysis tool".to_string(),
            capabilities: vec!["code-analysis".to_string()],
            tool_type: ToolType::Python,
        };
        
        let security_level = registry.classify_tool_security(&analysis_tool);
        assert_eq!(security_level, SecurityLevel::High);
        
        // Create utility tool
        let utility_tool = DiscoveredTool {
            name: "utility".to_string(),
            path: PathBuf::from("utility.sh"),
            description: "Utility tool".to_string(),
            capabilities: vec!["file-processing".to_string()],
            tool_type: ToolType::Shell,
        };
        
        let security_level = registry.classify_tool_security(&utility_tool);
        assert_eq!(security_level, SecurityLevel::Basic);
        
        Ok(())
    }
}