//! Example: Agent Tool Integration
//!
//! This example demonstrates how to integrate external tools with the Toka agent runtime,
//! showing the complete flow from tool discovery to agent execution.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tempfile::TempDir;
use tokio::fs;
use tracing::{info, warn};

use toka_tools::{
    ToolRegistry, ToolParams, ToolRegistryExt, ToolDiscoveryConfig,
};

#[cfg(feature = "external-tools")]
use toka_tools::wrappers::{PythonTool, ShellTool};

/// Simulated agent task that uses external tools
#[derive(Debug, Clone)]
pub struct AgentTask {
    pub name: String,
    pub tool_name: String,
    pub parameters: HashMap<String, String>,
    pub required_capabilities: Vec<String>,
}

/// Simulated agent that executes tasks using the tool registry
pub struct Agent {
    pub name: String,
    pub tool_registry: Arc<ToolRegistry>,
    pub capabilities: Vec<String>,
}

impl Agent {
    /// Create a new agent with a tool registry
    pub fn new(name: String, tool_registry: Arc<ToolRegistry>, capabilities: Vec<String>) -> Self {
        Self {
            name,
            tool_registry,
            capabilities,
        }
    }
    
    /// Execute a task using the tool registry
    pub async fn execute_task(&self, task: &AgentTask) -> Result<String> {
        info!("Agent {} executing task: {}", self.name, task.name);
        
        // Validate that agent has required capabilities
        for required_cap in &task.required_capabilities {
            if !self.capabilities.contains(required_cap) {
                return Err(anyhow::anyhow!(
                    "Agent {} lacks required capability: {}", 
                    self.name, 
                    required_cap
                ));
            }
        }
        
        // Execute tool through registry
        let params = ToolParams {
            name: task.tool_name.clone(),
            args: task.parameters.clone(),
        };
        
        let result = self.tool_registry.execute_tool(&task.tool_name, &params).await?;
        
        if result.success {
            info!("Task {} completed successfully", task.name);
            Ok(result.output)
        } else {
            Err(anyhow::anyhow!("Task {} failed: {}", task.name, result.output))
        }
    }
    
    /// List available tools
    pub async fn list_available_tools(&self) -> Vec<String> {
        self.tool_registry.list_tools().await
    }
}

/// Setup example workspace with Python and shell tools
async fn setup_example_workspace(workspace_dir: &std::path::Path) -> Result<()> {
    let scripts_dir = workspace_dir.join("scripts");
    fs::create_dir_all(&scripts_dir).await?;
    
    // Create Python date validation tool
    let date_validator = scripts_dir.join("validate_dates.py");
    fs::write(&date_validator, r#"#!/usr/bin/env python3
"""
Date validation tool for workspace files
"""
import sys
import argparse
from datetime import datetime

def main():
    parser = argparse.ArgumentParser(description='Validate dates in files')
    parser.add_argument('--mode', choices=['verbose', 'quiet'], default='quiet')
    parser.add_argument('--fix', action='store_true', help='Fix date issues')
    args = parser.parse_args()
    
    current_date = datetime.now().strftime('%Y-%m-%d')
    
    if args.mode == 'verbose':
        print(f"Date validation started at {current_date}")
        print("Scanning workspace files...")
        print("✅ All dates are valid!")
    else:
        print("Date validation: PASSED")
    
    if args.fix:
        print("🔧 Fixed 0 date issues")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
"#).await?;
    
    // Create shell build validation tool
    let build_validator = scripts_dir.join("validate_build.sh");
    fs::write(&build_validator, r#"#!/bin/bash
# Build system validation tool
set -euo pipefail

MODE=${1:-"quiet"}

if [ "$MODE" = "verbose" ]; then
    echo "🔍 Validating build system configuration..."
    echo "📦 Checking Cargo.toml files..."
    echo "🔧 Checking dependencies..."
    echo "✅ Build system validation completed successfully!"
else
    echo "Build validation: PASSED"
fi

exit 0
"#).await?;
    
    // Make shell script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&build_validator).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&build_validator, perms).await?;
    }
    
    // Create monitoring script
    let monitor_tool = scripts_dir.join("system_monitor.py");
    fs::write(&monitor_tool, r#"#!/usr/bin/env python3
"""
System monitoring tool
"""
import sys
import json
import time

def main():
    # Simulate system monitoring
    metrics = {
        "timestamp": int(time.time()),
        "system_status": "healthy",
        "agents_active": 3,
        "tools_registered": 5,
        "memory_usage": "45%",
        "cpu_usage": "23%"
    }
    
    print(json.dumps(metrics, indent=2))
    return 0

if __name__ == '__main__':
    sys.exit(main())
"#).await?;
    
    Ok(())
}

/// Demonstrate manual tool registration
#[cfg(feature = "external-tools")]
async fn demonstrate_manual_registration(workspace_dir: &std::path::Path) -> Result<Arc<ToolRegistry>> {
    info!("=== Manual Tool Registration ===");
    
    let registry = Arc::new(ToolRegistry::new_empty());
    
    // Register Echo tool (built-in)
    registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
    
    // Manually register Python tool
    let date_validator = PythonTool::new(
        workspace_dir.join("scripts/validate_dates.py"),
        "date-validator",
        "Validates dates in workspace files",
        vec!["date-validation".to_string(), "validation".to_string()],
    )?;
    registry.register_tool(Arc::new(date_validator)).await?;
    
    // Manually register shell tool
    let build_validator = ShellTool::new(
        workspace_dir.join("scripts/validate_build.sh"),
        "build-validator", 
        "Validates build system configuration",
        vec!["build-validation".to_string(), "validation".to_string()],
    )?;
    registry.register_tool(Arc::new(build_validator)).await?;
    
    let tools = registry.list_tools().await;
    info!("Manually registered tools: {:?}", tools);
    
    Ok(registry)
}

/// Demonstrate auto-discovery registration
async fn demonstrate_auto_discovery(workspace_dir: &std::path::Path) -> Result<Arc<ToolRegistry>> {
    info!("=== Auto-Discovery Tool Registration ===");
    
    let registry = Arc::new(ToolRegistry::new_empty());
    
    // Register built-in tools first
    registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
    
    // Configure auto-discovery
    let discovery_config = ToolDiscoveryConfig {
        search_directories: vec![workspace_dir.join("scripts")],
        include_patterns: vec!["*.py".to_string(), "*.sh".to_string()],
        exclude_patterns: vec!["*test*".to_string()],
        follow_symlinks: false,
        max_depth: 2,
    };
    
    // Auto-discover and register tools
    let count = registry.auto_register_tools_with_config(discovery_config).await?;
    info!("Auto-discovered and registered {} tools", count);
    
    let tools = registry.list_tools().await;
    info!("All registered tools: {:?}", tools);
    
    Ok(registry)
}

/// Demonstrate agent task execution
async fn demonstrate_agent_execution(registry: Arc<ToolRegistry>) -> Result<()> {
    info!("=== Agent Task Execution ===");
    
    // Create agents with different capabilities
    let validation_agent = Agent::new(
        "ValidationAgent".to_string(),
        registry.clone(),
        vec![
            "date-validation".to_string(),
            "build-validation".to_string(),
            "validation".to_string(),
        ],
    );
    
    let monitoring_agent = Agent::new(
        "MonitoringAgent".to_string(),
        registry.clone(),
        vec![
            "monitoring".to_string(),
            "system-monitoring".to_string(),
        ],
    );
    
    // Create tasks for validation agent
    let tasks = vec![
        AgentTask {
            name: "validate-workspace-dates".to_string(),
            tool_name: "validate-dates".to_string(),
            parameters: [
                ("mode".to_string(), "verbose".to_string()),
                ("fix".to_string(), "true".to_string()),
            ].into_iter().collect(),
            required_capabilities: vec!["date-validation".to_string()],
        },
        AgentTask {
            name: "validate-build-system".to_string(),
            tool_name: "validate-build".to_string(),
            parameters: [
                ("mode".to_string(), "verbose".to_string()),
            ].into_iter().collect(),
            required_capabilities: vec!["build-validation".to_string()],
        },
    ];
    
    // Execute validation tasks
    for task in &tasks {
        match validation_agent.execute_task(task).await {
            Ok(output) => {
                info!("✅ Task '{}' completed:\n{}", task.name, output);
            }
            Err(e) => {
                warn!("❌ Task '{}' failed: {}", task.name, e);
            }
        }
    }
    
    // Execute monitoring task
    let monitor_task = AgentTask {
        name: "system-health-check".to_string(),
        tool_name: "system-monitor".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec!["monitoring".to_string()],
    };
    
    match monitoring_agent.execute_task(&monitor_task).await {
        Ok(output) => {
            info!("✅ Monitoring task completed:\n{}", output);
        }
        Err(e) => {
            warn!("❌ Monitoring task failed: {}", e);
        }
    }
    
    Ok(())
}

/// Demonstrate capability validation
async fn demonstrate_capability_validation(registry: Arc<ToolRegistry>) -> Result<()> {
    info!("=== Capability Validation ===");
    
    // Create agent with limited capabilities
    let limited_agent = Agent::new(
        "LimitedAgent".to_string(),
        registry.clone(),
        vec!["general".to_string()], // Only general capability
    );
    
    // Try to execute a task requiring specific capabilities
    let restricted_task = AgentTask {
        name: "restricted-validation".to_string(),
        tool_name: "date-validator".to_string(),
        parameters: HashMap::new(),
        required_capabilities: vec!["date-validation".to_string()], // Agent lacks this
    };
    
    match limited_agent.execute_task(&restricted_task).await {
        Ok(_) => warn!("⚠️ Capability validation failed - task should have been rejected"),
        Err(e) => info!("✅ Capability validation working: {}", e),
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(false)
        .init();
    
    info!("🚀 Starting Toka Agent Tool Integration Example");
    
    // Create temporary workspace
    let temp_workspace = TempDir::new()?;
    let workspace_path = temp_workspace.path();
    
    info!("📁 Setting up example workspace at: {}", workspace_path.display());
    setup_example_workspace(workspace_path).await?;
    
    // Demonstrate different registration approaches
    #[cfg(feature = "external-tools")]
    {
        // Manual registration
        let manual_registry = demonstrate_manual_registration(workspace_path).await?;
        
        // Auto-discovery registration  
        let auto_registry = demonstrate_auto_discovery(workspace_path).await?;
        
        // Use auto-discovery registry for agent demonstrations
        demonstrate_agent_execution(auto_registry.clone()).await?;
        demonstrate_capability_validation(auto_registry).await?;
    }
    
    #[cfg(not(feature = "external-tools"))]
    {
        warn!("External tools feature not enabled - limited demonstration");
        
        let registry = Arc::new(ToolRegistry::new_empty());
        registry.register_tool(Arc::new(toka_tools::tools::EchoTool::new())).await?;
        
        let echo_agent = Agent::new(
            "EchoAgent".to_string(),
            registry.clone(),
            vec!["general".to_string()],
        );
        
        let echo_task = AgentTask {
            name: "echo-test".to_string(),
            tool_name: "echo".to_string(),
            parameters: [("message".to_string(), "Hello from Toka!".to_string())].into_iter().collect(),
            required_capabilities: vec!["general".to_string()],
        };
        
        let output = echo_agent.execute_task(&echo_task).await?;
        info!("✅ Echo task completed: {}", output);
    }
    
    info!("🎉 Agent tool integration example completed successfully!");
    
    Ok(())
}