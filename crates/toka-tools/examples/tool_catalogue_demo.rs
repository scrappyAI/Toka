//! Tool Catalogue Demo
//!
//! This example demonstrates the tool catalogue system for efficient tool discovery,
//! filtering, and dynamic tool selection. It shows how to:
//!
//! 1. Scan the workspace for tools
//! 2. Filter tools by various criteria
//! 3. Export catalogue for LLM consumption
//! 4. Integrate with the agent runtime

use anyhow::Result;
use toka_tools::catalogue::{
    ToolCatalogue, ToolFilter, ToolCategory, SecurityLevel, SideEffect
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Tool Catalogue Demo");
    
    // Create tool catalogue
    let catalogue = ToolCatalogue::new();
    
    // Scan workspace for tools
    info!("Scanning workspace for tools...");
    let tool_count = catalogue.scan_workspace().await?;
    info!("Found {} tools in workspace", tool_count);
    
    // Demonstrate basic operations
    demo_basic_operations(&catalogue).await?;
    
    // Demonstrate filtering
    demo_filtering(&catalogue).await?;
    
    // Demonstrate export capabilities
    demo_export_capabilities(&catalogue).await?;
    
    // Demonstrate statistics
    demo_statistics(&catalogue).await?;
    
    info!("Tool Catalogue Demo completed successfully");
    Ok(())
}

async fn demo_basic_operations(catalogue: &ToolCatalogue) -> Result<()> {
    info!("=== Basic Operations Demo ===");
    
    // Get all tools
    let all_tools = catalogue.get_all_tools().await;
    info!("Total tools available: {}", all_tools.len());
    
    // List tool names
    for tool in &all_tools {
        info!("- {} (v{}) - {}", tool.name, tool.version, tool.description);
    }
    
    // Get specific tool
    if let Some(tool) = catalogue.get_tool("date-validator").await {
        info!("Found date-validator tool: {}", tool.description);
    } else {
        warn!("date-validator tool not found");
    }
    
    Ok(())
}

async fn demo_filtering(catalogue: &ToolCatalogue) -> Result<()> {
    info!("=== Filtering Demo ===");
    
    // Filter by category
    let validation_tools = catalogue.filter_tools(
        ToolFilter::new().with_category(ToolCategory::Validation)
    ).await?;
    info!("Validation tools: {}", validation_tools.len());
    
    // Filter by capability
    let filesystem_tools = catalogue.filter_tools(
        ToolFilter::new().with_capability("filesystem-read")
    ).await?;
    info!("Filesystem tools: {}", filesystem_tools.len());
    
    // Filter by security level
    let safe_tools = catalogue.filter_tools(
        ToolFilter::new().with_security_level(SecurityLevel::Basic)
    ).await?;
    info!("Basic security tools: {}", safe_tools.len());
    
    // Filter by multiple criteria
    let advanced_filter = ToolFilter::new()
        .with_categories(vec![ToolCategory::Validation, ToolCategory::Development])
        .with_capability("validation")
        .with_security_levels(vec![SecurityLevel::Basic, SecurityLevel::Medium])
        .with_search("date")
        .with_limit(5);
    
    let filtered_tools = catalogue.filter_tools(advanced_filter).await?;
    info!("Advanced filtered tools: {}", filtered_tools.len());
    
    for tool in &filtered_tools {
        info!("  - {}: {}", tool.name, tool.description);
    }
    
    Ok(())
}

async fn demo_export_capabilities(catalogue: &ToolCatalogue) -> Result<()> {
    info!("=== Export Capabilities Demo ===");
    
    // Export to JSON for LLM consumption
    let json_export = catalogue.export_json().await?;
    info!("JSON export size: {} bytes", json_export.len());
    
    // Export to YAML
    let yaml_export = catalogue.export_yaml().await?;
    info!("YAML export size: {} bytes", yaml_export.len());
    
    // Show a sample of the JSON export
    if json_export.len() > 200 {
        info!("JSON export sample: {}", &json_export[..200]);
    } else {
        info!("JSON export: {}", json_export);
    }
    
    Ok(())
}

async fn demo_statistics(catalogue: &ToolCatalogue) -> Result<()> {
    info!("=== Statistics Demo ===");
    
    let stats = catalogue.get_statistics().await;
    info!("Catalogue statistics:");
    info!("  Total tools: {}", stats.total_tools);
    
    info!("  Category distribution:");
    for (category, count) in &stats.category_distribution {
        info!("    {}: {}", category, count);
    }
    
    info!("  Capability distribution:");
    for (capability, count) in &stats.capability_distribution {
        info!("    {}: {}", capability, count);
    }
    
    info!("  Tag distribution:");
    for (tag, count) in &stats.tag_distribution {
        info!("    {}: {}", tag, count);
    }
    
    if let Some(last_scan) = stats.last_scan {
        info!("  Last scan: {}", last_scan);
    }
    
    Ok(())
}

// Example of integrating with agent runtime
async fn demo_agent_integration(catalogue: &ToolCatalogue) -> Result<()> {
    info!("=== Agent Integration Demo ===");
    
    // Simulate LLM tool selection
    let task_description = "Validate dates in all markdown files and fix any future dates";
    
    // Export catalogue for LLM
    let catalogue_json = catalogue.export_json().await?;
    
    // Simulate LLM response (in real scenario, this would come from LLM)
    let llm_selected_tools = vec!["date-validator", "file-reader"];
    
    info!("LLM selected tools for task '{}':", task_description);
    for tool_id in &llm_selected_tools {
        if let Some(tool) = catalogue.get_tool(tool_id).await {
            info!("  - {}: {}", tool.name, tool.description);
            
            // Show tool capabilities
            info!("    Required capabilities: {:?}", tool.required_capabilities);
            info!("    Security level: {:?}", tool.security_level);
            info!("    Category: {:?}", tool.category);
        } else {
            warn!("Tool '{}' not found in catalogue", tool_id);
        }
    }
    
    Ok(())
}

// Example of creating a custom tool filter
fn create_custom_filter() -> ToolFilter {
    ToolFilter::new()
        .with_categories(vec![
            ToolCategory::Validation,
            ToolCategory::Development,
            ToolCategory::FileSystem
        ])
        .with_required_capabilities(vec![
            "filesystem-read".to_string(),
            "validation".to_string()
        ])
        .with_security_levels(vec![
            SecurityLevel::Basic,
            SecurityLevel::Medium
        ])
        .with_tags(vec![
            "automated".to_string(),
            "testing".to_string()
        ])
        .with_search("file")
        .with_limit(10)
        .include_deprecated()
}

// Example of tool metadata creation
fn create_sample_tool_metadata() -> toka_tools::catalogue::ToolMetadata {
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    
    toka_tools::catalogue::ToolMetadata {
        id: "custom-file-processor".to_string(),
        name: "Custom File Processor".to_string(),
        version: "1.0.0".to_string(),
        description: "A custom tool for processing files with specific requirements".to_string(),
        category: ToolCategory::FileSystem,
        required_capabilities: vec!["filesystem-read".to_string(), "filesystem-write".to_string()],
        optional_capabilities: vec!["validation".to_string()],
        security_level: SecurityLevel::Medium,
        resource_limits: toka_tools::catalogue::ResourceLimits {
            max_memory_mb: 256,
            max_cpu_percent: 50.0,
            max_execution_time: std::time::Duration::from_secs(60),
            max_output_size: 1024 * 1024,
            max_output_files: 10,
            max_disk_mb: 64,
        },
        sandbox_config: toka_tools::catalogue::SandboxConfig {
            use_namespaces: false,
            allow_network: false,
            readonly_paths: vec![std::path::PathBuf::from("/workspace")],
            writable_paths: vec![std::path::PathBuf::from("/workspace/output")],
            forbidden_paths: vec![],
            allowed_syscalls: vec![],
            env_whitelist: vec!["PATH".to_string()],
            disable_ptrace: false,
            disable_core_dumps: false,
        },
        side_effects: SideEffect::External,
        transports: vec![toka_tools::catalogue::Transport::InProcess],
        protocol_mappings: vec![],
        input_schema: None,
        output_schema: None,
        execution_metadata: toka_tools::catalogue::ExecutionMetadata {
            hot_swappable: true,
            parallel_safe: true,
            resource_intensive: false,
            avg_execution_time_ms: None,
            success_rate: None,
            last_executed: None,
            execution_count: 0,
        },
        discovery_metadata: toka_tools::catalogue::DiscoveryMetadata {
            auto_discover: true,
            discovery_patterns: vec!["*.yaml".to_string()],
            discovery_priority: 50,
            deprecated: false,
            replacement_tool: None,
            tags: vec!["custom".to_string(), "file-processing".to_string()],
        },
        extensions: HashMap::new(),
        last_modified: Utc::now(),
        file_path: PathBuf::from("custom-tool.yaml"),
        checksum: "sample-checksum".to_string(),
    }
} 