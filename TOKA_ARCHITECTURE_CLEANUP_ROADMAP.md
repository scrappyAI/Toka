# Toka Architecture Cleanup & Consolidation Roadmap

**Date**: 2025-01-15  
**Status**: Architecture Analysis Complete  
**Scope**: Core workspace cleanup, tool consolidation, and deployment readiness

## Executive Summary

The Toka codebase has evolved into a sophisticated but fragmented system with **scattered tools, runtime duplication, and unclear boundaries**. This roadmap provides a phased approach to:

1. **Eliminate runtime duplication** between `toka-runtime` and `toka-agent-runtime`
2. **Consolidate tool system** into a unified, injectable registry
3. **Standardize cross-language tools** with proper security boundaries
4. **Focus on core deployment-ready components**

## Current State Analysis

### 🔴 Critical Issues

#### 1. **Runtime Duplication**
```
toka-runtime          ← Dynamic code execution, sandboxing
toka-agent-runtime    ← Agent execution, LLM integration
```
**Problem**: Overlapping concerns, unclear boundaries, duplicated execution models

#### 2. **Tool System Fragmentation**
```
crates/toka-tools/           ← Rust tools (file ops, validation)
scripts/                     ← Bash scripts (setup, testing, workflows)
.cursor/version-manager.py   ← Python version management
```
**Problem**: No central registry, no runtime injection, inconsistent interfaces

#### 3. **Architecture Boundaries**
```
Core Crates: 27 total
├── Essential: 8 (kernel, types, auth, runtime, etc.)
├── Extensions: 12 (storage, security, orchestration)
└── Fragmented: 7 (tools, demos, duplicates)
```
**Problem**: Unclear core vs extension boundaries, some non-essential complexity

## Consolidation Strategy

### Phase 1: Runtime Consolidation (Week 1-2)

#### **1.1 Merge Runtime Concepts**
**Target**: Single `toka-runtime` crate with clear execution models

```rust
// NEW: Unified Runtime Architecture
pub struct RuntimeManager {
    // Core execution engine (from current toka-runtime)
    code_executor: CodeExecutor,
    // Agent execution engine (from toka-agent-runtime)
    agent_executor: AgentExecutor,
    // Unified tool registry
    tool_registry: ToolRegistry,
    // Security context
    security_context: SecurityContext,
}

// Execution Models
pub enum ExecutionModel {
    // Dynamic code execution (Python, WASM, etc.)
    DynamicCode { code_type: CodeType, sandbox: SandboxConfig },
    // Agent workflow execution
    AgentWorkflow { agent_config: AgentConfig, llm_integration: bool },
    // Tool execution (registered tools)
    ToolExecution { tool_name: String, params: ToolParams },
}
```

**Actions**:
- [ ] Merge `toka-agent-runtime` into `toka-runtime` as `runtime::agents` module
- [ ] Create unified `ExecutionModel` enum
- [ ] Consolidate security and resource management
- [ ] Update all references and imports

#### **1.2 Eliminate Circular Dependencies**
**Current Issue**: `toka-orchestration ←→ toka-agent-runtime`

**Solution**: Move to event-driven architecture
```rust
// NEW: Event-driven coordination
pub struct OrchestrationEngine {
    event_bus: Arc<EventBus>,
    runtime_manager: Arc<RuntimeManager>,
}

// Events instead of direct calls
#[derive(Event)]
pub enum OrchestrationEvent {
    SpawnAgent { config: AgentConfig, id: EntityId },
    AgentProgress { id: EntityId, progress: f64 },
    AgentComplete { id: EntityId, result: TaskResult },
}
```

### Phase 2: Tool System Unification (Week 2-3)

#### **2.1 Unified Tool Registry**
**Target**: Single source of truth for all tools

```rust
// NEW: Unified Tool System
pub struct ToolRegistry {
    // Native Rust tools
    native_tools: HashMap<String, Arc<dyn Tool>>,
    // External tools (Python, shell, etc.)
    external_tools: HashMap<String, ExternalTool>,
    // Tool manifests and metadata
    manifests: HashMap<String, ToolManifest>,
    // Security and capability validator
    security_validator: Arc<CapabilityValidator>,
}

#[derive(Serialize, Deserialize)]
pub struct ToolManifest {
    pub name: String,
    pub version: String,
    pub tool_type: ToolType,
    pub executable: ExecutableSpec,
    pub security: SecuritySpec,
    pub parameters: Vec<ParameterSpec>,
    pub capabilities: Vec<String>,
}

pub enum ToolType {
    Native,           // Rust implementation
    Python(PathBuf),  // Python script
    Shell(PathBuf),   // Shell script
    External(PathBuf), // Other executable
}
```

#### **2.2 Tool Consolidation Plan**

**Essential Tools to Consolidate**:
```yaml
# Current: crates/toka-tools/src/tools/
native_tools:
  - file-reader
  - file-writer
  - file-lister
  - date-validator
  - build-validator

# Current: scripts/
external_tools:
  - setup-tools:
      - setup_toka_testing.sh
      - setup-docker-environments.sh
  - validation-tools:
      - validate-env.sh
      - validate-links.sh
      - validate_dates.py
  - workflow-tools:
      - toka_workflow.sh
      - toka_interactive.sh
  - monitoring-tools:
      - raft_monitoring_service.sh

# Current: .cursor/
version_management:
  - version-manager.py
```

**Consolidation Actions**:
- [ ] Create unified tool manifests for all tools
- [ ] Implement `ExternalTool` wrapper for scripts
- [ ] Create tool discovery system
- [ ] Implement secure tool execution sandbox
- [ ] Add runtime tool injection mechanism

#### **2.3 Security Framework for Tools**
```rust
// NEW: Tool Security Framework
pub struct ToolSecurityContext {
    pub agent_id: EntityId,
    pub capabilities: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub sandbox_config: SandboxConfig,
}

pub struct SecureToolExecutor {
    security_validator: Arc<CapabilityValidator>,
    sandbox_manager: Arc<SandboxManager>,
}

impl SecureToolExecutor {
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: &ToolParams,
        context: &ToolSecurityContext,
    ) -> Result<ToolResult> {
        // 1. Validate capabilities
        self.security_validator.validate_tool_access(tool_name, &context.capabilities)?;
        
        // 2. Create secure sandbox
        let sandbox = self.sandbox_manager.create_sandbox(&context.sandbox_config)?;
        
        // 3. Execute with resource limits
        let result = sandbox.execute_with_limits(tool_name, params, &context.resource_limits).await?;
        
        // 4. Audit and return
        self.audit_tool_execution(tool_name, &result, context);
        Ok(result)
    }
}
```

### Phase 3: Core Crate Focus (Week 3-4)

#### **3.1 Essential Core Crates**
**Keep as separate crates** (well-architected):
```
Foundation Layer:
├── toka-types          ← Core types and traits
├── toka-auth           ← Authentication and authorization
├── toka-bus-core       ← Event bus system
└── toka-kernel         ← Deterministic state machine

Storage Layer:
├── toka-store-core     ← Storage abstractions
├── toka-store-memory   ← In-memory storage
├── toka-store-sqlite   ← SQLite storage
└── toka-store-sled     ← Sled storage

Runtime Layer:
├── toka-runtime        ← UNIFIED runtime (agents + code + tools)
├── toka-orchestration  ← Agent coordination
└── toka-llm-gateway    ← LLM integration

Tools Layer:
└── toka-tools          ← UNIFIED tool system
```

#### **3.2 Consolidate/Remove**
**Merge into core crates**:
```
MERGE:
├── toka-agent-runtime  → toka-runtime::agents
├── toka-collaborative-auth → toka-auth::collaborative
└── toka-orchestration-service → toka-orchestration::service

REMOVE (non-essential):
├── toka-demo-environment (move to examples/)
├── toka-testing (merge into individual crate tests)
└── toka-rule-metadata (move to toka-tools)
```

#### **3.3 Security Crate Consolidation**
**Current**: 7 separate security crates  
**Target**: 3 focused security crates

```
NEW Security Architecture:
├── toka-capability-core    ← Core capability system
├── toka-security-services  ← Rate limiting, key rotation, revocation
└── toka-security-validation ← JWT, delegation, CVM
```

### Phase 4: Deployment Readiness (Week 4-5)

#### **4.1 Build System Stabilization**
```bash
# NEW: Simplified build targets
cargo build --release --bin toka-orchestration  # Main service
cargo build --release --bin toka-cli           # CLI tool
cargo build --release --bin toka-config        # Config management

# Tool registry initialization
./target/release/toka-cli tools register --discover
./target/release/toka-cli tools list
```

#### **4.2 Container Deployment**
```dockerfile
# NEW: Unified deployment container
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/toka-orchestration /usr/local/bin/
COPY --from=builder /app/target/release/toka-cli /usr/local/bin/
COPY --from=builder /app/scripts/ /app/scripts/
COPY --from=builder /app/crates/toka-tools/manifests/ /app/tools/

# Initialize tool registry
RUN /usr/local/bin/toka-cli tools register --discover /app/tools/

CMD ["toka-orchestration"]
```

#### **4.3 Tool Injection at Runtime**
```rust
// NEW: Runtime tool injection
pub struct AgentExecutor {
    tool_registry: Arc<ToolRegistry>,
    security_context: SecurityContext,
}

impl AgentExecutor {
    pub async fn execute_agent_task(&self, task: &AgentTask) -> Result<TaskResult> {
        // Agent can compose and execute tools on the fly
        let tools_needed = self.analyze_task_requirements(task);
        
        for tool_name in tools_needed {
            let tool_result = self.tool_registry.execute_tool(
                &tool_name,
                &task.params,
                &self.security_context,
            ).await?;
            
            // Agent can combine tool results
            self.integrate_tool_result(tool_result);
        }
        
        Ok(TaskResult::Success)
    }
}
```

## Implementation Timeline

### Week 1: Foundation
- [x] Analyze current state
- [ ] Create consolidation plan
- [ ] Begin runtime merger

### Week 2: Core Consolidation
- [ ] Complete runtime unification
- [ ] Create unified tool registry
- [ ] Implement tool manifests

### Week 3: Tool Integration
- [ ] Migrate all scripts to tool registry
- [ ] Implement secure tool execution
- [ ] Add runtime tool injection

### Week 4: Deployment Prep
- [ ] Consolidate security crates
- [ ] Simplify build system
- [ ] Create deployment containers

### Week 5: Testing & Validation
- [ ] End-to-end integration tests
- [ ] Performance benchmarks
- [ ] Security validation

## Success Metrics

### Architecture Goals
- [ ] Single runtime for all execution models
- [ ] All tools registered in unified registry
- [ ] Clear core vs extension boundaries
- [ ] No circular dependencies

### Deployment Goals
- [ ] Single command deployment
- [ ] Runtime tool injection working
- [ ] Security boundaries enforced
- [ ] Performance targets met

### Developer Experience
- [ ] Clear API boundaries
- [ ] Consistent tool interfaces
- [ ] Good error messages
- [ ] Comprehensive documentation

## Next Steps

1. **Immediate (This Week)**:
   - Review and approve this roadmap
   - Begin runtime consolidation
   - Set up dedicated cleanup branch

2. **Short-term (Next 2 Weeks)**:
   - Complete runtime unification
   - Implement unified tool registry
   - Create tool manifests

3. **Medium-term (Next Month)**:
   - Deploy first consolidated version
   - Validate runtime tool injection
   - Performance optimization

This roadmap provides a clear path to a clean, focused architecture ready for production deployment while maintaining the sophisticated capabilities you've built.