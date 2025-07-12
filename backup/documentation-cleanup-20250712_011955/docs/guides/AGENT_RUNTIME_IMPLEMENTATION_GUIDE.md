# Toka Agent Runtime Implementation Guide

## Overview

The Toka Agent Runtime bridges the gap between agent orchestration and actual task execution. This implementation provides the missing execution layer that can interpret and execute agent configurations with LLM integration.

## Architecture

```text
Orchestration Engine → Runtime Integration → Agent Process Manager → Agent Executor → Task Executor → LLM Integration
                                        ↓
                                   Progress Reporting
```

### Core Components

1. **AgentExecutor** - Main execution loop that interprets agent configurations
2. **TaskExecutor** - LLM-integrated task execution with security validation  
3. **AgentProcessManager** - Process lifecycle management for spawned agents
4. **ProgressReporter** - Real-time progress updates to orchestration system
5. **CapabilityValidator** - Runtime permission checking against declared capabilities
6. **ResourceManager** - CPU, memory, and timeout enforcement

## Implementation Status

### ✅ Completed Components

#### Core Agent Runtime (`toka-agent-runtime`)
- **AgentExecutor**: Complete with workflow execution and state management
- **TaskExecutor**: LLM-integrated task execution with retry logic
- **ProgressReporter**: Progress tracking and orchestration communication
- **CapabilityValidator**: Security validation against declared capabilities
- **ResourceManager**: Resource limit enforcement and monitoring
- **AgentProcessManager**: Process lifecycle management

#### Integration Layer
- **RuntimeIntegration**: Connects orchestration with agent runtime
- **OrchestrationRuntimeExt**: Extension trait for orchestration engine

### 🔄 Integration Points

The agent runtime integrates with existing Toka components:

- **toka-orchestration**: Loads agent configurations and manages dependencies
- **toka-llm-gateway**: Provides LLM services for intelligent task execution
- **toka-runtime**: Provides kernel operations and message passing
- **toka-types**: Provides core types like EntityId and Message

## Usage Example

### 1. Basic Agent Execution

```rust
use std::sync::Arc;
use toka_agent_runtime::{AgentExecutor, AgentProcessManager};
use toka_orchestration::{AgentConfig, OrchestrationEngine, RuntimeIntegration};
use toka_llm_gateway::LlmGateway;
use toka_runtime::Runtime;
use toka_types::EntityId;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize core components
    let runtime = Arc::new(Runtime::new(Default::default(), Default::default()).await?);
    let llm_gateway = Arc::new(LlmGateway::new(/* config */).await?);
    let orchestration = Arc::new(OrchestrationEngine::new(config, runtime.clone()).await?);
    
    // Create runtime integration
    let integration = orchestration.create_runtime_integration(
        runtime.clone(),
        llm_gateway.clone(),
    );
    
    // Load agent configuration
    let agent_config = AgentConfig::load_from_file(
        "agents/v0.3.0/workstreams/build-system-stabilization.yaml"
    )?;
    
    // Start agent execution
    let agent_id = EntityId(42);
    let result = integration.start_agent_execution(agent_config, agent_id).await?;
    
    println!("Agent started: {:?}", result);
    
    // Monitor agent execution
    loop {
        integration.monitor_runtime().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
```

### 2. Advanced Process Management

```rust
use toka_agent_runtime::AgentProcessManager;

async fn manage_multiple_agents(
    process_manager: Arc<AgentProcessManager>,
    configs: Vec<AgentConfig>,
) -> anyhow::Result<()> {
    let mut agents = Vec::new();
    
    // Start multiple agents
    for (index, config) in configs.into_iter().enumerate() {
        let agent_id = EntityId(index as u64);
        let result = process_manager.start_agent(config, agent_id).await?;
        agents.push((agent_id, result));
        
        println!("Started agent {}: {:?}", index, result);
    }
    
    // Monitor all agents
    loop {
        let stats = process_manager.get_stats().await;
        println!("Runtime stats: {:?}", stats);
        
        let running = process_manager.get_running_agents();
        if running.is_empty() {
            println!("All agents completed");
            break;
        }
        
        // Check individual agent states
        for agent_id in &running {
            if let Some(state) = process_manager.get_agent_state(*agent_id).await {
                println!("Agent {:?} state: {:?}", agent_id, state);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    
    Ok(())
}
```

### 3. Custom Task Execution

```rust
use toka_agent_runtime::{AgentTask, TaskResult, AgentContext};
use async_trait::async_trait;
use std::time::Duration;

struct CustomTask {
    task_id: String,
    description: String,
}

#[async_trait]
impl AgentTask for CustomTask {
    async fn execute(&self, context: &AgentContext) -> anyhow::Result<TaskResult> {
        println!("Executing custom task: {}", self.description);
        
        // Perform custom logic here
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        Ok(TaskResult::success(
            self.task_id.clone(),
            self.description.clone(),
            Some("Custom task completed successfully".to_string()),
            Duration::from_secs(2),
        ))
    }
    
    fn task_id(&self) -> &str {
        &self.task_id
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn estimated_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs(5))
    }
}

// Usage in agent executor
async fn execute_custom_task(
    executor: &mut TaskExecutor,
    context: &AgentContext,
) -> anyhow::Result<()> {
    let custom_task = CustomTask {
        task_id: "custom-001".to_string(),
        description: "Custom processing task".to_string(),
    };
    
    let result = executor.execute_task(&custom_task, context).await?;
    println!("Task result: {:?}", result);
    
    Ok(())
}
```

## Agent Configuration Integration

The runtime works with existing agent configurations from `agents/v0.3.0/workstreams/`:

### Example: Build System Agent

```yaml
# agents/v0.3.0/workstreams/build-system-stabilization.yaml
metadata:
  name: "build-system-agent"
  version: "v0.3.0"
  created: "2025-07-12"
  workstream: "build-system-stabilization"
  branch: "feature/build-system-stabilization"

spec:
  name: "Build System Stabilization Agent"
  domain: "build-infrastructure"
  priority: "critical"

capabilities:
  primary:
    - "cargo-execution"
    - "dependency-management"
    - "build-optimization"
  secondary:
    - "testing-integration"
    - "ci-coordination"

tasks:
  default:
    - description: "Audit current build system performance and identify bottlenecks"
      priority: "high"
    - description: "Implement incremental build optimizations"
      priority: "high"
    - description: "Standardize dependency management across workspace"
      priority: "medium"

security:
  sandbox: true
  capabilities_required:
    - "filesystem-read"
    - "filesystem-write" 
    - "cargo-execution"
  resource_limits:
    max_memory: "200MB"
    max_cpu: "75%"
    timeout: "30m"
```

## Security Features

### Capability Validation

The runtime enforces security through capability validation:

```rust
use toka_agent_runtime::CapabilityValidator;

// Agent can only perform operations for which it has declared capabilities
let validator = CapabilityValidator::new(
    vec!["filesystem-read".to_string(), "cargo-execution".to_string()],
    security_config,
);

// This will succeed
assert!(validator.can_perform("filesystem-read")?);

// This will fail  
assert!(!validator.can_perform("network-access")?);
```

### Resource Management

```rust
use toka_agent_runtime::ResourceManager;

let mut manager = ResourceManager::new(resource_limits)?;

// Check before operation
if manager.would_exceed_memory(50_000_000) {
    return Err("Would exceed memory limit".into());
}

// Record usage after operation
manager.record_usage(100_tokens, Duration::from_secs(10))?;

let usage = manager.get_usage();
println!("Current usage: {}MB", usage.memory_mb());
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/agent-runtime-core
```

### 2. Implement Agent Runtime
- ✅ Core runtime components implemented
- ✅ Integration with orchestration
- ✅ Security and resource management

### 3. Integration Testing

Create integration tests to verify the runtime works with actual agent configurations:

```rust
#[tokio::test]
async fn test_build_system_agent_execution() {
    let config = AgentConfig::load_from_file(
        "agents/v0.3.0/workstreams/build-system-stabilization.yaml"
    ).unwrap();
    
    let runtime = create_test_runtime().await;
    let llm_gateway = create_test_llm_gateway().await;
    
    let executor = AgentExecutor::new(
        config,
        EntityId(1),
        runtime,
        llm_gateway,
    ).await.unwrap();
    
    let result = executor.run().await;
    assert!(result.is_ok());
}
```

### 4. Production Deployment

Once integration testing passes:

```bash
# Run full test suite
cargo test --workspace

# Deploy to production orchestration
cargo build --release
```

## Next Steps

### Immediate (Phase 1)
1. ✅ **Complete core agent runtime implementation**
2. ✅ **Integration with orchestration engine**
3. 🔄 **Basic integration testing with mock LLM**
4. 🔄 **Resource limit enforcement testing**

### Short-term (Phase 2) 
1. 🔄 **Integration testing with real agent configurations**
2. ⏳ **Performance optimization and resource usage profiling**
3. ⏳ **Enhanced error handling and recovery mechanisms**
4. ⏳ **Monitoring and observability improvements**

### Medium-term (Phase 3)
1. ⏳ **Advanced task scheduling and prioritization**
2. ⏳ **Cross-agent communication and coordination**
3. ⏳ **Persistent state management for long-running agents**
4. ⏳ **Advanced security sandboxing**

## Troubleshooting

### Common Issues

#### 1. Agent Fails to Start
```
Error: Agent failed to start: Failed to create agent executor
```

**Solution**: Check that all dependencies are properly configured:
- Runtime is initialized
- LLM gateway is accessible
- Agent configuration is valid

#### 2. Task Execution Timeout
```
Error: Task execution timeout: task-123 exceeded 5m0s
```

**Solution**: Either increase timeout in resource limits or optimize task execution:
```yaml
security:
  resource_limits:
    timeout: "10m"  # Increase timeout
```

#### 3. Capability Denied
```
Error: Capability not authorized: network-access required for download API data
```

**Solution**: Add required capability to agent configuration:
```yaml
security:
  capabilities_required:
    - "network-access"
```

## Monitoring and Observability

### Progress Tracking

The runtime provides comprehensive progress tracking:

```rust
// Monitor progress through the progress reporter
let progress = agent.get_progress().await;
println!("Agent progress: {:.1}% ({}/{})", 
         progress.progress * 100.0,
         progress.tasks_completed,
         progress.total_tasks);
```

### Metrics Collection

```rust
// Collect runtime statistics
let stats = process_manager.get_stats().await;
println!("Active agents: {}", stats.active_agents);
println!("Total LLM tokens used: {}", stats.total_llm_tokens);
println!("Average execution time: {:?}", stats.avg_agent_execution_time);
```

This implementation provides a complete agent execution runtime that transforms Toka from a coordination system into a fully functional agent execution platform.

## 🔗 Related Documentation

- **Quick Start**: [QUICK_START_TESTING.md](QUICK_START_TESTING.md)
- **Testing Guide**: [TOKA_TESTING_SETUP_GUIDE.md](TOKA_TESTING_SETUP_GUIDE.md)
- **Architecture**: [Architecture Overview](../architecture/README.md)
- **Main Documentation**: [Documentation Index](../README.md)

## 📚 See Also

- **Agent Research**: [Agent Implementation Research](../research/toka_agent_implementation_research_and_proposal.md)
- **Production Readiness**: [Production Readiness Report](../research/20250127_toka_production_readiness_report.md)
- **Implementation Roadmap**: [Implementation Roadmap](../reports/IMPLEMENTATION_ROADMAP.md)
- **Development Guide**: [Development Documentation](../development/README.md)