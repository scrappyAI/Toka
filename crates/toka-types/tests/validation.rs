use toka_types::{
    AgentConfig, AgentCapabilities, AgentDependencies, AgentMetadata, AgentObjective,
    AgentPriority, AgentSpec, AgentSpecConfig, AgentTasks, EntityId, Message, Operation,
    ReportingConfig, ReportingFrequency, ResourceLimits, SecurityConfig, TaskConfig,
    TaskPriority, TaskSpec, MAX_AGENT_NAME_LEN, MAX_CAPABILITY_TOKEN_LEN, MAX_TASK_DESCRIPTION_LEN,
};

#[test]
fn test_task_spec_validation() {
    // Valid task spec
    let valid_task = TaskSpec::new("Valid task description".to_string()).unwrap();
    assert_eq!(valid_task.description, "Valid task description");
    assert!(valid_task.validate().is_ok());

    // Empty description should fail
    assert!(TaskSpec::new("".to_string()).is_err());
    assert!(TaskSpec::new("   ".to_string()).is_err());

    // Too long description should fail
    let long_desc = "x".repeat(MAX_TASK_DESCRIPTION_LEN + 1);
    assert!(TaskSpec::new(long_desc).is_err());

    // Boundary case - exactly at limit should work
    let boundary_desc = "x".repeat(MAX_TASK_DESCRIPTION_LEN);
    assert!(TaskSpec::new(boundary_desc).is_ok());
}

#[test]
fn test_agent_spec_validation() {
    // Valid agent spec
    let valid_agent = AgentSpec::new("valid-agent".to_string()).unwrap();
    assert_eq!(valid_agent.name, "valid-agent");
    assert!(valid_agent.validate().is_ok());

    // Empty name should fail
    assert!(AgentSpec::new("".to_string()).is_err());
    assert!(AgentSpec::new("   ".to_string()).is_err());

    // Too long name should fail
    let long_name = "x".repeat(MAX_AGENT_NAME_LEN + 1);
    assert!(AgentSpec::new(long_name).is_err());

    // Boundary case - exactly at limit should work
    let boundary_name = "x".repeat(MAX_AGENT_NAME_LEN);
    assert!(AgentSpec::new(boundary_name).is_ok());
}

#[test]
fn test_message_validation() {
    let origin = EntityId(1);
    let task = TaskSpec::new("test task".to_string()).unwrap();
    let op = Operation::ScheduleAgentTask {
        agent: EntityId(2),
        task,
    };

    // Valid message
    let valid_capability = "valid-capability-token".to_string();
    let valid_message = Message::new(origin, valid_capability.clone(), op.clone()).unwrap();
    assert!(valid_message.validate().is_ok());

    // Empty capability should fail
    assert!(Message::new(origin, "".to_string(), op.clone()).is_err());
    assert!(Message::new(origin, "   ".to_string(), op.clone()).is_err());

    // Too long capability should fail
    let long_capability = "x".repeat(MAX_CAPABILITY_TOKEN_LEN + 1);
    assert!(Message::new(origin, long_capability, op.clone()).is_err());

    // Boundary case - exactly at limit should work
    let boundary_capability = "x".repeat(MAX_CAPABILITY_TOKEN_LEN);
    assert!(Message::new(origin, boundary_capability, op).is_ok());
}

#[test]
fn test_operation_validation() {
    let task = TaskSpec::new("test task".to_string()).unwrap();
    let agent_spec = AgentSpec::new("test-agent".to_string()).unwrap();

    // Valid operations
    let schedule_op = Operation::ScheduleAgentTask {
        agent: EntityId(1),
        task: task.clone(),
    };
    assert!(schedule_op.validate().is_ok());

    let spawn_op = Operation::SpawnSubAgent {
        parent: EntityId(1),
        spec: agent_spec.clone(),
    };
    assert!(spawn_op.validate().is_ok());

    let observation_op = Operation::EmitObservation {
        agent: EntityId(1),
        data: vec![1, 2, 3],
    };
    assert!(observation_op.validate().is_ok());
}

#[test]
fn test_agent_config_creation() {
    let metadata = AgentMetadata {
        name: "test-agent".to_string(),
        version: "1.0.0".to_string(),
        created: "2024-01-01".to_string(),
        workstream: "test".to_string(),
        branch: "main".to_string(),
    };

    let spec = AgentSpecConfig {
        name: "Test Agent".to_string(),
        domain: "testing".to_string(),
        priority: AgentPriority::High,
    };

    let capabilities = AgentCapabilities {
        primary: vec!["testing".to_string()],
        secondary: vec!["debugging".to_string()],
    };

    let objectives = vec![AgentObjective {
        description: "Test the system".to_string(),
        deliverable: "Test report".to_string(),
        validation: "All tests pass".to_string(),
    }];

    let tasks = AgentTasks {
        default: vec![TaskConfig {
            description: "Run tests".to_string(),
            priority: TaskPriority::High,
        }],
    };

    let dependencies = AgentDependencies {
        required: std::collections::HashMap::new(),
        optional: std::collections::HashMap::new(),
    };

    let reporting = ReportingConfig {
        frequency: ReportingFrequency::Daily,
        channels: vec!["console".to_string()],
        metrics: std::collections::HashMap::new(),
    };

    let security = SecurityConfig {
        sandbox: true,
        capabilities_required: vec!["test".to_string()],
        resource_limits: ResourceLimits {
            max_memory: "100MB".to_string(),
            max_cpu: "50%".to_string(),
            timeout: "1h".to_string(),
        },
    };

    let config = AgentConfig {
        metadata,
        spec,
        capabilities,
        objectives,
        tasks,
        dependencies,
        reporting,
        security,
    };

    // Verify all fields are accessible
    assert_eq!(config.metadata.name, "test-agent");
    assert_eq!(config.spec.name, "Test Agent");
    assert_eq!(config.capabilities.primary.len(), 1);
    assert_eq!(config.objectives.len(), 1);
    assert_eq!(config.tasks.default.len(), 1);
    assert_eq!(config.reporting.frequency, ReportingFrequency::Daily);
    assert!(config.security.sandbox);
}

#[test]
fn test_priority_enums() {
    // Test AgentPriority
    assert_ne!(AgentPriority::Critical, AgentPriority::High);
    assert_ne!(AgentPriority::High, AgentPriority::Medium);
    assert_ne!(AgentPriority::Medium, AgentPriority::Low);

    // Test TaskPriority
    assert_ne!(TaskPriority::High, TaskPriority::Medium);
    assert_ne!(TaskPriority::Medium, TaskPriority::Low);

    // Test ReportingFrequency
    assert_ne!(ReportingFrequency::Daily, ReportingFrequency::Weekly);
    assert_ne!(ReportingFrequency::Weekly, ReportingFrequency::OnMilestone);
}

#[test]
fn test_entity_id_operations() {
    let id1 = EntityId(1);
    let id2 = EntityId(2);
    let id1_copy = EntityId(1);

    // Test equality
    assert_eq!(id1, id1_copy);
    assert_ne!(id1, id2);

    // Test hash
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(id1, "value1");
    map.insert(id2, "value2");

    assert_eq!(map.get(&id1), Some(&"value1"));
    assert_eq!(map.get(&id2), Some(&"value2"));
    assert_eq!(map.get(&id1_copy), Some(&"value1"));
}

#[test]
fn test_serde_roundtrip_comprehensive() {
    // Test TaskSpec
    let task = TaskSpec::new("test task".to_string()).unwrap();
    let task_json = serde_json::to_string(&task).unwrap();
    let task_decoded: TaskSpec = serde_json::from_str(&task_json).unwrap();
    assert_eq!(task, task_decoded);

    // Test AgentSpec
    let agent_spec = AgentSpec::new("test-agent".to_string()).unwrap();
    let agent_json = serde_json::to_string(&agent_spec).unwrap();
    let agent_decoded: AgentSpec = serde_json::from_str(&agent_json).unwrap();
    assert_eq!(agent_spec, agent_decoded);

    // Test Message
    let origin = EntityId(1);
    let task = TaskSpec::new("test task".to_string()).unwrap();
    let op = Operation::ScheduleAgentTask {
        agent: EntityId(2),
        task,
    };
    let message = Message::new(origin, "capability-token".to_string(), op).unwrap();
    let message_json = serde_json::to_string(&message).unwrap();
    let message_decoded: Message = serde_json::from_str(&message_json).unwrap();
    assert_eq!(message.origin, message_decoded.origin);
    assert_eq!(message.capability, message_decoded.capability);
    // Note: Operation doesn't implement Eq, so we can't compare directly
} 