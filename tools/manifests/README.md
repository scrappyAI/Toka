# Tool Manifests

This directory contains YAML manifests for all tools in the unified Toka tool system. Each manifest defines:

- **Tool metadata**: Name, version, description, and categorization
- **Execution specifications**: How to run the tool and its requirements
- **Security configuration**: Sandbox settings and capability requirements
- **Runtime integration**: Agent types, protocols, and event handling
- **Parameter schema**: Input validation and default values

## Manifest Structure

```yaml
metadata:
  name: "tool-name"
  version: "1.0.0"
  category: "analysis|system|validation|runtime"
  description: "Tool description"
  author: "Toka Development Team"
  created: "2025-07-06"

spec:
  executable:
    type: "python|shell|binary"
    path: "relative/path/to/executable"
    interpreter: "python3|bash|node"
    working_directory: "."
  
  capabilities:
    required: ["filesystem-read", "code-analysis"]
    optional: ["filesystem-write", "network-access"]
  
  security:
    level: "high|medium|low"
    sandbox:
      memory_limit: "512MB"
      cpu_limit: "50%"
      timeout: "10m"
      allow_network: false
      readonly_paths: ["src", "crates"]
      writable_paths: ["output"]
  
  parameters:
    - name: "param_name"
      type: "string|enum|boolean|integer"
      description: "Parameter description"
      required: true
      default: "default_value"
      values: ["option1", "option2"]  # for enum types

interface:
  discovery:
    auto_discover: true
    patterns: ["**/pattern*.py"]
  
  execution:
    hot_swappable: true
    parallel_safe: true
    resource_intensive: false
  
  integration:
    agent_types: ["analysis", "development"]
    runtime_events: ["agent_lifecycle"]
    compatible_backends: ["all"]

protocols:
  - type: "mcp"
    function_name: "function_name"
    version: "1.0"
  - type: "a2a"
    action: "namespace.action"

outputs:
  primary:
    - type: "visualization"
      formats: ["mermaid", "svg", "png"]
      location: "output_directory"
    - type: "report"
      formats: ["markdown", "json"]
      location: "reports"
  
  metrics:
    - "execution_time"
    - "files_processed"
    - "errors_found"

dependencies:
  system: ["python3", "graphviz"]
  python: ["requests>=2.28.0", "pyyaml>=6.0.1"]
  services: ["optional-service"]
  workspace: ["Cargo.toml", "src/**/*.rs"]
```

## Security Levels

- **High**: Sandboxed execution with strict resource limits and no network access
- **Medium**: Controlled execution with network restrictions and moderate resource limits
- **Low**: Minimal restrictions for trusted tools

## Tool Categories

- **analysis**: Code analysis, dependency graphing, control flow analysis
- **system**: Build validation, environment setup, system management
- **validation**: Date validation, compliance checking, quality assurance
- **runtime**: Agent lifecycle, orchestration, runtime management

## Integration Points

Tools integrate with the agent runtime through:

1. **Discovery**: Automatic tool discovery using file patterns
2. **Execution**: Hot-swappable execution with capability validation
3. **Protocols**: MCP (Model Context Protocol) and A2A (Agent-to-Agent) communication
4. **Events**: Runtime event handling for agent lifecycle and task completion

## Usage

Tool manifests are automatically loaded by the `RuntimeToolRegistry` and used to:

- Validate agent capabilities before tool execution
- Configure sandbox environments for secure execution
- Enable hot-swapping of tools without agent restart
- Provide parameter validation and default values
- Route tool outputs to appropriate locations

The unified tool system ensures all tools are discoverable, composable, and securely executable within the Toka agent runtime. 