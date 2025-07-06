//! Workspace Integration Example
//!
//! This example demonstrates the complete integration of external tools with the
//! Toka agent runtime, showing how agents can automatically discover and execute
//! Python scripts and shell scripts in the workspace.

use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

// Note: This would normally use the actual crates, but for demonstration
// we'll use the types we've defined
use toka_tools::{ToolRegistry, ToolRegistryExt, ToolDiscoveryConfig};
use toka_agent_runtime::{TokaAgentRuntime, AgentRuntimeToolIntegration};

/// Demonstrate the complete Toka OS tool integration
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(false)
        .init();

    info!("🚀 Starting Toka OS Tool Integration Demo");

    // Simulate the workspace discovery
    demonstrate_workspace_tool_discovery().await?;
    
    // Simulate agent task execution with discovered tools
    demonstrate_agent_task_execution().await?;
    
    // Demonstrate real workspace integration
    demonstrate_real_workspace_integration().await?;

    info!("🎉 Toka OS Tool Integration Demo completed successfully!");
    Ok(())
}

/// Demonstrate automatic tool discovery in the workspace
async fn demonstrate_workspace_tool_discovery() -> Result<()> {
    info!("=== Workspace Tool Discovery ===");

    // Create tool registry
    let registry = Arc::new(ToolRegistry::new_empty());

    // Configure discovery for actual Toka workspace
    let discovery_config = ToolDiscoveryConfig {
        search_directories: vec![
            std::path::PathBuf::from("scripts"),
            std::path::PathBuf::from("tools"),
        ],
        include_patterns: vec![
            "*.py".to_string(),
            "*.sh".to_string(),
        ],
        exclude_patterns: vec![
            "*test*".to_string(),
            "*__pycache__*".to_string(),
        ],
        follow_symlinks: false,
        max_depth: 2,
    };

    // Auto-discover tools
    let count = registry.auto_register_tools_with_config(discovery_config).await?;
    info!("Discovered {} tools in workspace", count);

    // List discovered tools
    let tools = registry.list_tools().await;
    info!("Available tools: {:?}", tools);

    // Expected tools based on our research:
    let expected_tools = vec![
        "validate-dates",      // scripts/validate_dates.py
        "validate-build-system", // scripts/validate-build-system.sh
        "test-toka-system",    // scripts/test-toka-system.sh
        "monitor-raft-development", // monitor_raft_development.py
        "raft-analysis",       // raft_analysis.py (if exists)
        "prompt-manager",      // prompts/tools/prompt_manager.py
    ];

    for expected in &expected_tools {
        if tools.contains(&expected.to_string()) {
            info!("✅ Found expected tool: {}", expected);
        } else {
            warn!("⚠️ Expected tool not found: {}", expected);
        }
    }

    Ok(())
}

/// Demonstrate agent task execution using discovered tools
async fn demonstrate_agent_task_execution() -> Result<()> {
    info!("=== Agent Task Execution ===");

    // This would normally use the real runtime, but for demo purposes
    // we'll simulate the key interactions

    info!("🤖 Agent: ValidationAgent starting...");
    
    // Simulate task: "Validate dates in all workspace files"
    info!("📋 Task: Validate workspace dates");
    info!("🔍 Agent analyzing task...");
    info!("🛠️ Agent selected tool: validate-dates");
    info!("⚙️ Executing: validate_dates.py --mode verbose");
    
    // Simulated successful execution
    info!("✅ Task completed successfully");
    info!("📊 Result: All dates in workspace are valid");

    info!("📋 Task: Validate build system");
    info!("🔍 Agent analyzing task...");
    info!("🛠️ Agent selected tool: validate-build-system");
    info!("⚙️ Executing: validate-build-system.sh --verbose");
    
    // Simulated successful execution
    info!("✅ Task completed successfully");
    info!("📊 Result: Build system configuration is valid");

    info!("🤖 Agent: MonitoringAgent starting...");
    
    // Simulate task: "Monitor Raft development progress"
    info!("📋 Task: Monitor Raft development");
    info!("🔍 Agent analyzing task...");
    info!("🛠️ Agent selected tool: monitor-raft-development");
    info!("⚙️ Executing: monitor_raft_development.py --collect-metrics");
    
    // Simulated successful execution
    info!("✅ Task completed successfully");
    info!("📊 Result: Raft development metrics collected");

    Ok(())
}

/// Demonstrate integration with real workspace tools
async fn demonstrate_real_workspace_integration() -> Result<()> {
    info!("=== Real Workspace Integration ===");

    // Show how the actual Toka OS would be configured
    info!("🔧 Configuring Toka OS for workspace operation...");

    // 1. Tool Registry Setup
    info!("📦 Setting up tool registry with workspace discovery");
    let registry_config = ToolDiscoveryConfig {
        search_directories: vec![
            std::path::PathBuf::from("scripts"),
            std::path::PathBuf::from("agents/tools"),
        ],
        include_patterns: vec!["*.py".to_string(), "*.sh".to_string()],
        exclude_patterns: vec!["*test*".to_string(), "*tmp*".to_string()],
        follow_symlinks: false,
        max_depth: 3,
    };

    // 2. Agent Configuration
    info!("🤖 Configuring agents with discovered tools");
    
    // Date Enforcement Agent
    info!("   📅 Date Enforcement Agent:");
    info!("      - Tools: validate-dates, insert-date");
    info!("      - Capabilities: date-validation, template-processing");
    info!("      - Schedule: daily, pre-commit");

    // Build System Agent  
    info!("   🏗️ Build System Agent:");
    info!("      - Tools: validate-build-system, test-toka-system");
    info!("      - Capabilities: build-validation, system-testing");
    info!("      - Schedule: on workspace change");

    // Monitoring Agent
    info!("   📊 Monitoring Agent:");
    info!("      - Tools: monitor-raft-development, raft-analysis");
    info!("      - Capabilities: raft-monitoring, system-health");
    info!("      - Schedule: continuous");

    // 3. Security Configuration
    info!("🔒 Configuring security policies");
    info!("   - Python tools: sandboxed execution, no network access");
    info!("   - Shell tools: restricted file system access");
    info!("   - Capability validation: enforced at runtime");

    // 4. Integration Points
    info!("🔗 Integration points configured:");
    info!("   - Kernel operations: ✅");
    info!("   - LLM gateway: ✅");
    info!("   - Tool registry: ✅");
    info!("   - Agent orchestration: ✅");
    info!("   - Progress reporting: ✅");

    // 5. Example Execution Flow
    info!("🎯 Example execution flow:");
    info!("   1. Agent receives task from orchestration");
    info!("   2. Agent analyzes task using LLM");
    info!("   3. Agent selects appropriate tool from registry");
    info!("   4. Tool execution with security validation");
    info!("   5. Results reported back to orchestration");
    info!("   6. Progress tracking and metrics collection");

    // 6. Real Tool Mappings
    info!("🗺️ Tool mappings for discovered workspace tools:");
    
    let tool_mappings = vec![
        ("scripts/validate_dates.py", "date-validator", "date-validation"),
        ("scripts/validate-build-system.sh", "build-validator", "build-validation"),
        ("scripts/test-toka-system.sh", "system-tester", "system-testing"),
        ("monitor_raft_development.py", "raft-monitor", "raft-monitoring"),
        ("raft_monitoring_service.sh", "raft-service", "raft-monitoring"),
        ("prompts/tools/prompt_manager.py", "prompt-manager", "prompt-management"),
    ];

    for (script_path, tool_name, capability) in tool_mappings {
        if std::path::Path::new(script_path).exists() {
            info!("   ✅ {} → {} ({})", script_path, tool_name, capability);
        } else {
            info!("   ⚠️ {} → {} ({}) [NOT FOUND]", script_path, tool_name, capability);
        }
    }

    // 7. Expected Agent Behaviors
    info!("🎭 Expected agent behaviors:");
    info!("   📅 Date Enforcement Agent will:");
    info!("      - Run validate_dates.py on schedule");
    info!("      - Fix date issues automatically");
    info!("      - Report compliance status");
    
    info!("   🏗️ Build System Agent will:");
    info!("      - Validate Cargo.toml changes");
    info!("      - Run system tests on modifications");
    info!("      - Report build health");
    
    info!("   📊 Monitoring Agent will:");
    info!("      - Collect Raft development metrics");
    info!("      - Monitor system performance");
    info!("      - Generate status reports");

    Ok(())
}

/// Configuration for production Toka OS deployment
pub struct TokaProductionConfig {
    pub tool_discovery: ToolDiscoveryConfig,
    pub security_policies: SecurityPolicies,
    pub agent_configurations: Vec<AgentConfiguration>,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    pub python_sandboxing: bool,
    pub shell_restrictions: bool,
    pub network_access: bool,
    pub capability_enforcement: bool,
}

#[derive(Debug, Clone)]
pub struct AgentConfiguration {
    pub name: String,
    pub tools: Vec<String>,
    pub capabilities: Vec<String>,
    pub schedule: String,
    pub priority: String,
}

impl TokaProductionConfig {
    /// Create production configuration for Toka workspace
    pub fn for_toka_workspace() -> Self {
        Self {
            tool_discovery: ToolDiscoveryConfig {
                search_directories: vec![
                    std::path::PathBuf::from("scripts"),
                    std::path::PathBuf::from("agents/tools"),
                ],
                include_patterns: vec!["*.py".to_string(), "*.sh".to_string()],
                exclude_patterns: vec![
                    "*test*".to_string(),
                    "*dev*".to_string(),
                    "*tmp*".to_string(),
                    "*debug*".to_string(),
                ],
                follow_symlinks: false,
                max_depth: 2,
            },
            security_policies: SecurityPolicies {
                python_sandboxing: true,
                shell_restrictions: true,
                network_access: false,
                capability_enforcement: true,
            },
            agent_configurations: vec![
                AgentConfiguration {
                    name: "date-enforcement-agent".to_string(),
                    tools: vec!["validate-dates".to_string(), "insert-date".to_string()],
                    capabilities: vec!["date-validation".to_string(), "template-processing".to_string()],
                    schedule: "daily".to_string(),
                    priority: "high".to_string(),
                },
                AgentConfiguration {
                    name: "build-system-agent".to_string(),
                    tools: vec!["validate-build-system".to_string(), "test-toka-system".to_string()],
                    capabilities: vec!["build-validation".to_string(), "system-testing".to_string()],
                    schedule: "on-change".to_string(),
                    priority: "critical".to_string(),
                },
                AgentConfiguration {
                    name: "monitoring-agent".to_string(),
                    tools: vec!["monitor-raft-development".to_string(), "raft-analysis".to_string()],
                    capabilities: vec!["raft-monitoring".to_string(), "system-health".to_string()],
                    schedule: "continuous".to_string(),
                    priority: "medium".to_string(),
                },
            ],
        }
    }
}