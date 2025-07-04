# Agent Orchestration Guide
**Version:** v0.3.0 – 2025-07-04  
**Purpose:** Guide for main agent to orchestrate domain-specific workstream agents  
**Compatibility:** Toka OS v0.3.0 Enhancement Roadmap

---

## Overview

This guide provides the main coordinating agent with instructions for spawning, managing, and coordinating the six domain-specific workstream agents for the v0.3.0 enhancement roadmap.

## Agent Spawning Sequence

### Phase 1: Critical Infrastructure (Week 1-2)

```rust
// 1. Spawn Build System Stabilization Agent (Critical Priority)
let build_agent_spec = AgentSpec::new("Build System Stabilization Agent".to_string())?;
let build_agent_msg = Message {
    origin: main_agent_id,
    capability: "agent-orchestration".to_string(),
    op: Operation::SpawnSubAgent { 
        parent: main_agent_id, 
        spec: build_agent_spec 
    },
};
let build_agent_event = runtime.submit(build_agent_msg).await?;

// 2. Wait for build system stabilization completion before proceeding
// Monitor build agent progress through kernel events
```

### Phase 2: Foundation Services (Week 2-3)

```rust
// Only proceed after build system is stable
if build_system_stable {
    // Spawn Testing Infrastructure Agent
    let testing_agent_spec = AgentSpec::new("Testing Infrastructure Expansion Agent".to_string())?;
    let testing_msg = Message {
        origin: main_agent_id,
        capability: "agent-orchestration".to_string(),
        op: Operation::SpawnSubAgent { 
            parent: main_agent_id, 
            spec: testing_agent_spec 
        },
    };
    let testing_event = runtime.submit(testing_msg).await?;
}
```

### Phase 3: Parallel Development (Week 3-6)

```rust
// Spawn remaining agents in parallel once foundation is ready
let agent_specs = vec![
    ("Kernel Event Model Enhancement Agent", "kernel-events-enhancement"),
    ("Storage Layer Advancement Agent", "storage-advancement"),
    ("Security Framework Extension Agent", "security-extension"),
    ("Performance & Observability Foundation Agent", "performance-observability"),
];

for (agent_name, agent_id) in agent_specs {
    let spec = AgentSpec::new(agent_name.to_string())?;
    let msg = Message {
        origin: main_agent_id,
        capability: "agent-orchestration".to_string(),
        op: Operation::SpawnSubAgent { 
            parent: main_agent_id, 
            spec 
        },
    };
    let event = runtime.submit(msg).await?;
    // Store agent_id for task assignment
}
```

## Task Assignment Strategy

### Priority-Based Task Distribution

```rust
fn assign_tasks_by_priority(agents: &HashMap<String, EntityId>, runtime: &Runtime) -> Result<()> {
    // Critical priority tasks (build system) first
    assign_critical_tasks(&agents["build-system-stabilization"], runtime).await?;
    
    // High priority tasks (testing, kernel events) once build is stable
    if build_system_ready() {
        assign_high_priority_tasks(&agents, runtime).await?;
    }
    
    // Medium priority tasks (storage, security, performance) in parallel
    if foundation_ready() {
        assign_medium_priority_tasks(&agents, runtime).await?;
    }
    
    Ok(())
}

async fn assign_critical_tasks(agent_id: &EntityId, runtime: &Runtime) -> Result<()> {
    let tasks = vec![
        "Analyze current base64ct dependency conflict in workspace",
        "Research base64ct edition2024 compatibility requirements",
        "Update Cargo.toml files to resolve dependency conflicts",
        "Test build across all workspace crates",
    ];
    
    for task_desc in tasks {
        let task = TaskSpec::new(task_desc.to_string())?;
        let msg = Message {
            origin: main_agent_id,
            capability: "task-assignment".to_string(),
            op: Operation::ScheduleAgentTask { 
                agent: *agent_id, 
                task 
            },
        };
        runtime.submit(msg).await?;
    }
    Ok(())
}
```

## Progress Monitoring

### Event-Based Progress Tracking

```rust
// Listen for agent progress events
async fn monitor_agent_progress(runtime: &Runtime) -> Result<()> {
    let mut event_receiver = runtime.subscribe_events().await?;
    
    while let Some(event) = event_receiver.recv().await {
        match event {
            KernelEvent::TaskCompleted { agent, task, .. } => {
                handle_task_completion(agent, task).await?;
            }
            KernelEvent::TaskFailed { agent, task, error, .. } => {
                handle_task_failure(agent, task, error).await?;
            }
            KernelEvent::AgentTerminated { agent, reason, .. } => {
                handle_agent_termination(agent, reason).await?;
            }
            _ => {} // Handle other events as needed
        }
    }
    Ok(())
}

async fn handle_task_completion(agent: EntityId, task: TaskSpec) -> Result<()> {
    // Update workstream progress
    // Check if agent can proceed to next phase
    // Notify dependent agents of completion
    Ok(())
}
```

### Dependency Resolution

```rust
struct WorkstreamDependencies {
    critical_path: Vec<String>,
    dependencies: HashMap<String, Vec<String>>,
}

impl WorkstreamDependencies {
    fn new() -> Self {
        let mut deps = HashMap::new();
        
        // Testing depends on build system
        deps.insert("testing-infrastructure".to_string(), 
                   vec!["build-system-stabilization".to_string()]);
        
        // All other workstreams depend on build system
        for workstream in ["kernel-events-enhancement", "storage-advancement", 
                          "security-extension", "performance-observability"] {
            deps.insert(workstream.to_string(), 
                       vec!["build-system-stabilization".to_string()]);
        }
        
        // Performance agent benefits from testing baseline
        deps.get_mut("performance-observability").unwrap()
           .push("testing-infrastructure".to_string());
        
        Self {
            critical_path: vec!["build-system-stabilization".to_string()],
            dependencies: deps,
        }
    }
    
    fn can_start_workstream(&self, workstream: &str, completed: &HashSet<String>) -> bool {
        if let Some(deps) = self.dependencies.get(workstream) {
            deps.iter().all(|dep| completed.contains(dep))
        } else {
            true // No dependencies
        }
    }
}
```

## Integration Points

### Cross-Agent Communication

```rust
// Agents report progress through observations
async fn handle_agent_observation(agent: EntityId, data: Vec<u8>) -> Result<()> {
    let observation: AgentObservation = serde_json::from_slice(&data)?;
    
    match observation.observation_type {
        ObservationType::Progress => {
            update_workstream_progress(agent, observation.data).await?;
        }
        ObservationType::Completion => {
            mark_workstream_complete(agent, observation.data).await?;
            notify_dependent_agents(agent).await?;
        }
        ObservationType::Issue => {
            handle_workstream_issue(agent, observation.data).await?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct AgentObservation {
    observation_type: ObservationType,
    timestamp: String,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
enum ObservationType {
    Progress,
    Completion,
    Issue,
    Metrics,
}
```

### Resource Management

```rust
struct ResourceManager {
    agent_allocations: HashMap<EntityId, ResourceAllocation>,
    total_resources: SystemResources,
}

#[derive(Clone)]
struct ResourceAllocation {
    max_memory: String,
    max_cpu: String,
    timeout: String,
    priority: AgentPriority,
}

impl ResourceManager {
    fn allocate_resources(&mut self, agent: EntityId, config: &AgentConfig) -> Result<()> {
        let allocation = ResourceAllocation {
            max_memory: config.security.resource_limits.max_memory.clone(),
            max_cpu: config.security.resource_limits.max_cpu.clone(),
            timeout: config.security.resource_limits.timeout.clone(),
            priority: config.spec.priority.into(),
        };
        
        // Validate resource availability
        if self.can_allocate(&allocation)? {
            self.agent_allocations.insert(agent, allocation);
            Ok(())
        } else {
            Err(ResourceError::InsufficientResources.into())
        }
    }
}
```

## Validation and Testing

### Configuration Validation

```bash
#!/bin/bash
# validate-agent-configs.sh

echo "Validating agent configurations..."

for config_file in agents/v0.3.0/workstreams/*.yaml; do
    echo "Validating $(basename "$config_file")..."
    
    # Validate YAML syntax
    if ! toka-config validate --file "$config_file"; then
        echo "❌ Syntax error in $config_file"
        exit 1
    fi
    
    # Validate required fields
    if ! toka-config read --file "$config_file" | jq -e '.metadata.name' > /dev/null; then
        echo "❌ Missing metadata.name in $config_file"
        exit 1
    fi
    
    echo "✅ $(basename "$config_file") is valid"
done

echo "✅ All agent configurations validated successfully"
```

### Integration Testing

```rust
#[cfg(test)]
mod orchestration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_spawning_sequence() -> Result<()> {
        let runtime = create_test_runtime().await?;
        let main_agent_id = EntityId(1);
        
        // Test build system agent spawning
        let build_spec = AgentSpec::new("Build System Test Agent".to_string())?;
        let spawn_msg = Message {
            origin: main_agent_id,
            capability: "test-orchestration".to_string(),
            op: Operation::SpawnSubAgent { 
                parent: main_agent_id, 
                spec: build_spec 
            },
        };
        
        let event = runtime.submit(spawn_msg).await?;
        assert!(matches!(event, KernelEvent::AgentSpawned { .. }));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dependency_resolution() -> Result<()> {
        let deps = WorkstreamDependencies::new();
        let mut completed = HashSet::new();
        
        // Initially, only build system can start
        assert!(deps.can_start_workstream("build-system-stabilization", &completed));
        assert!(!deps.can_start_workstream("testing-infrastructure", &completed));
        
        // After build system completes, testing can start
        completed.insert("build-system-stabilization".to_string());
        assert!(deps.can_start_workstream("testing-infrastructure", &completed));
        
        Ok(())
    }
}
```

## Error Handling and Recovery

### Agent Failure Recovery

```rust
async fn handle_agent_failure(failed_agent: EntityId, reason: String) -> Result<()> {
    // Determine criticality of failed agent
    let agent_info = get_agent_info(failed_agent).await?;
    
    match agent_info.priority {
        AgentPriority::Critical => {
            // Block all dependent agents
            suspend_dependent_agents(failed_agent).await?;
            // Attempt automatic recovery
            restart_critical_agent(failed_agent, agent_info).await?;
        }
        AgentPriority::High | AgentPriority::Medium => {
            // Log failure and continue with other agents
            log_agent_failure(failed_agent, reason).await?;
            // Reassign tasks if possible
            reassign_failed_tasks(failed_agent).await?;
        }
    }
    Ok(())
}
```

---

## Summary

This orchestration guide provides the main agent with:

1. **Structured spawning sequence** respecting dependencies
2. **Task assignment strategies** based on priorities and capabilities
3. **Progress monitoring** through event-driven architecture
4. **Resource management** ensuring efficient utilization
5. **Error handling** with graceful degradation
6. **Validation tools** for configuration and integration testing

The main agent should follow this guide to successfully coordinate the v0.3.0 enhancement roadmap implementation across all six workstreams. 