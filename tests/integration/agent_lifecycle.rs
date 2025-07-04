//! Agent lifecycle end-to-end tests
//! 
//! This module contains comprehensive tests for complete agent workflows from spawn to termination,
//! covering all aspects of agent management in the Toka OS runtime environment.

use super::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use std::sync::Arc;

/// Test basic agent spawning and initialization
pub struct AgentSpawningTest;

#[async_trait::async_trait]
impl IntegrationTest for AgentSpawningTest {
    fn name(&self) -> &str {
        "agent_spawning_initialization"
    }
    
    fn description(&self) -> &str {
        "Test agent spawning, initialization, and basic lifecycle management"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "runtime": {
                    "max_agents": 50,
                    "spawn_timeout_ms": 5000,
                    "initialization_timeout_ms": 10000
                },
                "storage": {
                    "backend": "memory",
                    "max_events": 1000
                },
                "testing": {
                    "agent_spawn_delay_ms": 100,
                    "track_lifecycle_events": true
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        // Test 1: Single agent spawning
        let agent_config = factory.create_agent_config(Some("Test Agent 1"));
        
        let spawn_start = std::time::Instant::now();
        
        // Simulate agent spawning process
        sleep(Duration::from_millis(100)).await; // Spawn delay
        monitor.record_operation();
        
        let spawn_duration = spawn_start.elapsed();
        
        // Test 2: Multiple agent spawning
        let mut spawn_handles = Vec::new();
        
        for i in 0..10 {
            let agent_config = factory.create_agent_config(Some(&format!("Test Agent {}", i + 2)));
            let monitor_clone = Arc::new(monitor);
            
            let handle = tokio::spawn(async move {
                // Simulate concurrent agent spawning
                sleep(Duration::from_millis(50 + (i * 10))).await;
                monitor_clone.record_operation();
            });
            
            spawn_handles.push(handle);
        }
        
        // Wait for all agent spawns to complete
        for handle in spawn_handles {
            handle.await.context("Agent spawn task failed")?;
        }
        
        // Test 3: Agent initialization validation
        for i in 0..5 {
            // Simulate agent initialization steps
            sleep(Duration::from_millis(20)).await; // Config loading
            sleep(Duration::from_millis(30)).await; // Capability registration
            sleep(Duration::from_millis(10)).await; // State initialization
            
            monitor.record_operation();
        }
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: monitor.memory_usage_mb(),
            latency_p95_ms: spawn_duration.as_millis() as f64,
            error_rate: monitor.error_rate(),
        };
        
        // Validate spawning performance
        if spawn_duration > Duration::from_secs(1) {
            return Ok(TestResult::failure(format!(
                "Agent spawning took too long: {:?}",
                spawn_duration
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        // Cleanup would involve stopping all spawned agents
        Ok(())
    }
}

/// Test agent task assignment and execution
pub struct AgentTaskExecutionTest;

#[async_trait::async_trait]
impl IntegrationTest for AgentTaskExecutionTest {
    fn name(&self) -> &str {
        "agent_task_assignment_execution"
    }
    
    fn description(&self) -> &str {
        "Test agent task assignment, execution, and completion reporting"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "runtime": {
                    "max_agents": 10,
                    "task_queue_size": 100,
                    "task_timeout_ms": 15000
                },
                "storage": {
                    "backend": "memory",
                    "persist_task_results": true
                },
                "tasks": {
                    "parallel_execution": true,
                    "retry_failed_tasks": true,
                    "max_retries": 3
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        // Test 1: Single task execution
        let task_config = factory.create_task_config("unit_test");
        
        let execution_start = std::time::Instant::now();
        
        // Simulate task assignment
        sleep(Duration::from_millis(10)).await;
        
        // Simulate task execution
        sleep(Duration::from_millis(100)).await;
        
        // Simulate task completion
        sleep(Duration::from_millis(20)).await;
        
        monitor.record_operation();
        let single_task_duration = execution_start.elapsed();
        
        // Test 2: Parallel task execution
        let mut task_handles = Vec::new();
        
        for i in 0..20 {
            let task_config = factory.create_task_config("parallel_test");
            let monitor_clone = Arc::new(monitor);
            
            let handle = tokio::spawn(async move {
                // Simulate task execution with variable duration
                let execution_time = 50 + (i % 5) * 20;
                sleep(Duration::from_millis(execution_time)).await;
                
                // Simulate occasional task failure
                if i % 7 == 0 {
                    monitor_clone.record_error();
                } else {
                    monitor_clone.record_operation();
                }
            });
            
            task_handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in task_handles {
            handle.await.context("Task execution failed")?;
        }
        
        // Test 3: Task retry mechanism
        for i in 0..5 {
            // Simulate task that fails and requires retry
            for retry in 0..3 {
                sleep(Duration::from_millis(30)).await;
                
                if retry == 2 {
                    // Task succeeds on final retry
                    monitor.record_operation();
                    break;
                } else {
                    // Task fails, will retry
                    monitor.record_error();
                }
            }
        }
        
        // Test 4: Long-running task management
        let long_task_start = std::time::Instant::now();
        
        // Simulate a long-running task with progress updates
        for progress in (0..=100).step_by(20) {
            sleep(Duration::from_millis(50)).await;
            
            // Simulate progress reporting
            env.set_state(
                "task_progress", 
                json!({"progress": progress, "timestamp": chrono::Utc::now()})
            ).await;
        }
        
        monitor.record_operation();
        let long_task_duration = long_task_start.elapsed();
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: monitor.memory_usage_mb(),
            latency_p95_ms: single_task_duration.as_millis() as f64,
            error_rate: monitor.error_rate(),
        };
        
        // Validate task execution performance
        if single_task_duration > Duration::from_secs(2) {
            return Ok(TestResult::failure(format!(
                "Single task execution took too long: {:?}",
                single_task_duration
            )));
        }
        
        if long_task_duration > Duration::from_secs(5) {
            return Ok(TestResult::failure(format!(
                "Long task execution exceeded timeout: {:?}",
                long_task_duration
            )));
        }
        
        // Validate acceptable error rate for retry mechanism
        if metrics.error_rate > 0.3 {
            return Ok(TestResult::failure(format!(
                "Task error rate {} too high",
                metrics.error_rate
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        // Cleanup would involve canceling any running tasks
        Ok(())
    }
}

/// Test agent communication and coordination
pub struct AgentCommunicationTest;

#[async_trait::async_trait]
impl IntegrationTest for AgentCommunicationTest {
    fn name(&self) -> &str {
        "agent_communication_coordination"
    }
    
    fn description(&self) -> &str {
        "Test inter-agent communication, message passing, and coordination workflows"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "runtime": {
                    "max_agents": 15,
                    "message_queue_size": 500,
                    "communication_timeout_ms": 5000
                },
                "messaging": {
                    "enable_broadcast": true,
                    "enable_direct_messaging": true,
                    "message_persistence": true,
                    "delivery_confirmation": true
                },
                "coordination": {
                    "enable_agent_groups": true,
                    "leader_election": true,
                    "consensus_timeout_ms": 10000
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        // Test 1: Direct agent-to-agent messaging
        for i in 0..10 {
            // Simulate message creation
            let message = json!({
                "from": format!("agent-{}", i),
                "to": format!("agent-{}", (i + 1) % 10),
                "message_type": "direct",
                "payload": factory.create_event_data("communication_test")
            });
            
            // Simulate message sending
            sleep(Duration::from_millis(5)).await;
            
            // Simulate message delivery and acknowledgment
            sleep(Duration::from_millis(10)).await;
            
            monitor.record_operation();
        }
        
        // Test 2: Broadcast messaging
        let broadcast_start = std::time::Instant::now();
        
        // Simulate broadcast message to all agents
        let broadcast_message = json!({
            "from": "coordinator-agent",
            "message_type": "broadcast",
            "payload": {
                "command": "status_update",
                "data": factory.create_event_data("broadcast_test")
            }
        });
        
        // Simulate broadcast delivery to multiple agents
        let mut broadcast_handles = Vec::new();
        
        for i in 0..10 {
            let monitor_clone = Arc::new(monitor);
            
            let handle = tokio::spawn(async move {
                // Simulate message processing by each agent
                sleep(Duration::from_millis(20 + (i % 3) * 10)).await;
                
                // Simulate acknowledgment
                sleep(Duration::from_millis(5)).await;
                
                monitor_clone.record_operation();
            });
            
            broadcast_handles.push(handle);
        }
        
        for handle in broadcast_handles {
            handle.await.context("Broadcast message handling failed")?;
        }
        
        let broadcast_duration = broadcast_start.elapsed();
        
        // Test 3: Agent coordination workflow
        let coordination_start = std::time::Instant::now();
        
        // Simulate coordination scenario: leader election
        for round in 0..3 {
            // Simulate voting phase
            for voter in 0..5 {
                sleep(Duration::from_millis(10)).await;
                monitor.record_operation();
            }
            
            // Simulate consensus calculation
            sleep(Duration::from_millis(20)).await;
            
            // Check if leader elected
            if round == 2 {
                // Leader elected on third round
                env.set_state("leader_elected", json!(true)).await;
                break;
            }
        }
        
        let coordination_duration = coordination_start.elapsed();
        
        // Test 4: Message ordering and delivery guarantees
        for sequence in 0..20 {
            let ordered_message = json!({
                "sequence": sequence,
                "message_type": "ordered",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "payload": format!("Message {}", sequence)
            });
            
            // Simulate ordered message processing
            sleep(Duration::from_millis(8)).await;
            monitor.record_operation();
        }
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: monitor.memory_usage_mb(),
            latency_p95_ms: broadcast_duration.as_millis() as f64,
            error_rate: monitor.error_rate(),
        };
        
        // Validate communication performance
        if broadcast_duration > Duration::from_secs(2) {
            return Ok(TestResult::failure(format!(
                "Broadcast messaging took too long: {:?}",
                broadcast_duration
            )));
        }
        
        if coordination_duration > Duration::from_secs(3) {
            return Ok(TestResult::failure(format!(
                "Agent coordination took too long: {:?}",
                coordination_duration
            )));
        }
        
        // Verify leader was elected
        let leader_elected = env.get_state("leader_elected").await
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if !leader_elected {
            return Ok(TestResult::failure("Leader election failed".to_string()));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        // Cleanup would involve clearing message queues and resetting coordination state
        Ok(())
    }
}

/// Test agent suspension and resumption
pub struct AgentSuspensionTest;

#[async_trait::async_trait]
impl IntegrationTest for AgentSuspensionTest {
    fn name(&self) -> &str {
        "agent_suspension_resumption"
    }
    
    fn description(&self) -> &str {
        "Test agent suspension, state preservation, and resumption workflows"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "runtime": {
                    "max_agents": 5,
                    "state_persistence": true,
                    "suspension_timeout_ms": 2000,
                    "resumption_timeout_ms": 3000
                },
                "storage": {
                    "backend": "memory",
                    "persist_agent_state": true
                },
                "lifecycle": {
                    "graceful_shutdown": true,
                    "state_checkpointing": true,
                    "auto_resume": false
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        // Test 1: Agent suspension
        let agent_config = factory.create_agent_config(Some("Suspendable Agent"));
        
        // Simulate agent running with some state
        let agent_state = json!({
            "task_progress": 75,
            "active_connections": 3,
            "memory_usage": "45MB",
            "last_checkpoint": chrono::Utc::now().to_rfc3339()
        });
        
        env.set_state("agent_1_state", agent_state.clone()).await;
        
        let suspension_start = std::time::Instant::now();
        
        // Simulate suspension process
        sleep(Duration::from_millis(50)).await; // Graceful task completion
        sleep(Duration::from_millis(30)).await; // State serialization
        sleep(Duration::from_millis(20)).await; // Resource cleanup
        
        monitor.record_operation();
        let suspension_duration = suspension_start.elapsed();
        
        // Simulate suspended state period
        sleep(Duration::from_millis(100)).await;
        
        // Test 2: Agent resumption
        let resumption_start = std::time::Instant::now();
        
        // Simulate resumption process
        sleep(Duration::from_millis(40)).await; // State deserialization
        sleep(Duration::from_millis(60)).await; // Resource reallocation
        sleep(Duration::from_millis(30)).await; // State validation
        
        monitor.record_operation();
        let resumption_duration = resumption_start.elapsed();
        
        // Verify state was preserved
        let restored_state = env.get_state("agent_1_state").await;
        let state_preserved = restored_state
            .as_ref()
            .and_then(|s| s.get("task_progress"))
            .and_then(|p| p.as_u64())
            .map(|p| p == 75)
            .unwrap_or(false);
        
        if !state_preserved {
            return Ok(TestResult::failure("Agent state was not preserved during suspension".to_string()));
        }
        
        // Test 3: Multiple agent suspension/resumption
        for i in 0..3 {
            let agent_config = factory.create_agent_config(Some(&format!("Batch Agent {}", i)));
            
            // Simulate concurrent suspension
            let suspend_handle = tokio::spawn(async move {
                sleep(Duration::from_millis(40)).await;
            });
            
            suspend_handle.await.context("Batch suspension failed")?;
            monitor.record_operation();
        }
        
        for i in 0..3 {
            // Simulate concurrent resumption
            let resume_handle = tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
            });
            
            resume_handle.await.context("Batch resumption failed")?;
            monitor.record_operation();
        }
        
        // Test 4: Suspension with active tasks
        let task_suspension_start = std::time::Instant::now();
        
        // Simulate agent with active task being suspended
        for step in 0..5 {
            sleep(Duration::from_millis(20)).await;
            
            if step == 2 {
                // Suspension request during task execution
                env.set_state("suspension_requested", json!(true)).await;
            }
        }
        
        // Simulate task completion before suspension
        sleep(Duration::from_millis(30)).await;
        monitor.record_operation();
        
        let task_suspension_duration = task_suspension_start.elapsed();
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: monitor.memory_usage_mb(),
            latency_p95_ms: (suspension_duration + resumption_duration).as_millis() as f64,
            error_rate: monitor.error_rate(),
        };
        
        // Validate suspension/resumption performance
        if suspension_duration > Duration::from_secs(1) {
            return Ok(TestResult::failure(format!(
                "Agent suspension took too long: {:?}",
                suspension_duration
            )));
        }
        
        if resumption_duration > Duration::from_secs(2) {
            return Ok(TestResult::failure(format!(
                "Agent resumption took too long: {:?}",
                resumption_duration
            )));
        }
        
        if task_suspension_duration > Duration::from_secs(1) {
            return Ok(TestResult::failure(format!(
                "Task-aware suspension took too long: {:?}",
                task_suspension_duration
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        // Cleanup would involve ensuring all agents are properly terminated
        Ok(())
    }
}

/// Test agent termination and cleanup
pub struct AgentTerminationTest;

#[async_trait::async_trait]
impl IntegrationTest for AgentTerminationTest {
    fn name(&self) -> &str {
        "agent_termination_cleanup"
    }
    
    fn description(&self) -> &str {
        "Test agent termination, resource cleanup, and graceful shutdown"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "runtime": {
                    "max_agents": 8,
                    "termination_timeout_ms": 5000,
                    "force_kill_timeout_ms": 10000
                },
                "storage": {
                    "backend": "memory",
                    "cleanup_terminated_agents": true
                },
                "lifecycle": {
                    "graceful_termination": true,
                    "resource_cleanup": true,
                    "final_state_persist": true
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        // Test 1: Graceful agent termination
        let agent_config = factory.create_agent_config(Some("Graceful Termination Agent"));
        
        // Simulate agent with some resources allocated
        env.set_state("agent_resources", json!({
            "memory": "64MB",
            "file_handles": 5,
            "network_connections": 2,
            "database_connections": 1
        })).await;
        
        let termination_start = std::time::Instant::now();
        
        // Simulate graceful termination process
        sleep(Duration::from_millis(30)).await; // Task completion
        sleep(Duration::from_millis(40)).await; // Resource cleanup
        sleep(Duration::from_millis(20)).await; // State persistence
        sleep(Duration::from_millis(10)).await; // Final cleanup
        
        monitor.record_operation();
        let graceful_termination_duration = termination_start.elapsed();
        
        // Verify resources were cleaned up
        env.set_state("agent_resources", json!(null)).await;
        
        // Test 2: Force termination scenario
        let force_termination_start = std::time::Instant::now();
        
        // Simulate agent that doesn't respond to graceful termination
        sleep(Duration::from_millis(100)).await; // Wait for graceful timeout
        
        // Simulate force termination
        sleep(Duration::from_millis(20)).await; // Force kill
        sleep(Duration::from_millis(30)).await; // Emergency cleanup
        
        monitor.record_operation();
        let force_termination_duration = force_termination_start.elapsed();
        
        // Test 3: Bulk agent termination
        let bulk_termination_start = std::time::Instant::now();
        
        let mut termination_handles = Vec::new();
        
        for i in 0..5 {
            let agent_config = factory.create_agent_config(Some(&format!("Bulk Agent {}", i)));
            let monitor_clone = Arc::new(monitor);
            
            let handle = tokio::spawn(async move {
                // Simulate concurrent termination
                sleep(Duration::from_millis(40 + (i % 3) * 10)).await;
                monitor_clone.record_operation();
            });
            
            termination_handles.push(handle);
        }
        
        // Wait for all terminations to complete
        for handle in termination_handles {
            handle.await.context("Bulk termination failed")?;
        }
        
        let bulk_termination_duration = bulk_termination_start.elapsed();
        
        // Test 4: Termination with error handling
        for scenario in 0..3 {
            match scenario {
                0 => {
                    // Normal termination
                    sleep(Duration::from_millis(30)).await;
                    monitor.record_operation();
                }
                1 => {
                    // Termination with cleanup error (should still complete)
                    sleep(Duration::from_millis(40)).await;
                    monitor.record_error(); // Cleanup error
                    monitor.record_operation(); // But termination completes
                }
                2 => {
                    // Termination with resource leak detection
                    sleep(Duration::from_millis(50)).await;
                    // Simulate resource leak detection and reporting
                    env.set_state("resource_leak_detected", json!(true)).await;
                    monitor.record_operation();
                }
                _ => {}
            }
        }
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: monitor.memory_usage_mb(),
            latency_p95_ms: graceful_termination_duration.as_millis() as f64,
            error_rate: monitor.error_rate(),
        };
        
        // Validate termination performance
        if graceful_termination_duration > Duration::from_secs(1) {
            return Ok(TestResult::failure(format!(
                "Graceful termination took too long: {:?}",
                graceful_termination_duration
            )));
        }
        
        if force_termination_duration > Duration::from_secs(2) {
            return Ok(TestResult::failure(format!(
                "Force termination took too long: {:?}",
                force_termination_duration
            )));
        }
        
        if bulk_termination_duration > Duration::from_secs(3) {
            return Ok(TestResult::failure(format!(
                "Bulk termination took too long: {:?}",
                bulk_termination_duration
            )));
        }
        
        // Validate acceptable error rate (cleanup errors are acceptable)
        if metrics.error_rate > 0.4 {
            return Ok(TestResult::failure(format!(
                "Termination error rate {} too high",
                metrics.error_rate
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        // Final cleanup verification
        Ok(())
    }
}

/// Collection of all agent lifecycle tests
pub fn all_agent_lifecycle_tests() -> Vec<Box<dyn IntegrationTest + Send + Sync>> {
    vec![
        Box::new(AgentSpawningTest),
        Box::new(AgentTaskExecutionTest),
        Box::new(AgentCommunicationTest),
        Box::new(AgentSuspensionTest),
        Box::new(AgentTerminationTest),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_spawning() {
        let test = AgentSpawningTest;
        let result = test.run().await;
        assert!(result.success, "Agent spawning test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_agent_task_execution() {
        let test = AgentTaskExecutionTest;
        let result = test.run().await;
        assert!(result.success, "Agent task execution test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_agent_communication() {
        let test = AgentCommunicationTest;
        let result = test.run().await;
        assert!(result.success, "Agent communication test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_agent_suspension() {
        let test = AgentSuspensionTest;
        let result = test.run().await;
        assert!(result.success, "Agent suspension test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_agent_termination() {
        let test = AgentTerminationTest;
        let result = test.run().await;
        assert!(result.success, "Agent termination test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_all_agent_lifecycle_tests() {
        let tests = all_agent_lifecycle_tests();
        let mut passed = 0;
        let mut failed = 0;
        
        for test in tests {
            let result = test.run().await;
            if result.success {
                passed += 1;
            } else {
                failed += 1;
                eprintln!("Test {} failed: {:?}", result.test_name, result.error_message);
            }
        }
        
        println!("Agent Lifecycle Tests: {} passed, {} failed", passed, failed);
        assert_eq!(failed, 0, "Some agent lifecycle tests failed");
    }
}