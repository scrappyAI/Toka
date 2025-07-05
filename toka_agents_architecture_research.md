# Toka Agents Architecture Research Report

**Date:** 2025-01-11  
**Purpose:** Research on architecting Toka as both a standalone agent runtime and a tool for Cursor  
**Status:** Architecture Analysis and Recommendations

---

## Executive Summary

Toka OS is a sophisticated **deterministic, capability-based operating system for agentic AI systems** built in Rust. The codebase demonstrates a well-architected foundation that can be extended to serve dual purposes:

1. **Standalone Agent Runtime**: Independent orchestration of AI agents with LLM integration
2. **Cursor Integration Tool**: Extensible system that Cursor can invoke for agent-based tasks

This research provides architectural recommendations for achieving both use cases while maintaining security, determinism, and scalability.

---

## Current Architecture Analysis

### Core Components

The Toka ecosystem consists of several well-designed layers:

#### 1. **Kernel Layer (Deterministic Core)**
- **Location**: `crates/toka-kernel/`
- **Purpose**: Deterministic state machine executor with capability-based security
- **Key Features**:
  - Synchronous, deterministic execution (no async side-effects in handlers)
  - Capability-guarded syscall surface via `Operation` enum
  - Agent primitives: `SpawnSubAgent`, `ScheduleAgentTask`, `EmitObservation`
  - In-memory `WorldState` with agent task queues
  - Comprehensive security validation and audit logging

#### 2. **Runtime Layer (Coordination)**
- **Location**: `crates/toka-runtime/`
- **Purpose**: Bridges deterministic kernel with storage and external systems
- **Key Features**:
  - Pluggable storage backends (Memory, Sled, SQLite)
  - Event bus for real-time notifications
  - Authentication via JWT tokens
  - Lifecycle management of kernel instances

#### 3. **Orchestration Layer (Agent Coordination)**
- **Location**: `crates/toka-orchestration/`
- **Purpose**: Multi-agent coordination and dependency resolution
- **Key Features**:
  - YAML-based agent configuration system
  - Dependency resolution for agent spawning
  - Progress monitoring and workstream coordination
  - LLM integration for intelligent coordination
  - Phase-based orchestration (Critical → Foundation → Parallel Development)

#### 4. **LLM Gateway (AI Integration)**
- **Location**: `crates/toka-llm-gateway/`
- **Purpose**: Secure LLM provider integration
- **Key Features**:
  - Memory-safe secrets management
  - Support for Anthropic Claude and OpenAI GPT
  - Rate limiting and request sanitization
  - Response validation and token usage tracking

#### 5. **CLI Interface (Standalone Tool)**
- **Location**: `crates/toka-cli/`
- **Purpose**: Command-line interface for direct Toka interaction
- **Key Features**:
  - Task scheduling and agent spawning
  - State querying and daemon mode
  - JWT token generation for development
  - Multiple storage backend support

### Agent Configuration System

Agents are configured via YAML files with rich metadata:

```yaml
metadata:
  name: "agent-name"
  version: "v0.3.0"
  workstream: "workstream-name"
  branch: "feature/branch-name"
  
spec:
  name: "Human-readable agent name"
  domain: "domain-name"
  priority: "critical|high|medium|low"
  
capabilities:
  primary: ["capability1", "capability2"]
  secondary: ["capability3"]
  
objectives:
  - description: "Objective description"
    deliverable: "Expected deliverable"
    validation: "Validation criteria"
    
dependencies:
  required: {"agent-name": "reason"}
  optional: {"agent-name": "reason"}
  
security:
  sandbox: true
  capabilities_required: ["capability-name"]
  resource_limits:
    max_memory: "100MB"
    max_cpu: "50%"
    timeout: "1h"
```

### Current Agent Workflow

1. **Configuration Loading**: YAML configs loaded from `agents/v0.3.0/workstreams/`
2. **Dependency Resolution**: Agents spawned based on dependency graph
3. **Phase-Based Orchestration**: Critical → Foundation → Parallel Development
4. **Task Assignment**: Tasks assigned based on agent capabilities and priorities
5. **Progress Monitoring**: Event-driven progress tracking via `KernelEvent`s
6. **LLM Integration**: Intelligent coordination via LLM gateway

---

## Architectural Vision: Dual-Purpose Design

### Use Case 1: Standalone Agent Runtime

**Toka as Independent System**
- Complete agent lifecycle management
- Multi-agent orchestration with LLM intelligence
- Persistent state management
- Web API for external integration
- Monitoring and observability

### Use Case 2: Cursor Integration Tool

**Toka as Cursor Tool**
- Cursor can invoke Toka for specific agent tasks
- Lightweight agent execution for code assistance
- Integration with Cursor's existing workflow
- Minimal setup and configuration
- Fast startup and execution

---

## Recommended Architecture Changes

### 1. **Modular Runtime Architecture**

Create a flexible runtime system that can operate in different modes:

```rust
// New runtime modes
#[derive(Debug, Clone)]
pub enum RuntimeMode {
    /// Full standalone runtime with all features
    Standalone {
        web_server: bool,
        orchestration: bool,
        persistence: bool,
        llm_integration: bool,
    },
    /// Lightweight tool mode for external integration
    Tool {
        timeout: Duration,
        resource_limits: ResourceLimits,
    },
    /// Embedded mode for library usage
    Embedded {
        minimal_logging: bool,
        in_memory_only: bool,
    },
}

pub struct TokaRuntime {
    mode: RuntimeMode,
    kernel: Arc<Kernel>,
    orchestration: Option<Arc<OrchestrationEngine>>,
    web_server: Option<Arc<WebServer>>,
    // ... other components
}
```

### 2. **Plugin-Based Agent System**

Implement a plugin architecture for agents:

```rust
// Agent plugin trait
#[async_trait]
pub trait AgentPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    async fn execute(&self, context: &AgentContext, task: &TaskSpec) -> Result<TaskResult>;
    async fn validate(&self, task: &TaskSpec) -> Result<()>;
}

// Agent registry for dynamic loading
pub struct AgentRegistry {
    plugins: HashMap<String, Box<dyn AgentPlugin>>,
    builtin_agents: HashMap<String, BuiltinAgent>,
}
```

### 3. **API Gateway for External Integration**

Create a unified API that both Cursor and other tools can use:

```rust
// Unified Toka API
#[derive(Debug, Clone)]
pub struct TokaApi {
    runtime: Arc<TokaRuntime>,
}

impl TokaApi {
    // Simple agent execution for tools like Cursor
    pub async fn execute_agent_task(
        &self, 
        agent_name: &str, 
        task: &str,
        context: Option<HashMap<String, String>>
    ) -> Result<AgentResult> {
        // Simplified execution path for external tools
    }
    
    // Full orchestration for standalone mode
    pub async fn start_orchestration(
        &self,
        config: OrchestrationConfig
    ) -> Result<OrchestrationSession> {
        // Full orchestration workflow
    }
    
    // State queries for both modes
    pub async fn query_state(&self) -> Result<SystemState> {
        // Unified state querying
    }
}
```

### 4. **Configuration Abstraction**

Create different configuration levels:

```rust
// Simplified config for tool mode
#[derive(Debug, Clone)]
pub struct ToolConfig {
    pub timeout: Duration,
    pub max_memory: String,
    pub available_agents: Vec<String>,
    pub llm_provider: Option<LlmProvider>,
}

// Full config for standalone mode
#[derive(Debug, Clone)]
pub struct StandaloneConfig {
    pub orchestration: OrchestrationConfig,
    pub web_server: WebServerConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

// Unified config enum
#[derive(Debug, Clone)]
pub enum TokaConfig {
    Tool(ToolConfig),
    Standalone(StandaloneConfig),
    Embedded(EmbeddedConfig),
}
```

---

## Implementation Roadmap

### Phase 1: Core Architecture Refactoring (Weeks 1-2)

**Objective**: Establish the foundation for dual-purpose architecture

**Tasks**:
1. **Runtime Mode Abstraction**
   - Create `RuntimeMode` enum and configuration system
   - Implement conditional component initialization
   - Add mode-specific feature flags

2. **API Gateway Layer**
   - Design unified `TokaApi` interface
   - Implement simplified execution paths for tool mode
   - Add proper error handling and timeouts

3. **Agent Plugin System**
   - Define `AgentPlugin` trait and registry
   - Convert existing agents to plugin architecture
   - Add dynamic agent loading capabilities

### Phase 2: Tool Mode Implementation (Weeks 3-4)

**Objective**: Implement lightweight tool mode for Cursor integration

**Tasks**:
1. **Lightweight Runtime**
   - Implement fast startup tool mode
   - Add minimal configuration options
   - Optimize for single-task execution

2. **Cursor Integration Layer**
   - Create Cursor-specific API endpoints
   - Implement task execution with context passing
   - Add result formatting for Cursor consumption

3. **Agent Task Templates**
   - Create common agent task templates
   - Implement code analysis agents
   - Add documentation generation agents

### Phase 3: Standalone Mode Enhancement (Weeks 5-6)

**Objective**: Enhance standalone mode with web interface and advanced features

**Tasks**:
1. **Web Server Integration**
   - Add REST API server using Axum
   - Implement WebSocket for real-time updates
   - Create admin interface for orchestration

2. **Enhanced Orchestration**
   - Add workflow visualization
   - Implement advanced dependency resolution
   - Add real-time progress monitoring

3. **Monitoring and Observability**
   - Add metrics collection
   - Implement logging aggregation
   - Create dashboard for system health

### Phase 4: Integration and Testing (Weeks 7-8)

**Objective**: Integrate both modes and ensure seamless operation

**Tasks**:
1. **Integration Testing**
   - Test tool mode with Cursor
   - Validate standalone mode orchestration
   - Performance testing and optimization

2. **Documentation and Examples**
   - Create Cursor integration guide
   - Document standalone deployment
   - Add example agent configurations

3. **Security Hardening**
   - Security audit of both modes
   - Implement additional safeguards
   - Add compliance features

---

## Cursor Integration Strategy

### Integration Points

1. **Command-Line Interface**
   ```bash
   # Cursor can invoke Toka via CLI
   toka execute --agent code-analyzer --task "analyze function complexity" --context ./src/main.rs
   ```

2. **HTTP API**
   ```http
   POST /api/v1/execute
   {
     "agent": "code-analyzer",
     "task": "analyze function complexity",
     "context": {
       "file_path": "./src/main.rs",
       "language": "rust"
     }
   }
   ```

3. **Library Integration**
   ```rust
   // Cursor can embed Toka as a library
   use toka_api::TokaApi;
   
   let toka = TokaApi::new(ToolConfig::default()).await?;
   let result = toka.execute_agent_task(
       "code-analyzer",
       "analyze function complexity",
       Some(context)
   ).await?;
   ```

### Recommended Cursor Agents

1. **Code Analysis Agent**
   - Function complexity analysis
   - Code smell detection
   - Architecture review

2. **Documentation Agent**
   - API documentation generation
   - README creation
   - Code comment generation

3. **Testing Agent**
   - Unit test generation
   - Integration test creation
   - Test coverage analysis

4. **Refactoring Agent**
   - Code structure improvement
   - Performance optimization
   - Security vulnerability fixes

---

## Security Considerations

### Capability-Based Security

The existing capability-based security model provides excellent foundation:

1. **Token-Based Authentication**
   - JWT tokens with capability claims
   - Fine-grained permission system
   - Audit trail for all operations

2. **Sandboxed Execution**
   - Resource limits per agent
   - Isolated execution environments
   - Fail-safe defaults

3. **Input Validation**
   - Request sanitization
   - Response validation
   - Injection attack prevention

### Additional Security for Dual-Purpose System

1. **Mode-Specific Security**
   - Stricter limits for tool mode
   - Enhanced monitoring for standalone mode
   - Separate security policies per mode

2. **External Integration Security**
   - API rate limiting
   - Authentication for external tools
   - Secure context passing

---

## Performance Considerations

### Tool Mode Optimization

1. **Fast Startup**
   - Lazy initialization of components
   - Cached agent loading
   - Minimal dependency resolution

2. **Memory Efficiency**
   - Shared kernel instances
   - Efficient task queuing
   - Garbage collection optimization

3. **Network Efficiency**
   - Connection pooling for LLM providers
   - Request batching
   - Response caching

### Standalone Mode Scalability

1. **Horizontal Scaling**
   - Multiple runtime instances
   - Load balancing
   - Shared state management

2. **Resource Management**
   - Dynamic resource allocation
   - Agent pool management
   - Automatic scaling

---

## Conclusion

Toka OS has a solid architectural foundation that can be extended to serve both as a standalone agent runtime and as a tool for Cursor. The recommended approach involves:

1. **Modular Runtime Architecture** with mode-specific initialization
2. **Unified API Layer** for consistent external integration
3. **Plugin-Based Agent System** for extensibility
4. **Comprehensive Security Model** maintaining capability-based access control

This dual-purpose design will allow Toka to:
- **Serve Cursor** as a lightweight, fast tool for code assistance
- **Operate Independently** as a full-featured agent orchestration system
- **Maintain Security** through capability-based access control
- **Scale Appropriately** for both use cases

The implementation can be achieved incrementally, starting with the core architecture refactoring and building up to full dual-mode operation. The existing codebase provides an excellent foundation, and the proposed changes are evolutionary rather than revolutionary, ensuring stability while adding the required flexibility.