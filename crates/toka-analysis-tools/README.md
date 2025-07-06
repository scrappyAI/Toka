# Toka Analysis Tools

Enterprise-grade Rust integration for Python-based code analysis tools in the Toka OS ecosystem.

## Overview

This crate provides secure, sandboxed execution of Python analysis tools within the Toka system, enabling agents to perform code analysis while maintaining strict security boundaries and resource controls.

## Features

- **🔒 Secure Execution**: All Python tools run in sandboxed environments with Linux namespaces
- **⚡ Multiple Analysis Tools**: Control flow analysis, dependency analysis, and combined analysis
- **🎯 Multiple Output Formats**: Mermaid diagrams, JSON, HTML, and Markdown
- **📊 Comprehensive Metrics**: Performance monitoring and resource usage tracking
- **🗄️ Result Caching**: Intelligent caching of analysis results with TTL
- **🛡️ Security First**: Capability validation, input sanitization, and resource limits
- **🔧 Toka Integration**: Native tool registration with the Toka tool system

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Toka Analysis Tools                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Tool Registry  │  Security Layer  │  Executor  │  Output Processing  │  Cache  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                              Secure Python Sandbox                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────────┐  │
│  │ Control Flow    │  │ Dependency      │  │ Combined Analysis              │  │
│  │ Analysis        │  │ Analysis        │  │ (Multiple Tools)               │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Add to Dependencies

```toml
[dependencies]
toka-analysis-tools = { path = "../toka-analysis-tools" }
toka-tools = { path = "../toka-tools" }
```

### 2. Basic Integration

```rust
use std::collections::HashMap;
use toka_tools::{ToolRegistry, ToolParams};
use toka_analysis_tools::{AnalysisToolRegistry, AnalysisConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create main Toka tool registry
    let tool_registry = ToolRegistry::new().await?;
    
    // Create and configure analysis tools
    let analysis_registry = AnalysisToolRegistry::new().await?;
    analysis_registry.register_all_tools(&tool_registry).await?;
    
    // Execute control flow analysis
    let mut args = HashMap::new();
    args.insert("target_function".to_string(), "main".to_string());
    args.insert("output_format".to_string(), "mermaid".to_string());
    
    let params = ToolParams {
        name: "control-flow-analysis".to_string(),
        args,
    };
    
    let result = tool_registry.execute_tool("control-flow-analysis", &params).await?;
    println!("Analysis result: {}", result.output);
    
    Ok(())
}
```

## Available Tools

### Control Flow Analysis (`control-flow-analysis`)

Analyzes function control flow patterns and complexity metrics.

**Parameters:**
- `target_function` (optional): Specific function to analyze
- `target_file` (optional): Specific file to analyze
- `output_format`: Output format (`json`, `mermaid`, `html`, `markdown`)
- `include_complexity`: Include complexity metrics (`true`/`false`)

**Output:**
- Function control flow diagrams
- Complexity metrics (cyclomatic, cognitive)
- Mermaid flowcharts
- Async pattern detection

### Dependency Analysis (`dependency-analysis`)

Analyzes crate dependencies and workspace architecture.

**Parameters:**
- `workspace_root` (optional): Workspace root path
- `output_format`: Output format (`json`, `mermaid`, `html`, `markdown`)
- `include_agents` (optional): Include agent composition analysis

**Output:**
- Dependency graphs
- Crate relationship diagrams
- Agent composition analysis
- Architecture visualization

### Combined Analysis (`combined-analysis`)

Runs multiple analysis tools together for comprehensive reports.

**Parameters:**
- `output_format`: Output format (`json`, `mermaid`, `html`, `markdown`)
- `include_mermaid`: Include Mermaid diagrams (`true`/`false`)

**Output:**
- Combined analysis results
- Cross-referential insights
- Comprehensive reports

## Security Model

### Sandboxing

All Python tools execute in secure sandboxes with:

- **Process Isolation**: Linux namespaces (PID, mount, UTS)
- **Network Isolation**: No network access by default
- **Filesystem Restrictions**: Limited read/write access
- **Resource Limits**: CPU, memory, disk, and execution time limits

### Capability Validation

Tools require explicit capabilities:

- `filesystem-read`: Read access to workspace files
- `filesystem-write`: Write access to output directories
- `process-spawn`: Permission to execute Python processes

### Input Sanitization

All inputs are validated for:

- Path traversal attempts (`../`, `~`)
- Command injection (`&&`, `||`, `;`)
- Shell metacharacters (`$()`, `${}`, backticks)
- Dangerous Python patterns (`eval`, `exec`, `import os`)

## Configuration

### Basic Configuration

```rust
use toka_analysis_tools::AnalysisConfig;
use std::time::Duration;

let config = AnalysisConfig {
    python_path: std::path::PathBuf::from("python3"),
    tools_directory: std::path::PathBuf::from("toka_analysis_tools"),
    output_directory: std::path::PathBuf::from("target/analysis"),
    workspace_root: std::path::PathBuf::from("."),
    timeout: Duration::from_secs(600),
    enable_cache: true,
    enable_metrics: true,
    ..Default::default()
};
```

### Production Configuration

```rust
use toka_analysis_tools::{AnalysisConfig, ResourceLimits, CacheConfig};

let config = AnalysisConfig {
    // Secure Python path
    python_path: std::path::PathBuf::from("/usr/bin/python3"),
    
    // Restricted directories
    tools_directory: std::path::PathBuf::from("/opt/toka/analysis-tools"),
    output_directory: std::path::PathBuf::from("/tmp/toka-analysis"),
    
    // Tight resource limits
    resource_limits: ResourceLimits {
        max_memory_mb: 256,
        max_cpu_percent: 25.0,
        max_execution_time: Duration::from_secs(300),
        max_output_size: 5 * 1024 * 1024,
        max_output_files: 50,
        max_disk_mb: 512,
    },
    
    // Optimized caching
    cache_config: CacheConfig {
        enabled: true,
        max_size: 500,
        ttl: Duration::from_secs(1800),
    },
    
    ..Default::default()
};
```

## Agent Integration

### Capability Declaration

Agents must declare required capabilities in their configuration:

```toml
# agents/analysis-agent.toml
[agent.security]
sandbox = true
capabilities_required = [
    "filesystem-read",
    "filesystem-write", 
    "process-spawn"
]
resource_limits = {
    max_memory = "512MB",
    max_cpu = "50%",
    timeout = "10m"
}
```

### Runtime Validation

```rust
use toka_agent_runtime::CapabilityValidator;

// The agent runtime automatically validates capabilities
let validator = CapabilityValidator::new(
    capabilities_required,
    security_config,
);

// Tools are only accessible if agent has required capabilities
if validator.can_perform("process-spawn")? {
    // Agent can use analysis tools
}
```

## Performance & Monitoring

### Metrics Collection

```rust
// Get execution metrics
let metrics = analysis_registry.get_metrics();
let all_metrics = metrics.get_metrics().await;

for metric in all_metrics {
    println!("Tool: {}, Success: {}, Time: {:?}", 
             metric.tool_name, 
             metric.success, 
             metric.execution_time);
}
```

### Cache Statistics

```rust
// Monitor cache performance
let cache_stats = analysis_registry.get_cache_stats().await;
println!("Cache hits: {}, misses: {}, entries: {}", 
         cache_stats.hits, 
         cache_stats.misses, 
         cache_stats.entries);
```

### Resource Monitoring

All executions are monitored for:
- Memory usage
- CPU consumption
- Execution time
- Output size
- File system usage

## Python Tool Requirements

The Python analysis tools must follow specific interfaces:

### Command Line Interface

```bash
python3 control_flow.py \
    --workspace-root /workspace \
    --output-dir /tmp/analysis \
    --execution-id uuid-here \
    --target-function main \
    --output-format mermaid
```

### Environment Variables

- `TOKA_WORKSPACE_ROOT`: Workspace root directory
- `TOKA_OUTPUT_DIR`: Output directory for results
- `TOKA_EXECUTION_ID`: Unique execution identifier
- `TOKA_SANDBOX`: Indicates sandboxed execution

### Output Format

Tools must output structured JSON to stdout:

```json
{
    "success": true,
    "analysis": {
        "function_name": "main",
        "complexity": 5,
        "control_flow": {...},
        "mermaid_diagram": "flowchart TD..."
    },
    "metadata": {
        "tool_version": "1.0.0",
        "execution_time": "2.5s",
        "output_files": ["diagram.svg", "report.html"]
    }
}
```

## Error Handling

The system provides comprehensive error handling:

```rust
match tool_registry.execute_tool("control-flow-analysis", &params).await {
    Ok(result) => {
        if result.success {
            println!("Analysis completed: {}", result.output);
        } else {
            println!("Analysis failed: check logs for details");
        }
    }
    Err(e) => {
        match e.downcast_ref::<AnalysisError>() {
            Some(AnalysisError::SecurityViolation(msg)) => {
                println!("Security violation: {}", msg);
            }
            Some(AnalysisError::ResourceLimitExceeded(msg)) => {
                println!("Resource limit exceeded: {}", msg);
            }
            Some(AnalysisError::ExecutionFailed(msg)) => {
                println!("Execution failed: {}", msg);
            }
            _ => {
                println!("Unexpected error: {}", e);
            }
        }
    }
}
```

## Examples

See the [`examples/`](examples/) directory for comprehensive examples:

- [`integration.rs`](examples/integration.rs): Complete integration example
- Agent integration patterns
- Custom configuration examples
- Error handling patterns

## Development

### Running Examples

```bash
# Run the integration example
cargo run --example integration --features="control-flow-analysis,dependency-analysis"

# Run with sandbox disabled (testing only)
cargo run --example integration --features="unsafe-direct-execution"
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with sandbox features
cargo test --features="sandbox"

# Run integration tests
cargo test --test integration
```

### Building

```bash
# Build with all features
cargo build --all-features

# Build production optimized
cargo build --release --features="control-flow-analysis,dependency-analysis"
```

## Security Considerations

### Production Deployment

1. **Python Environment**: Use isolated Python environments
2. **File Permissions**: Restrict file system access to minimum required
3. **Network Isolation**: Disable network access in production
4. **Resource Limits**: Set conservative resource limits
5. **Audit Logging**: Enable comprehensive audit logging
6. **Regular Updates**: Keep Python dependencies updated

### Container Deployment

```dockerfile
# Use minimal Python image
FROM python:3.11-slim

# Install analysis tools
COPY toka_analysis_tools/ /opt/toka/analysis-tools/
RUN pip install -r /opt/toka/analysis-tools/requirements.txt

# Create non-root user
RUN useradd -r -s /bin/false toka-analysis

# Set secure permissions
RUN chown -R toka-analysis:toka-analysis /opt/toka/analysis-tools
USER toka-analysis

# Configure resource limits
LABEL toka.resource.memory="512MB"
LABEL toka.resource.cpu="0.5"
LABEL toka.security.sandbox="true"
```

## License

This crate is part of the Toka OS project and follows the same licensing terms.

## Contributing

Contributions are welcome! Please ensure all contributions:

1. Follow Rust coding standards (`rustfmt`, `clippy`)
2. Include comprehensive tests
3. Maintain security best practices
4. Update documentation as needed

See the main Toka OS contributing guidelines for detailed information.