# Python Tools Integration - Complete Guide

**Date**: 2025-07-12  
**Status**: ✅ COMPLETE INTEGRATION GUIDE  
**Scope**: Comprehensive Python tools integration for Toka Agent OS

## Overview

This complete guide combines the unified approach and detailed implementation for Python tools integration with the Rust-based Toka Agent OS. It provides both strategic direction and practical implementation details.


---

## From: PYTHON_TOOLS_INTEGRATION_UNIFIED_APPROACH.md


## Executive Summary

After analyzing multiple cursor branches, I've identified two complementary approaches to Python tools integration:

1. **Cleanup Branch**: Specialized analysis tools with comprehensive security and sandboxing
2. **Research-Integration Branch**: Generic external tool wrappers with auto-discovery and LLM guidance

This document presents a unified approach that combines the strengths of both, creating a comprehensive Python tools integration system for the Toka agent OS.

## Comparative Analysis

### Approach 1: Specialized Analysis Tools (Cleanup Branch)

**Strengths:**
- Dedicated `toka-analysis-tools` crate with comprehensive security
- Sandboxed execution with Linux namespaces
- Resource limits and capability validation
- Structured Python package with proper interfaces
- Comprehensive error handling and caching
- Multiple output formats (JSON, Mermaid, HTML)

**Limitations:**
- Limited to analysis tools only
- Requires separate crate for each tool category
- Less flexible for general-purpose tool integration

**Key Files:**
- `crates/toka-analysis-tools/` - Rust crate for analysis tools
- `toka_analysis_tools/` - Python package with tool registry
- Comprehensive README with security model documentation

### Approach 2: Generic External Tool Wrappers (Research-Integration Branch)

**Strengths:**
- Generic `ExternalTool`, `PythonTool`, `ShellTool` wrappers
- Auto-discovery system for workspace tools
- LLM-guided tool selection and execution
- Integration with existing `toka-tools` crate
- Flexible architecture for any external tool type
- Comprehensive research and gap analysis

**Limitations:**
- Less specialized security for Python-specific needs
- Limited sandboxing compared to analysis tools approach
- No structured Python package interface

**Key Files:**
- `crates/toka-tools/src/wrappers/` - Generic tool wrappers
- `research_tool_integration_gaps.md` - Comprehensive gap analysis
- `implementation_summary.md` - Implementation details

## Unified Architecture

### Core Components

#### 1. Enhanced Tool Wrappers (`crates/toka-tools/src/wrappers/`)

```rust
// Base external tool with security
pub struct ExternalTool {
    manifest: ToolManifest,
    executable: PathBuf,
    security_config: SecurityConfig,
    sandbox_config: SandboxConfig,
}

// Enhanced Python tool with analysis capabilities
pub struct PythonTool {
    external_tool: ExternalTool,
    python_config: PythonConfig,
    analysis_capabilities: AnalysisCapabilities,
}

// Shell tool with validation
pub struct ShellTool {
    external_tool: ExternalTool,
    shell_config: ShellConfig,
}
```

#### 2. Specialized Analysis Engine (`crates/toka-analysis-tools/`)

```rust
// Dedicated analysis tools with enhanced security
pub struct AnalysisToolRegistry {
    config: AnalysisConfig,
    executor: Arc<PythonExecutor>,
    cache: Arc<RwLock<ResultCache>>,
    sandbox: Arc<PythonSandbox>,
}

// Integration with generic tool system
impl AnalysisToolRegistry {
    pub async fn integrate_with_tool_registry(&self, registry: &ToolRegistry) -> Result<()> {
        // Register analysis tools in main registry
        // Apply enhanced security for analysis tools
    }
}
```

#### 3. Auto-Discovery and Registration (`crates/toka-tools/src/discovery/`)

```rust
pub struct ToolDiscovery {
    config: DiscoveryConfig,
    security_validator: SecurityValidator,
    capability_inference: CapabilityInference,
}

impl ToolDiscovery {
    pub async fn discover_python_tools(&self) -> Result<Vec<PythonTool>>;
    pub async fn discover_analysis_tools(&self) -> Result<Vec<AnalysisToolSpec>>;
    pub async fn register_all_tools(&self, registry: &ToolRegistry) -> Result<()>;
}
```

#### 4. Unified Security Model

```rust
pub struct UnifiedSecurityConfig {
    // Base security for all external tools
    pub base_security: SecurityConfig,
    
    // Enhanced security for analysis tools
    pub analysis_security: AnalysisSecurityConfig,
    
    // Capability-based access control
    pub capability_enforcement: CapabilityEnforcement,
    
    // Resource limits by tool category
    pub resource_limits: CategoryResourceLimits,
}
```

### Integration Points

#### 1. Agent Runtime Integration

```rust
// Enhanced agent runtime with unified tool support
pub struct TokaAgentRuntime {
    tool_registry: Arc<ToolRegistry>,
    analysis_registry: Arc<AnalysisToolRegistry>,
    discovery: Arc<ToolDiscovery>,
    security_validator: Arc<SecurityValidator>,
}

impl TokaAgentRuntime {
    pub async fn new_with_unified_tools() -> Result<Self> {
        // Initialize all registries
        // Auto-discover tools
        // Apply security policies
        // Register with kernel
    }
}
```

#### 2. LLM-Guided Tool Selection

```rust
pub struct LLMToolSelector {
    llm_gateway: Arc<LLMGateway>,
    tool_registry: Arc<ToolRegistry>,
    capability_matcher: CapabilityMatcher,
}

impl LLMToolSelector {
    pub async fn select_tool_for_task(&self, task: &str) -> Result<ToolSelection>;
    pub async fn extract_parameters(&self, task: &str, tool: &Tool) -> Result<ToolParams>;
}
```

#### 3. Kernel Integration

```rust
// Kernel operations for tool management
pub enum KernelOperation {
    RegisterTool(ToolRegistration),
    ExecuteTool(ToolExecution),
    ValidateCapabilities(CapabilityValidation),
    MonitorExecution(ExecutionMonitor),
}
```

## Implementation Strategy

### Phase 1: Foundation (Week 1-2)

1. **Merge Base Components**
   - Combine external tool wrappers from research branch
   - Integrate analysis engine from cleanup branch
   - Create unified security model

2. **Unified Tool Registry**
   - Enhanced `ToolRegistry` with analysis tools support
   - Auto-discovery system for all tool types
   - Security validation pipeline

3. **Core Infrastructure**
   - Sandboxing system for Python tools
   - Resource management and limits
   - Capability validation framework

### Phase 2: Security Enhancement (Week 3-4)

1. **Enhanced Sandboxing**
   - Linux namespaces for process isolation
   - Filesystem access restrictions
   - Network isolation controls

2. **Capability System**
   - Fine-grained capability declarations
   - Runtime capability validation
   - Agent capability intersection

3. **Resource Management**
   - Memory, CPU, and time limits
   - Disk I/O restrictions
   - Process monitoring

### Phase 3: Advanced Features (Week 5-6)

1. **LLM Integration**
   - Intelligent tool selection
   - Parameter extraction from natural language
   - Error interpretation and retry logic

2. **Caching and Optimization**
   - Result caching for idempotent operations
   - Tool execution optimization
   - Performance monitoring

3. **Advanced Analysis**
   - Multi-tool orchestration
   - Cross-tool data sharing
   - Workflow execution

### Phase 4: Production Readiness (Week 7-8)

1. **Comprehensive Testing**
   - Security testing and validation
   - Performance benchmarking
   - Integration testing

2. **Documentation and Examples**
   - Complete API documentation
   - Usage examples and tutorials
   - Best practices guide

3. **Production Deployment**
   - Configuration management
   - Monitoring and alerting
   - Backup and recovery

## Security Architecture

### Multi-Layer Security Model

```rust
pub struct SecurityLayer {
    // Layer 1: Tool Discovery Security
    pub discovery_security: DiscoverySecurityConfig,
    
    // Layer 2: Registration Security
    pub registration_security: RegistrationSecurityConfig,
    
    // Layer 3: Execution Security
    pub execution_security: ExecutionSecurityConfig,
    
    // Layer 4: Output Security
    pub output_security: OutputSecurityConfig,
}
```

### Capability-Based Access Control

```rust
pub struct CapabilityModel {
    // Base capabilities for all tools
    pub base_capabilities: Vec<Capability>,
    
    // Python-specific capabilities
    pub python_capabilities: Vec<PythonCapability>,
    
    // Analysis-specific capabilities
    pub analysis_capabilities: Vec<AnalysisCapability>,
    
    // Agent-specific capabilities
    pub agent_capabilities: Vec<AgentCapability>,
}
```

### Sandbox Configuration

```rust
pub struct SandboxConfig {
    // Process isolation
    pub use_namespaces: bool,
    pub allowed_syscalls: Vec<Syscall>,
    
    // Filesystem restrictions
    pub readonly_paths: Vec<PathBuf>,
    pub writable_paths: Vec<PathBuf>,
    pub forbidden_paths: Vec<PathBuf>,
    
    // Network restrictions
    pub allow_network: bool,
    pub allowed_hosts: Vec<String>,
    
    // Resource limits
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub time_limit: Option<Duration>,
}
```

## Tool Categories and Configurations

### Analysis Tools (High Security)

```rust
pub struct AnalysisToolConfig {
    // Enhanced sandboxing
    pub sandbox: SandboxConfig {
        use_namespaces: true,
        allow_network: false,
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        cpu_limit: Some(0.5), // 50% CPU
        time_limit: Some(Duration::from_secs(600)), // 10 minutes
    },
    
    // Restricted capabilities
    pub capabilities: vec![
        "filesystem-read",
        "filesystem-write-restricted",
        "process-spawn-restricted",
    ],
    
    // Output validation
    pub output_validation: OutputValidationConfig {
        max_output_size: 10 * 1024 * 1024, // 10MB
        allowed_formats: vec!["json", "mermaid", "html"],
        sanitize_output: true,
    },
}
```

### General Tools (Moderate Security)

```rust
pub struct GeneralToolConfig {
    // Standard sandboxing
    pub sandbox: SandboxConfig {
        use_namespaces: false,
        allow_network: true,
        memory_limit: Some(256 * 1024 * 1024), // 256MB
        cpu_limit: Some(0.25), // 25% CPU
        time_limit: Some(Duration::from_secs(300)), // 5 minutes
    },
    
    // Standard capabilities
    pub capabilities: vec![
        "filesystem-read",
        "filesystem-write",
        "process-spawn",
        "network-access",
    ],
}
```

### Utility Tools (Basic Security)

```rust
pub struct UtilityToolConfig {
    // Minimal sandboxing
    pub sandbox: SandboxConfig {
        use_namespaces: false,
        allow_network: false,
        memory_limit: Some(128 * 1024 * 1024), // 128MB
        cpu_limit: Some(0.1), // 10% CPU
        time_limit: Some(Duration::from_secs(60)), // 1 minute
    },
    
    // Basic capabilities
    pub capabilities: vec![
        "filesystem-read",
    ],
}
```

## Usage Examples

### 1. Auto-Discovery and Registration

```rust
use toka_tools::{UnifiedToolRegistry, ToolDiscovery};

// Initialize unified registry
let registry = UnifiedToolRegistry::new().await?;

// Auto-discover all tools
let discovery = ToolDiscovery::new_with_security_defaults();
let discovered = discovery.discover_all_tools().await?;

// Register with appropriate security levels
registry.register_tools_with_security(discovered).await?;

println!("Registered {} tools", registry.tool_count());
```

### 2. Agent Task Execution

```rust
use toka_agent_runtime::TokaAgentRuntime;

let runtime = TokaAgentRuntime::new_with_unified_tools().await?;

// Agent receives natural language task
let task = "Analyze the control flow of the main function and create a visualization";

// LLM selects appropriate tool and extracts parameters
let execution = runtime.execute_task_with_llm_guidance(task).await?;

// Result includes tool output and execution metadata
println!("Task result: {}", execution.result.output);
```

### 3. Direct Tool Usage

```rust
use toka_tools::wrappers::PythonTool;

// Create enhanced Python tool
let analyzer = PythonTool::new_analysis_tool(
    PathBuf::from("control_flow_graph_visualizer.py"),
    "control-flow-analyzer",
    vec!["code-analysis", "visualization"],
).await?;

// Execute with security validation
let params = ToolParams {
    name: "control-flow-analyzer".to_string(),
    args: [
        ("target_function", "main"),
        ("output_format", "mermaid"),
    ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
};

let result = analyzer.execute_secure(&params).await?;
```

## Migration Path

### Immediate Actions (Week 1)

1. **Merge Existing Code**
   - Combine tool wrappers from research branch
   - Integrate analysis engine from cleanup branch
   - Resolve conflicts and create unified API

2. **Security Integration**
   - Apply enhanced security from cleanup branch to all tools
   - Implement capability validation for generic tools
   - Add sandboxing configuration

3. **Registry Unification**
   - Merge tool registries into single system
   - Implement auto-discovery for all tool types
   - Add security classification

### Short-term Goals (Week 2-4)

1. **Complete Integration**
   - LLM-guided tool selection
   - Enhanced Python tool support
   - Comprehensive testing

2. **Security Hardening**
   - Implement full sandboxing
   - Add capability enforcement
   - Resource management

3. **Production Features**
   - Monitoring and logging
   - Error handling and recovery
   - Performance optimization

### Long-term Vision (Month 2-3)

1. **Advanced Capabilities**
   - Multi-tool orchestration
   - Workflow execution
   - Cross-agent tool sharing

2. **Ecosystem Integration**
   - Plugin system for custom tools
   - Tool marketplace
   - Community contributions

3. **AI-Enhanced Operations**
   - Intelligent tool recommendation
   - Automated tool optimization
   - Predictive resource management

## Benefits of Unified Approach

### 1. Comprehensive Security
- Multi-layer security model
- Capability-based access control
- Enhanced sandboxing for sensitive operations
- Consistent security policies across all tools

### 2. Operational Excellence
- Unified tool registry and management
- Auto-discovery and registration
- LLM-guided tool selection
- Comprehensive monitoring and logging

### 3. Developer Experience
- Consistent API across all tool types
- Automatic tool discovery and registration
- Rich documentation and examples
- Extensible architecture for custom tools

### 4. Performance and Scalability
- Efficient tool execution
- Resource management and limits
- Caching and optimization
- Concurrent execution support

## Conclusion

The unified approach combines the specialized security and analysis capabilities of the cleanup branch with the flexible, generic tool integration of the research branch. This creates a comprehensive Python tools integration system that is:

- **Secure**: Multi-layer security with capability-based access control
- **Flexible**: Support for any type of external tool
- **Intelligent**: LLM-guided tool selection and execution
- **Scalable**: Auto-discovery and efficient resource management
- **Maintainable**: Clean architecture with clear separation of concerns

This unified system positions Toka as a true agentic operating system where Python tools and shell scripts can be safely registered, used, and executed within the Rust-based agent runtime, with security, efficiency, and intelligence built-in from the ground up.

## Next Steps

1. **Implement Core Unification** - Merge the best components from both branches
2. **Security Enhancement** - Implement comprehensive security model
3. **Testing and Validation** - Comprehensive testing of unified system
4. **Documentation** - Complete API and usage documentation
5. **Production Deployment** - Deploy in production environment

The unified approach provides a solid foundation for secure, efficient, and intelligent Python tools integration within the Toka agent OS ecosystem.
---

## From: UNIFIED_PYTHON_TOOLS_INTEGRATION_README.md

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