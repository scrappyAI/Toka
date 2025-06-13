use crate::runtime::{Runtime, RuntimeConfig};
use crate::agents::SymbolicAgent;
use crate::tools::{ToolRegistry, ToolParams};
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

/// Runtime instance shared across commands
static RUNTIME: Mutex<Option<Arc<Runtime>>> = Mutex::const_new(None);

/// Tool registry instance shared across commands
static TOOL_REGISTRY: Mutex<Option<Arc<ToolRegistry>>> = Mutex::const_new(None);

/// Initialize the runtime if not already initialized
async fn ensure_runtime(config: Option<PathBuf>) -> Result<Arc<Runtime>> {
    let mut runtime_guard = RUNTIME.lock().await;
    
    if runtime_guard.is_none() {
        let config = RuntimeConfig {
            vault_path: config
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "runtime_data".to_string()),
            max_agents: 100,
            event_buffer_size: 1000,
        };
        
        let runtime = Runtime::new(config).await
            .context("Failed to initialize runtime")?;
            
        *runtime_guard = Some(Arc::new(runtime));
    }
    
    Ok(runtime_guard.as_ref().unwrap().clone())
}

/// Initialize the tool registry if not already initialized
async fn ensure_tool_registry() -> Result<Arc<ToolRegistry>> {
    let mut registry_guard = TOOL_REGISTRY.lock().await;
    
    if registry_guard.is_none() {
        let registry = ToolRegistry::new();
            
        *registry_guard = Some(Arc::new(registry));
    }
    
    Ok(registry_guard.as_ref().unwrap().clone())
}

/// Start the runtime daemon
pub async fn start_runtime(config: Option<PathBuf>) -> Result<()> {
    let runtime = ensure_runtime(config).await?;
    runtime.start().await?;
    println!("Runtime started successfully");
    Ok(())
}

/// Create a new agent
pub async fn create_agent(agent_type: String, _config: Option<String>) -> Result<()> {
    let runtime = ensure_runtime(None).await?;
    
    // For now, we only support symbolic agents
    if agent_type != "symbolic" {
        return Err(anyhow::anyhow!("Unsupported agent type: {}", agent_type));
    }
    
    let agent = Box::new(SymbolicAgent::new(&format!("agent_{}", uuid::Uuid::new_v4())));
    let agent_id = runtime.register_agent(agent).await?;
    
    // Save the state after registering the new agent
    runtime.save_state().await?;

    println!("Created new agent with ID: {}", agent_id);
    Ok(())
}

/// List all registered agents
pub async fn list_agents() -> Result<()> {
    let runtime = ensure_runtime(None).await?;
    let agents = runtime.list_agents().await;
    
    if agents.is_empty() {
        println!("No agents registered");
    } else {
        println!("Registered agents:");
        for agent_id in agents {
            println!(" - {}", agent_id);
        }
    }
    
    Ok(())
}

/// Control an agent's execution
pub async fn control_agent(agent_id: String, command: String, data: Option<String>) -> Result<()> {
    let _runtime = ensure_runtime(None).await?;
    
    match command.as_str() {
        "pause" => {
            // TODO: Implement agent pause
            println!("Pausing agent: {}", agent_id);
        }
        "resume" => {
            // TODO: Implement agent resume
            println!("Resuming agent: {}", agent_id);
        }
        "set-goals" => {
            if let Some(goals) = data {
                // TODO: Implement goal setting
                println!("Setting goals for agent {}: {}", agent_id, goals);
            } else {
                return Err(anyhow::anyhow!("Goals data required for set-goals command"));
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown command: {}", command));
        }
    }
    
    Ok(())
}

/// Emit an event into the system
pub async fn emit_event(event_type: String, data: String) -> Result<()> {
    let runtime = ensure_runtime(None).await?;
    runtime.emit_event(event_type.clone(), data.clone()).await?;
    println!("Emitted event of type: {}", event_type);
    Ok(())
}

/// Run a tool with the given arguments
pub async fn run_tool(tool_name: String, args: Option<Vec<String>>) -> Result<()> {
    let registry = ensure_tool_registry().await?;
    
    // Parse arguments from key=value format
    let mut tool_args = HashMap::new();
    if let Some(args) = args {
        for arg in args {
            if let Some((key, value)) = arg.split_once('=') {
                tool_args.insert(key.to_string(), value.to_string());
            } else {
                return Err(anyhow::anyhow!("Invalid argument format: '{}'. Use key=value format.", arg));
            }
        }
    }
    
    // Special handling for reporting tool with sample data
    if tool_name == "reporting" && tool_args.is_empty() {
        let sample_data = r#"{
            "transactions": [
                {
                    "date": "2024-01-01",
                    "amount": 5000.00,
                    "category": "Salary",
                    "description": "Monthly salary"
                },
                {
                    "date": "2024-01-05",
                    "amount": -1200.00,
                    "category": "Rent",
                    "description": "Monthly rent"
                },
                {
                    "date": "2024-01-10",
                    "amount": -300.00,
                    "category": "Groceries",
                    "description": "Weekly groceries"
                },
                {
                    "date": "2024-01-15",
                    "amount": -80.00,
                    "category": "Utilities",
                    "description": "Electricity bill"
                }
            ],
            "metadata": {
                "period_start": "2024-01-01",
                "period_end": "2024-01-31",
                "currency": "USD"
            }
        }"#;
        tool_args.insert("data".to_string(), sample_data.to_string());
    }
    
    let params = ToolParams {
        name: tool_name.clone(),
        args: tool_args,
    };
    
    let result = registry.execute_tool(&tool_name, &params).await?;
    
    if result.success {
        println!("✅ Tool '{}' executed successfully:", tool_name);
        println!("{}", result.output);
        println!("\n📊 Execution metadata:");
        println!("  - Execution time: {}ms", result.metadata.execution_time_ms);
        println!("  - Tool version: {}", result.metadata.tool_version);
    } else {
        println!("❌ Tool '{}' failed:", tool_name);
        println!("{}", result.output);
    }
    
    Ok(())
}

/// List all available tools
pub async fn list_tools() -> Result<()> {
    let registry = ensure_tool_registry().await?;
    let tools = registry.list_tools().await;
    
    if tools.is_empty() {
        println!("No tools available");
    } else {
        println!("Available tools:");
        for tool_name in tools {
            if let Some(tool) = registry.get_tool(&tool_name).await {
                println!("  📧 {} - {}", tool_name, tool.description());
            }
        }
        println!("\nUsage: toka run <tool> --args key=value,key2=value2");
        println!("Example: toka run reporting (uses sample data)");
    }
    
    Ok(())
} 