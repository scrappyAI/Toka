# Toka Unified Tool System

This directory contains all tools available in the Toka ecosystem, organized in a unified structure that supports composable, hot-swappable execution through the agent runtime.

## Architecture Overview

The unified tool system integrates three key components:
1. **Tool Discovery & Security** (`crates/toka-tools`) - Automatic discovery, sandboxing, capability validation
2. **Agent Runtime Integration** - Hot-swappable tools that agents can dynamically load and execute
3. **Unified Configuration** - YAML-based tool manifests with consistent schema

## Directory Structure

```
tools/
├── manifests/          # Tool manifests (YAML) - tool definitions and capabilities
├── analysis/           # Code and system analysis tools
├── system/            # System management and build tools
├── validation/        # Quality assurance and validation tools
├── runtime/           # Runtime and orchestration tools
└── README.md          # This file
```

## Tool Categories

### Analysis Tools (`analysis/`)
High-security tools for code analysis, dependency graphing, and system inspection:
- Control flow analysis
- Dependency visualization 
- RAFT consensus analysis
- Performance monitoring

**Security Level:** High (sandboxed, limited filesystem access)

### System Tools (`system/`)
Medium-security tools for build system, environment setup, and CI/CD:
- Build validation
- Environment setup
- Workstream coordination
- GitHub workflow management

**Security Level:** Medium (controlled filesystem access, limited network)

### Validation Tools (`validation/`)
Tools for quality assurance, testing, and compliance:
- Date validation
- Configuration validation
- Test execution
- Compliance checking

**Security Level:** Medium (read-only with specific write permissions)

### Runtime Tools (`runtime/`)
Tools that integrate directly with the agent runtime for orchestration:
- Agent lifecycle management
- Task coordination
- Resource monitoring
- Hot-swapping utilities

**Security Level:** Variable (based on specific tool requirements)

## Tool Manifest Format

All tools use unified YAML manifests in `manifests/` directory:

```yaml
# Example: tools/manifests/control-flow-analyzer.yaml
metadata:
  name: "control-flow-analyzer"
  version: "1.0.0"
  category: "analysis"
  description: "Analyzes Rust code control flow and generates visualizations"

spec:
  executable:
    type: "python"
    path: "analysis/control_flow_graph_visualizer.py"
    interpreter: "python3"
  
  capabilities:
    required:
      - "filesystem-read"
      - "code-analysis"
      - "visualization"
    optional:
      - "filesystem-write"  # for output files
  
  security:
    level: "high"
    sandbox:
      memory_limit: "512MB"
      cpu_limit: "50%"
      timeout: "10m"
      allow_network: false
      readonly_paths:
        - "/workspace/crates"
        - "/workspace/src"
      writable_paths:
        - "/workspace/control_flow_graphs"
  
  parameters:
    - name: "target_function"
      type: "string"
      description: "Function name to analyze"
      required: false
    - name: "output_format"
      type: "enum"
      values: ["mermaid", "png", "svg", "html"]
      default: "mermaid"
    - name: "complexity_analysis"
      type: "boolean"
      default: true

interface:
  discovery:
    auto_discover: true
    patterns:
      - "**/*visualizer*.py"
      - "**/control_flow*.py"
  
  execution:
    hot_swappable: true
    parallel_safe: true
    resource_intensive: true
  
  integration:
    agent_types: ["analysis", "development", "orchestration"]
    runtime_events: ["agent_lifecycle", "task_completion"]
    
protocols:
  - type: "mcp"
    function_name: "analyze_control_flow"
    version: "1.0"
  - type: "a2a"
    action: "code_analysis"
```

## Integration with Agent Runtime

Tools are automatically discovered and registered with the agent runtime through:

1. **Discovery Phase:** `toka-tools` scans tool directories and manifests
2. **Registration:** Tools are registered with capability validation and security classification
3. **Runtime Integration:** Agents can dynamically load and execute tools based on capabilities
4. **Hot-Swapping:** Tools can be updated or replaced without restarting agents

## Usage Examples

### Agent Configuration
```yaml
# In agent configuration
capabilities:
  primary:
    - "code-analysis"
    - "visualization"
  tools:
    preferred:
      - "control-flow-analyzer"
      - "dependency-visualizer"
    fallback:
      - "simple-code-analyzer"
```

### Runtime Execution
```rust
// In agent code
let tool_registry = context.get_tool_registry();
let analyzer = tool_registry
    .get_tool("control-flow-analyzer")
    .await?;

let params = ToolParams {
    name: "control-flow-analyzer".to_string(),
    args: [
        ("target_function", "process_request"),
        ("output_format", "mermaid"),
    ].into_iter().collect(),
};

let result = tool_registry
    .execute_tool_secure("control-flow-analyzer", &params, &agent_capabilities)
    .await?;
```

## Security Model

All tools operate under the unified security model:
- **Capability-based access control**
- **Sandboxed execution with resource limits**
- **Automated security classification**
- **Audit logging for all tool executions**

## Migration Status

This unified tool system replaces:
- Scattered Python files at project root
- Mixed configuration formats (TOML/YAML)
- Disconnected tool discovery mechanisms
- Manual tool registration and execution

All legacy tool locations are being migrated to this unified structure. 