//! Integration example for Toka Analysis Tools
//!
//! This example demonstrates how to register and use the Python analysis tools
//! with the Toka system in a secure, production-ready manner.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use toka_tools::{ToolRegistry, ToolParams};
use toka_analysis_tools::{AnalysisToolRegistry, AnalysisConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Toka Analysis Tools Integration Example");

    // 1. Create the main Toka tool registry
    let tool_registry = ToolRegistry::new().await?;
    println!("✅ Created main tool registry");

    // 2. Create analysis tools configuration
    let analysis_config = AnalysisConfig {
        python_path: std::path::PathBuf::from("python3"),
        tools_directory: std::path::PathBuf::from("toka_analysis_tools"),
        output_directory: std::path::PathBuf::from("target/analysis"),
        workspace_root: std::path::PathBuf::from("."),
        enable_cache: true,
        enable_metrics: true,
        ..Default::default()
    };

    // 3. Create and configure the analysis tool registry
    let analysis_registry = AnalysisToolRegistry::with_config(analysis_config).await?;
    println!("✅ Created analysis tool registry");

    // 4. Register all analysis tools with the main registry
    analysis_registry.register_all_tools(&tool_registry).await?;
    println!("✅ Registered analysis tools");

    // 5. List available tools
    let available_tools = tool_registry.list_tools().await;
    println!("📋 Available tools: {:?}", available_tools);

    // 6. Example: Execute control flow analysis
    println!("\n🔍 Running Control Flow Analysis Example");
    let mut args = HashMap::new();
    args.insert("target_function".to_string(), "main".to_string());
    args.insert("output_format".to_string(), "mermaid".to_string());
    args.insert("include_complexity".to_string(), "true".to_string());

    let params = ToolParams {
        name: "control-flow-analysis".to_string(),
        args,
    };

    match tool_registry.execute_tool("control-flow-analysis", &params).await {
        Ok(result) => {
            println!("✅ Control flow analysis completed successfully");
            println!("📊 Result: {}", result.output);
            println!("⏱️  Execution time: {}ms", result.metadata.execution_time_ms);
        }
        Err(e) => {
            println!("❌ Control flow analysis failed: {}", e);
            // In a real application, we might still proceed with other tools
        }
    }

    // 7. Example: Execute dependency analysis
    println!("\n🔗 Running Dependency Analysis Example");
    let mut args = HashMap::new();
    args.insert("workspace_root".to_string(), ".".to_string());
    args.insert("output_format".to_string(), "json".to_string());
    args.insert("include_agents".to_string(), "true".to_string());

    let params = ToolParams {
        name: "dependency-analysis".to_string(),
        args,
    };

    match tool_registry.execute_tool("dependency-analysis", &params).await {
        Ok(result) => {
            println!("✅ Dependency analysis completed successfully");
            println!("📊 Result: {}", result.output);
            println!("⏱️  Execution time: {}ms", result.metadata.execution_time_ms);
        }
        Err(e) => {
            println!("❌ Dependency analysis failed: {}", e);
        }
    }

    // 8. Example: Execute combined analysis
    println!("\n🔄 Running Combined Analysis Example");
    let mut args = HashMap::new();
    args.insert("output_format".to_string(), "json".to_string());
    args.insert("include_mermaid".to_string(), "true".to_string());

    let params = ToolParams {
        name: "combined-analysis".to_string(),
        args,
    };

    match tool_registry.execute_tool("combined-analysis", &params).await {
        Ok(result) => {
            println!("✅ Combined analysis completed successfully");
            println!("📊 Result length: {} bytes", result.output.len());
            println!("⏱️  Execution time: {}ms", result.metadata.execution_time_ms);
        }
        Err(e) => {
            println!("❌ Combined analysis failed: {}", e);
        }
    }

    // 9. Show metrics and cache statistics
    println!("\n📈 Analysis Metrics and Statistics");
    let metrics = analysis_registry.get_metrics();
    let all_metrics = metrics.get_metrics().await;
    println!("📊 Total executions: {}", all_metrics.len());

    let cache_stats = analysis_registry.get_cache_stats().await;
    println!("🗃️  Cache stats - Hits: {}, Misses: {}, Entries: {}", 
             cache_stats.hits, cache_stats.misses, cache_stats.entries);

    println!("\n🎉 Integration example completed successfully!");

    Ok(())
}

/// Example function showing how to integrate with agent runtime
#[allow(dead_code)]
async fn agent_integration_example() -> Result<()> {
    use toka_agent_runtime::CapabilityValidator;
    use toka_types::SecurityConfig;

    // This shows how an agent would use the analysis tools
    println!("🤖 Agent Integration Example");

    // 1. Create security configuration for agent
    let security_config = SecurityConfig {
        sandbox: true,
        capabilities_required: vec![
            "filesystem-read".to_string(),
            "filesystem-write".to_string(),
            "process-spawn".to_string(),
        ],
        resource_limits: toka_types::ResourceLimits {
            max_memory: "512MB".to_string(),
            max_cpu: "50%".to_string(),
            timeout: "10m".to_string(),
        },
    };

    // 2. Create capability validator
    let validator = CapabilityValidator::new(
        security_config.capabilities_required.clone(),
        security_config,
    );

    // 3. Check if agent can use analysis tools
    if validator.can_perform("filesystem-read").unwrap_or(false) &&
       validator.can_perform("process-spawn").unwrap_or(false) {
        println!("✅ Agent has required capabilities for analysis tools");
        
        // Agent can safely use the analysis tools
        // (Tool execution would happen here)
        
    } else {
        println!("❌ Agent lacks required capabilities");
    }

    Ok(())
}

/// Example function showing custom tool configuration
#[allow(dead_code)]
async fn custom_configuration_example() -> Result<()> {
    use std::time::Duration;
    use toka_analysis_tools::{AnalysisConfig, ResourceLimits, CacheConfig};

    println!("⚙️  Custom Configuration Example");

    // Create custom configuration with tighter security
    let custom_config = AnalysisConfig {
        python_path: std::path::PathBuf::from("/usr/bin/python3"),
        tools_directory: std::path::PathBuf::from("/opt/toka/analysis-tools"),
        output_directory: std::path::PathBuf::from("/tmp/toka-analysis"),
        workspace_root: std::path::PathBuf::from("/workspace"),
        
        // Tighter resource limits
        resource_limits: ResourceLimits {
            max_memory_mb: 256,  // 256MB limit
            max_cpu_percent: 25.0,  // 25% CPU limit
            max_execution_time: Duration::from_secs(300),  // 5 minute limit
            max_output_size: 5 * 1024 * 1024,  // 5MB output limit
            max_output_files: 50,
            max_disk_mb: 512,
        },
        
        // Custom cache configuration
        cache_config: CacheConfig {
            enabled: true,
            max_size: 500,  // Smaller cache
            ttl: Duration::from_secs(1800),  // 30 minute TTL
        },
        
        timeout: Duration::from_secs(300),
        enable_cache: true,
        enable_metrics: true,
        
        // More restrictive security
        security: toka_types::SecurityConfig {
            sandbox: true,
            capabilities_required: vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
                "process-spawn".to_string(),
            ],
            resource_limits: toka_types::ResourceLimits {
                max_memory: "256MB".to_string(),
                max_cpu: "25%".to_string(),
                timeout: "5m".to_string(),
            },
        },
    };

    // Create registry with custom configuration
    let analysis_registry = AnalysisToolRegistry::with_config(custom_config).await?;
    println!("✅ Created analysis registry with custom configuration");

    // This configuration would be suitable for production environments
    // with strict resource constraints

    Ok(())
}