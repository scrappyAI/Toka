# Unified Python Tools Integration for Toka Agent OS

A comprehensive system for securely integrating Python tools and shell scripts with the Rust-based Toka agent operating system, combining the strengths of specialized analysis tools with flexible external tool integration.

## Overview

This unified integration system provides:

- **🔒 Multi-Level Security**: Graduated security levels from basic to high-security analysis tools
- **🔍 Auto-Discovery**: Intelligent discovery and registration of Python and shell tools
- **🤖 LLM-Guided Execution**: AI-powered tool selection and parameter extraction
- **📊 Comprehensive Analysis**: Specialized support for code analysis and visualization tools
- **⚡ Performance Optimized**: Efficient execution with caching and resource management
- **🛡️ Capability-Based Security**: Fine-grained permission system with runtime validation

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        Unified Python Tools Integration                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Auto-Discovery  │  Security Validator  │  LLM Selector  │  Tool Registry      │
├─────────────────────────────────────────────────────────────────────────────────┤
│  High Security   │     Medium Security     │      Basic Security              │
│  Analysis Tools  │     System Tools       │      Utility Tools               │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Python Wrapper  │     Shell Wrapper      │      External Wrapper           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                              Sandbox Executor                                   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Basic Usage

```rust
use toka_tools::wrappers::{UnifiedToolRegistry, ToolDiscovery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize unified registry
    let registry = UnifiedToolRegistry::new().await?;
    
    // Auto-discover all tools in workspace
    let count = registry.auto_register_tools().await?;
    println!("Registered {} tools", count);
    
    // List available tools
    let tools = registry.list_tools().await;
    for tool in tools {
        println!("Available tool: {}", tool);
    }
    
    Ok(())
}
```

### 2. Execute Tools with Security Validation

```rust
use toka_tools::core::{ToolParams, ToolResult};
use std::collections::HashMap;

// Agent capabilities (would come from agent configuration)
let agent_capabilities = vec![
    "filesystem-read".to_string(),
    "code-analysis".to_string(),
];

// Prepare tool parameters
let mut args = HashMap::new();
args.insert("target_function".to_string(), "main".to_string());
args.insert("output_format".to_string(), "mermaid".to_string());

let params = ToolParams {
    name: "control-flow-graph-visualizer".to_string(),
    args,
};

// Execute with security validation
let result = registry.execute_tool_secure(
    "control-flow-graph-visualizer",
    &params,
    &agent_capabilities,
).await?;

println!("Tool output: {}", result.output);
```

### 3. Custom Tool Discovery

```rust
use toka_tools::wrappers::{ToolDiscoveryBuilder, SecurityLevel};

let discovery = ToolDiscoveryBuilder::new()
    .add_search_directory("scripts")
    .add_search_directory("tools")
    .add_include_pattern("*.py")
    .add_include_pattern("*.sh")
    .add_exclude_pattern("*test*")
    .add_exclude_pattern("*__pycache__*")
    .max_depth(3)
    .build();

let discovered_tools = discovery.discover_all_tools().await?;

// Register with appropriate security levels
for tool_spec in discovered_tools {
    let security_level = if tool_spec.capabilities.contains(&"code-analysis".to_string()) {
        SecurityLevel::High
    } else if tool_spec.capabilities.contains(&"system-monitoring".to_string()) {
        SecurityLevel::Medium
    } else {
        SecurityLevel::Basic
    };
    
    registry.register_tool_with_security(tool_spec, security_level).await?;
}
```

## Security Model

### Security Levels

#### High Security (Analysis Tools)
- **Sandboxing**: Linux namespaces enabled
- **Network**: Disabled
- **Memory Limit**: 512MB
- **CPU Limit**: 50%
- **Execution Time**: 10 minutes
- **Capabilities**: Code analysis, visualization, restricted filesystem access

#### Medium Security (System Tools)
- **Sandboxing**: Process isolation only
- **Network**: Enabled
- **Memory Limit**: 256MB
- **CPU Limit**: 25%
- **Execution Time**: 5 minutes
- **Capabilities**: System monitoring, build tools, network access

#### Basic Security (Utility Tools)
- **Sandboxing**: Minimal restrictions
- **Network**: Disabled
- **Memory Limit**: 128MB
- **CPU Limit**: 10%
- **Execution Time**: 1 minute
- **Capabilities**: File processing, basic utilities

### Capability System

```rust
// Standard capabilities
let capabilities = vec![
    "filesystem-read",      // Read files
    "filesystem-write",     // Write files
    "process-spawn",        // Execute processes
    "network-access",       // Network operations
    "code-analysis",        // Analyze code structure
    "system-monitoring",    // Monitor system resources
    "visualization",        // Generate visualizations
];
```

### Sandbox Configuration

```rust
use toka_tools::wrappers::security::{SecurityConfig, SecurityLevel};

// Create high-security configuration
let config = SecurityConfig::for_level(SecurityLevel::High);

// Or custom configuration
let custom_config = SecurityConfig::with_capabilities(
    SecurityLevel::Medium,
    vec![
        "filesystem-read".to_string(),
        "code-analysis".to_string(),
    ]
);
```

## Tool Categories

### Analysis Tools (High Security)

**Discovered from workspace:**
- `control_flow_graph_visualizer.py` → `control-flow-visualizer`
- `dependency_graph_visualizer.py` → `dependency-visualizer`
- `raft_analysis.py` → `raft-analyzer`

**Capabilities:**
- Code structure analysis
- Dependency visualization
- Mermaid diagram generation
- Interactive HTML reports

**Example Usage:**
```rust
let params = ToolParams {
    name: "control-flow-visualizer".to_string(),
    args: [
        ("target_function", "process_append_entries_request"),
        ("output_format", "mermaid"),
        ("include_complexity", "true"),
    ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
};

let result = registry.execute_tool_secure(
    "control-flow-visualizer",
    &params,
    &["filesystem-read", "code-analysis", "visualization"],
).await?;
```

### System Tools (Medium Security)

**Discovered from workspace:**
- `scripts/validate-build-system.sh` → `build-validator`
- `scripts/test-toka-system.sh` → `system-tester`
- `monitor_raft_development.py` → `raft-monitor`
- `raft_monitoring_service.sh` → `raft-service`

**Capabilities:**
- System validation
- Build system testing
- Performance monitoring
- Service management

### Utility Tools (Basic Security)

**Discovered from workspace:**
- `scripts/validate_dates.py` → `date-validator`
- `scripts/setup-env.sh` → `env-setup`
- `scripts/insert_date.sh` → `date-inserter`

**Capabilities:**
- File processing
- Date validation
- Environment setup
- Text processing

## Integration with Existing Systems

### Agent Runtime Integration

```rust
use toka_agent_runtime::TokaAgentRuntime;
use toka_tools::wrappers::UnifiedToolRegistry;

// Enhanced agent runtime with unified tools
let runtime = TokaAgentRuntime::new_with_unified_tools().await?;

// Agents can now use any discovered tool
let task = "Analyze the control flow of the main function and create a visualization";
let execution = runtime.execute_task_with_llm_guidance(task).await?;
```

### LLM-Guided Tool Selection

```rust
use toka_tools::wrappers::LLMToolSelector;

let selector = LLMToolSelector::new(llm_gateway, tool_registry);

// Natural language task
let task = "Generate a dependency graph for the toka-agents crate";

// LLM selects appropriate tool and extracts parameters
let tool_selection = selector.select_tool_for_task(task).await?;
let params = selector.extract_parameters(task, &tool_selection.tool).await?;

// Execute the selected tool
let result = registry.execute_tool_secure(
    &tool_selection.tool_name,
    &params,
    &agent_capabilities,
).await?;
```

### Kernel Integration

```rust
use toka_kernel::{KernelOperation, ToolRegistration};

// Register tools with kernel
let registration = ToolRegistration {
    tool_name: "control-flow-visualizer".to_string(),
    capabilities: vec!["code-analysis".to_string()],
    security_level: SecurityLevel::High,
};

kernel.execute_operation(KernelOperation::RegisterTool(registration)).await?;
```

## Configuration

### Discovery Configuration

```toml
# toka-tools-config.toml
[discovery]
search_directories = ["scripts", "tools", "analysis"]
include_patterns = ["*.py", "*.sh", "*.bash"]
exclude_patterns = ["*test*", "*__pycache__*", "*.pyc"]
follow_symlinks = false
max_depth = 3

[discovery.capability_rules]
name_rules = { "*analysis*" = ["code-analysis"], "*monitor*" = ["system-monitoring"] }
path_rules = { "scripts/*" = ["utility"], "analysis/*" = ["code-analysis"] }
content_rules = { "import ast" = ["code-analysis"], "import subprocess" = ["process-spawn"] }
```

### Security Configuration

```toml
# security-config.toml
[security.high]
max_memory_mb = 512
max_cpu_percent = 50.0
max_execution_time = "10m"
use_namespaces = true
allow_network = false

[security.medium]
max_memory_mb = 256
max_cpu_percent = 25.0
max_execution_time = "5m"
use_namespaces = false
allow_network = true

[security.basic]
max_memory_mb = 128
max_cpu_percent = 10.0
max_execution_time = "1m"
use_namespaces = false
allow_network = false
```

### Agent Configuration

```toml
# agents/analysis-agent.toml
[agent]
name = "analysis-agent"
capabilities = [
    "filesystem-read",
    "code-analysis",
    "visualization",
]

[agent.tools]
required = [
    "control-flow-visualizer",
    "dependency-visualizer",
    "raft-analyzer"
]

[agent.security]
security_level = "high"
sandbox = true
audit_logging = true
```

## Error Handling

```rust
use toka_tools::wrappers::security::AnalysisError;

match registry.execute_tool_secure(tool_name, &params, &agent_capabilities).await {
    Ok(result) => {
        if result.success {
            println!("Tool output: {}", result.output);
        } else {
            eprintln!("Tool execution failed: {}", result.output);
        }
    }
    Err(e) => {
        match e.downcast_ref::<AnalysisError>() {
            Some(AnalysisError::SecurityViolation(msg)) => {
                eprintln!("Security violation: {}", msg);
            }
            Some(AnalysisError::ResourceLimitExceeded(msg)) => {
                eprintln!("Resource limit exceeded: {}", msg);
            }
            Some(AnalysisError::ToolNotFound(tool)) => {
                eprintln!("Tool not found: {}", tool);
            }
            _ => {
                eprintln!("Unexpected error: {}", e);
            }
        }
    }
}
```

## Monitoring and Metrics

```rust
// Get execution metrics
let metrics = registry.get_execution_metrics("control-flow-visualizer").await;
if let Some(metrics) = metrics {
    println!("Tool: {}", metrics.tool_name);
    println!("Execution time: {:?}", metrics.execution_time);
    println!("Success: {}", metrics.success);
    println!("Timestamp: {:?}", metrics.timestamp);
}

// Get security classification
let security = registry.get_tool_security("control-flow-visualizer").await;
if let Some(security) = security {
    println!("Security level: {:?}", security.security_level);
    println!("Capabilities: {:?}", security.capabilities);
    println!("Resource limits: {:?}", security.resource_limits);
}
```

## Migration from Existing Approaches

### From Direct Script Execution

**Before:**
```bash
# Direct script execution
python3 control_flow_graph_visualizer.py --target-function main --output-format mermaid
```

**After:**
```rust
// Unified tool execution
let params = ToolParams {
    name: "control-flow-visualizer".to_string(),
    args: [
        ("target_function", "main"),
        ("output_format", "mermaid"),
    ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
};

let result = registry.execute_tool_secure(
    "control-flow-visualizer",
    &params,
    &agent_capabilities,
).await?;
```

### From Agent Direct References

**Before:**
```toml
# agents/analysis-agent.toml
[agent.interface]
executable = "control_flow_graph_visualizer.py"

[agent.commands]
analyze = {
    executable = "control_flow_graph_visualizer.py",
    args = ["--target-function", "main"]
}
```

**After:**
```toml
# agents/analysis-agent.toml
[agent]
capabilities = ["filesystem-read", "code-analysis", "visualization"]

[agent.tools]
required = ["control-flow-visualizer"]

[agent.tasks]
analyze_control_flow = {
    tool = "control-flow-visualizer",
    args = { target_function = "main", output_format = "mermaid" }
}
```

## Testing

### Unit Tests

```bash
# Run unified tool tests
cargo test --package toka-tools --lib wrappers

# Run security tests
cargo test --package toka-tools --lib wrappers::security

# Run discovery tests
cargo test --package toka-tools --lib wrappers::discovery
```

### Integration Tests

```bash
# Test with real workspace tools
cargo test --package toka-tools --test integration

# Test security validation
cargo test --package toka-tools --test security_validation

# Test tool discovery
cargo test --package toka-tools --test tool_discovery
```

### Performance Tests

```bash
# Benchmark tool execution overhead
cargo bench --package toka-tools

# Test resource limit enforcement
cargo test --package toka-tools --test resource_limits

# Test concurrent tool execution
cargo test --package toka-tools --test concurrent_execution
```

## Troubleshooting

### Common Issues

#### Tool Not Found
```
Error: Tool not found: my-script
```
**Solution:** Check discovery configuration and ensure the script is in a search directory.

#### Security Violation
```
Error: Agent lacks required capability: code-analysis
```
**Solution:** Add the required capability to the agent configuration.

#### Resource Limit Exceeded
```
Error: Output size 15728640 exceeds limit 10485760
```
**Solution:** Increase resource limits for the tool's security level or optimize tool output.

#### Sandbox Execution Failed
```
Error: Sandbox creation failed: Permission denied
```
**Solution:** Ensure proper permissions for namespace creation or disable namespaces for testing.

### Debug Mode

```rust
// Enable debug logging
env_logger::init();

// Create registry with debug configuration
let registry = UnifiedToolRegistry::new_with_debug().await?;

// Execute with detailed logging
let result = registry.execute_tool_debug(
    "control-flow-visualizer",
    &params,
    &agent_capabilities,
).await?;
```

### Validation

```bash
# Validate tool configurations
toka-tools validate-config --config toka-tools-config.toml

# Check tool discovery
toka-tools discover --dry-run

# Test security configurations
toka-tools test-security --tool control-flow-visualizer
```

## Contributing

1. **Follow Security Guidelines**: All new tools must implement proper security configurations
2. **Add Tests**: Include unit tests, integration tests, and security validation tests
3. **Update Documentation**: Document new capabilities and security considerations
4. **Performance Testing**: Benchmark new features and optimizations

### Adding New Tool Types

```rust
// Example: Adding support for R scripts
pub enum ToolType {
    Python,
    Shell,
    External,
    R,  // New tool type
}

// Implement discovery logic
fn determine_tool_type(&self, path: &Path) -> Result<ToolType> {
    if let Some(extension) = path.extension() {
        match extension.to_str() {
            Some("py") => return Ok(ToolType::Python),
            Some("sh") | Some("bash") => return Ok(ToolType::Shell),
            Some("R") | Some("r") => return Ok(ToolType::R),  // New
            _ => {}
        }
    }
    Ok(ToolType::External)
}
```

## License

This unified Python tools integration system is part of the Toka OS project and follows the same licensing terms.

## Support

For questions, issues, or contributions:
- Create an issue in the Toka OS repository
- Follow the contribution guidelines
- Ensure all security requirements are met

The unified approach provides a solid foundation for secure, efficient, and intelligent Python tools integration within the Toka agent OS ecosystem.