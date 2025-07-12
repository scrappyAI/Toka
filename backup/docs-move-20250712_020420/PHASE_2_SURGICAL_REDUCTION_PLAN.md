# Phase 2 Surgical Reduction Plan: Canonical Agent Architecture

**Date**: 2025-07-12 (UTC)  
**Status**: 🚀 ACTIVE  
**Focus**: Surgical reduction and canonical `toka-agents` crate  

## 🎯 Strategic Objectives

### **Primary Goal: Canonical Agent Architecture**
- **Eliminate**: Multiple runtime crates (`toka-runtime`, `toka-agent-runtime`, `toka-orchestration-service`)
- **Create**: Single canonical `toka-agents` crate that handles all agent functionality
- **Consolidate**: Orchestration overlap into unified system
- **Enable**: Wire agents directly into Toka for real testing

### **Key Principle: Surgical Reduction**
> Remove duplication through strategic consolidation, not feature removal

## 📊 Current State Analysis

### **Runtime Crate Duplication**
```
CURRENT (Fragmented):
├── toka-runtime              ← Dynamic code execution
├── toka-agent-runtime        ← Agent execution runtime  
├── toka-orchestration        ← Agent coordination
├── toka-orchestration-service ← Orchestration binary
└── toka-performance          ← Performance monitoring

TARGET (Canonical):
├── toka-runtime              ← Unified execution (code + agents)
├── toka-agents              ← CANONICAL agent system
└── toka-orchestration        ← Simplified coordination
```

### **Orchestration Overlap Analysis**
```
OVERLAPPING FUNCTIONALITY:
├── Agent lifecycle management (orchestration + agent-runtime)
├── Task execution coordination (orchestration + agent-runtime)  
├── Progress reporting (orchestration + agent-runtime)
├── Resource management (orchestration + agent-runtime)
└── LLM integration (orchestration + agent-runtime)
```

## 🔧 Implementation Strategy

### **Phase 2.1: Create Canonical `toka-agents` Crate**

#### **Architecture: Unified Agent System**
```rust
// NEW: crates/toka-agents/src/lib.rs
pub mod agent;          // Core agent abstraction
pub mod executor;       // Agent execution engine
pub mod orchestration;  // Agent orchestration
pub mod runtime;        // Runtime integration
pub mod lifecycle;      // Agent lifecycle management
pub mod coordination;   // Multi-agent coordination
pub mod capabilities;   // Capability management
pub mod resources;      // Resource management
pub mod progress;       // Progress tracking
pub mod integration;    // LLM and external integration

// Unified Agent API
pub struct Agent {
    pub id: EntityId,
    pub config: AgentConfig,
    pub executor: AgentExecutor,
    pub capabilities: CapabilitySet,
    pub resources: ResourceManager,
    pub state: AgentState,
}

// Unified Agent Manager
pub struct AgentManager {
    agents: Arc<DashMap<EntityId, Agent>>,
    orchestrator: AgentOrchestrator,
    runtime: Arc<RuntimeManager>,
    coordination: CoordinationEngine,
}
```

#### **Key Components to Consolidate**
1. **Agent Execution** (from `toka-agent-runtime`)
2. **Agent Orchestration** (from `toka-orchestration`)
3. **Agent Lifecycle** (from both)
4. **Resource Management** (from both)
5. **Progress Tracking** (from both)

### **Phase 2.2: Eliminate Runtime Duplication**

#### **Before: Multiple Runtime Systems**
```
toka-runtime:
├── Dynamic code execution
├── Sandbox management
├── Security enforcement
└── Resource monitoring

toka-agent-runtime:
├── Agent execution
├── Task coordination
├── Progress reporting
└── LLM integration
```

#### **After: Unified Runtime System**
```rust
// ENHANCED: crates/toka-runtime/src/lib.rs
pub enum ExecutionTarget {
    DynamicCode { code_type: CodeType, code: String },
    Agent { agent_id: EntityId, task: TaskSpec },
    Tool { tool_name: String, args: JsonValue },
}

pub struct RuntimeManager {
    // Unified execution engine
    execution_engine: UnifiedExecutionEngine,
    // Agent management integration
    agent_manager: Arc<AgentManager>,
    // Security and sandboxing
    security_manager: SecurityManager,
    // Resource tracking
    resource_tracker: ResourceTracker,
}
```

### **Phase 2.3: Simplify Orchestration**

#### **Remove Orchestration Service Duplication**
```
BEFORE:
├── toka-orchestration (crate)
└── toka-orchestration-service (binary)

AFTER:
├── toka-agents (handles orchestration)
└── toka-cli (single binary interface)
```

#### **Orchestration Integration**
```rust
// SIMPLIFIED: Agent orchestration becomes part of toka-agents
impl AgentManager {
    pub async fn orchestrate_agents(&self, config: OrchestrationConfig) -> Result<OrchestrationSession> {
        // Unified orchestration within canonical agent system
        let session = self.orchestrator.create_session(config).await?;
        self.coordination.coordinate_agents(session).await
    }
}
```

## 🛠️ Technical Implementation

### **Step 1: Create `toka-agents` Crate**

```toml
# NEW: crates/toka-agents/Cargo.toml
[package]
name = "toka-agents"
version = "0.2.1"
edition = "2021"
description = "Canonical agent system for Toka OS - unified agent execution, orchestration, and coordination"

[dependencies]
# Core Toka dependencies
toka-types = { path = "../toka-types" }
toka-kernel = { path = "../toka-kernel" }
toka-runtime = { path = "../toka-runtime" }
toka-bus-core = { path = "../toka-bus-core" }
toka-llm-gateway = { path = "../toka-llm-gateway" }

# Async and utilities
tokio = { workspace = true }
futures = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
dashmap = "5.5"
```

### **Step 2: Migrate Core Components**

#### **From `toka-agent-runtime`**
- `AgentExecutor` → `toka-agents::executor::AgentExecutor`
- `TaskExecutor` → `toka-agents::executor::TaskExecutor`
- `ProgressReporter` → `toka-agents::progress::ProgressReporter`
- `ResourceManager` → `toka-agents::resources::ResourceManager`
- `CapabilityValidator` → `toka-agents::capabilities::CapabilityValidator`

#### **From `toka-orchestration`**
- `OrchestrationEngine` → `toka-agents::orchestration::AgentOrchestrator`
- `DependencyResolver` → `toka-agents::coordination::DependencyResolver`
- `ProgressMonitor` → `toka-agents::progress::ProgressMonitor`
- `WorkstreamCoordinator` → `toka-agents::coordination::WorkstreamCoordinator`

### **Step 3: Update Runtime Integration**

```rust
// ENHANCED: crates/toka-runtime/src/lib.rs
impl RuntimeManager {
    pub async fn execute_agent(&self, agent_id: EntityId, task: TaskSpec) -> Result<ExecutionResult> {
        // Direct integration with canonical agent system
        let agent = self.agent_manager.get_agent(&agent_id).await?;
        let result = agent.execute_task(task).await?;
        
        // Unified result format
        ExecutionResult {
            success: result.success,
            output: result.output,
            agent_results: Some(result.into()),
            metadata: self.collect_metadata().await?,
            // ... other fields
        }
    }
}
```

### **Step 4: Simplify CLI Interface**

```rust
// ENHANCED: crates/toka-cli/src/main.rs
#[derive(Parser)]
#[command(name = "toka")]
enum TokaCommand {
    /// Agent management
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Runtime operations
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommand,
    },
    /// Orchestration operations
    Orchestrate {
        config: PathBuf,
        #[arg(long)]
        watch: bool,
    },
}

#[derive(Parser)]
enum AgentCommand {
    /// List all agents
    List,
    /// Spawn an agent
    Spawn { config: PathBuf },
    /// Monitor agent progress
    Monitor { agent_id: String },
    /// Stop an agent
    Stop { agent_id: String },
}
```

## 📈 Benefits of Canonical Architecture

### **Immediate Benefits**
1. **Single Source of Truth**: All agent functionality in one place
2. **Eliminate Duplication**: No more overlapping orchestration/runtime code
3. **Simplified Dependencies**: Clear dependency graph
4. **Unified API**: Consistent interface for all agent operations
5. **Better Testing**: Single crate to test all agent functionality

### **Strategic Benefits**
1. **Real Agent Testing**: Can wire agents directly into Toka
2. **Simplified Deployment**: Single agent system to deploy
3. **Easier Maintenance**: One codebase for all agent concerns
4. **Better Performance**: Eliminate inter-crate communication overhead
5. **Cleaner Architecture**: Clear separation of concerns

## 🚀 Implementation Timeline

### **Week 1: Foundation**
- [x] Phase 1 complete (runtime consolidation)
- [ ] Create `toka-agents` crate structure
- [ ] Migrate core agent types and interfaces
- [ ] Update dependency graph

### **Week 2: Core Migration**
- [ ] Migrate `AgentExecutor` and `TaskExecutor`
- [ ] Migrate orchestration components
- [ ] Implement unified agent API
- [ ] Update runtime integration

### **Week 3: Integration & Testing**
- [ ] Wire agents into Toka runtime
- [ ] Implement real agent testing
- [ ] Update CLI interface
- [ ] Integration testing

### **Week 4: Cleanup & Optimization**
- [ ] Remove deprecated crates
- [ ] Optimize performance
- [ ] Documentation updates
- [ ] Final testing and validation

## 🎯 Success Metrics

### **Architecture Goals**
- [ ] Single `toka-agents` crate handling all agent functionality
- [ ] Eliminated runtime duplication
- [ ] Simplified orchestration system
- [ ] Clear dependency graph with no circular dependencies

### **Functionality Goals**
- [ ] Agents can be wired directly into Toka
- [ ] Real agent testing and management
- [ ] Unified CLI interface
- [ ] Performance equal or better than current system

### **Development Goals**
- [ ] Reduced codebase complexity
- [ ] Easier maintenance and development
- [ ] Clear API boundaries
- [ ] Comprehensive test coverage

## 🔄 Next Steps

1. **Immediate**: Create `toka-agents` crate with basic structure
2. **Short-term**: Migrate core components and update integrations
3. **Medium-term**: Wire agents into Toka and enable real testing
4. **Long-term**: Optimize and refine canonical architecture

This surgical reduction approach maintains all existing functionality while creating a clean, canonical agent architecture that enables real agent testing and management in Toka OS.