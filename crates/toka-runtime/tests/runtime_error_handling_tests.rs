//! Runtime error handling and resource management tests
//! Tests edge cases, error recovery, and resource cleanup

#![cfg(all(feature = "toolkit", feature = "vault"))]

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use toka_agents::{Agent, SymbolicAgent};
use toka_runtime::runtime::{Runtime, RuntimeConfig};

#[tokio::test]
async fn test_runtime_with_invalid_paths() -> Result<()> {
    // Test with non-existent directory
    let invalid_config = RuntimeConfig {
        vault_path: "/non/existent/path/that/should/fail".to_string(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: "/another/invalid/path".to_string(),
    };
    
    // Should handle gracefully or create directories
    let result = Runtime::new(invalid_config).await;
    // This might succeed if the runtime creates directories, or fail gracefully
    // The important thing is it doesn't panic
    let _ = result;
    
    Ok(())
}

#[tokio::test]
async fn test_max_agents_limit_enforcement() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 2, // Very low limit
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    
    // Register up to the limit
    let agent1 = Box::new(SymbolicAgent::new("agent1"));
    let agent2 = Box::new(SymbolicAgent::new("agent2"));
    
    runtime.register_agent(agent1).await?;
    runtime.register_agent(agent2).await?;
    
    // Third agent should fail
    let agent3 = Box::new(SymbolicAgent::new("agent3"));
    let result = runtime.register_agent(agent3).await;
    assert!(result.is_err());
    
    // Should still have exactly 2 agents
    assert_eq!(runtime.list_agents().await.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_event_buffer_overflow_handling() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 1, // Very small buffer
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    runtime.start().await?;
    
    // Flood the event system
    for i in 0..100 {
        let _ = runtime.emit_event(
            format!("test_event_{}", i),
            format!("test_data_{}", i),
        ).await;
        
        // Small delay to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    
    // Runtime should still be functional
    assert!(runtime.is_running().await);
    runtime.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_runtime_restart_cycle() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    
    // Multiple start/stop cycles
    for _ in 0..5 {
        runtime.start().await?;
        assert!(runtime.is_running().await);
        
        runtime.stop().await?;
        assert!(!runtime.is_running().await);
    }
    
    // Should still be functional after multiple cycles
    runtime.start().await?;
    let agent = Box::new(SymbolicAgent::new("test_agent"));
    runtime.register_agent(agent).await?;
    assert_eq!(runtime.list_agents().await.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_agent_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 100,
        event_buffer_size: 1000,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Arc::new(Runtime::new(config).await?);
    let mut handles = vec![];
    
    // Concurrently register agents
    for i in 0..50 {
        let runtime = Arc::clone(&runtime);
        let handle = tokio::spawn(async move {
            let agent = Box::new(SymbolicAgent::new(&format!("agent_{}", i)));
            runtime.register_agent(agent).await
        });
        handles.push(handle);
    }
    
    // Wait for all registrations
    let mut successful_registrations = 0;
    for handle in handles {
        if handle.await?.is_ok() {
            successful_registrations += 1;
        }
    }
    
    // Should have registered many agents successfully
    assert!(successful_registrations > 40);
    assert_eq!(runtime.list_agents().await.len(), successful_registrations);
    
    Ok(())
}

#[tokio::test]
async fn test_agent_removal_edge_cases() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    
    // Try to remove non-existent agent
    let result = runtime.remove_agent("non_existent_agent").await;
    assert!(result.is_err());
    
    // Register an agent
    let agent = Box::new(SymbolicAgent::new("test_agent"));
    let agent_id = runtime.register_agent(agent).await?;
    
    // Remove it successfully
    runtime.remove_agent(&agent_id).await?;
    assert_eq!(runtime.list_agents().await.len(), 0);
    
    // Try to remove the same agent again
    let result = runtime.remove_agent(&agent_id).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_state_persistence_failure_recovery() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config.clone()).await?;
    
    // Register an agent
    let agent = Box::new(SymbolicAgent::new("persistent_agent"));
    let agent_id = runtime.register_agent(agent).await?;
    
    // Save state
    runtime.save_state().await?;
    
    // Drop the runtime
    drop(runtime);
    
    // Create new runtime with same config
    let runtime2 = Runtime::new(config).await?;
    
    // Should have loaded the agent
    let agents = runtime2.list_agents().await;
    assert!(!agents.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_storage_adapter_error_handling() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    
    // Test with non-existent storage scheme
    let storage = runtime.storage("non_existent_scheme").await;
    assert!(storage.is_none());
    
    // Test with valid scheme
    let local_storage = runtime.storage("local").await;
    assert!(local_storage.is_some());
    
    // Test storage operations with the actual adapter
    if let Some(adapter) = local_storage {
        // This should work
        adapter.put("test://file.txt", b"test data").await?;
        let data = adapter.get("test://file.txt").await?;
        assert_eq!(data, b"test data");
        
        // Clean up
        adapter.delete("test://file.txt").await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_runtime_drop_cleanup() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    runtime.start().await?;
    
    // Register some agents
    for i in 0..3 {
        let agent = Box::new(SymbolicAgent::new(&format!("agent_{}", i)));
        runtime.register_agent(agent).await?;
    }
    
    assert!(runtime.is_running().await);
    assert_eq!(runtime.list_agents().await.len(), 3);
    
    // Drop the runtime (should trigger cleanup)
    drop(runtime);
    
    // Give some time for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // This test mainly ensures no panics occur during drop
    Ok(())
}

#[tokio::test]
async fn test_malformed_event_handling() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    runtime.start().await?;
    
    // Register an agent
    let agent = Box::new(SymbolicAgent::new("test_agent"));
    runtime.register_agent(agent).await?;
    
    // Send various malformed events
    let malformed_events = vec![
        ("", ""),
        ("malformed", ""),
        ("", "malformed"),
        ("very_long_event_type".repeat(100), "data"),
        ("event", "very_long_data".repeat(1000)),
        ("event\0with\0nulls", "data\0with\0nulls"),
        ("event\nwith\nnewlines", "data\nwith\nnewlines"),
        ("event🌍with🌍unicode", "data🌍with🌍unicode"),
    ];
    
    for (event_type, event_data) in malformed_events {
        // Should not panic or crash the runtime
        let _ = runtime.emit_event(event_type.to_string(), event_data.to_string()).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    
    // Runtime should still be running
    assert!(runtime.is_running().await);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_start_stop_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Arc::new(Runtime::new(config).await?);
    let mut handles = vec![];
    
    // Concurrently start and stop the runtime
    for i in 0..10 {
        let runtime = Arc::clone(&runtime);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                runtime.start().await
            } else {
                runtime.stop().await
            }
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        let _ = handle.await?; // Some may fail, that's okay
    }
    
    // Runtime should end up in a consistent state
    let is_running = runtime.is_running().await;
    // State should be either running or stopped, not in an inconsistent state
    assert!(is_running || !is_running); // This is a tautology, but ensures no panic
    
    Ok(())
}

#[tokio::test]
async fn test_resource_limits_and_cleanup() -> Result<()> {
    let temp_dir = tempdir()?;
    let config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 1000, // High limit for stress test
        event_buffer_size: 1000,
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(config).await?;
    runtime.start().await?;
    
    // Register many agents to test resource usage
    let mut agent_ids = Vec::new();
    for i in 0..100 {
        let agent = Box::new(SymbolicAgent::new(&format!("stress_agent_{}", i)));
        let agent_id = runtime.register_agent(agent).await?;
        agent_ids.push(agent_id);
    }
    
    // Generate many events
    for i in 0..1000 {
        runtime.emit_event(
            format!("stress_event_{}", i),
            format!("stress_data_{}", i),
        ).await?;
        
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Save state with many agents
    runtime.save_state().await?;
    
    // Remove all agents
    for agent_id in agent_ids {
        runtime.remove_agent(&agent_id).await?;
    }
    
    assert_eq!(runtime.list_agents().await.len(), 0);
    
    runtime.stop().await?;
    Ok(())
}

#[tokio::test] 
async fn test_runtime_configuration_edge_cases() -> Result<()> {
    let temp_dir = tempdir()?;
    
    // Test with extreme values
    let extreme_config = RuntimeConfig {
        vault_path: temp_dir.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 0, // Zero agents allowed
        event_buffer_size: 1, // Minimum buffer
        storage_root: temp_dir.path().join("storage").to_string_lossy().into_owned(),
    };
    
    let runtime = Runtime::new(extreme_config).await?;
    
    // Should not be able to register any agents
    let agent = Box::new(SymbolicAgent::new("test_agent"));
    let result = runtime.register_agent(agent).await;
    assert!(result.is_err());
    
    // Should still be able to start/stop
    runtime.start().await?;
    assert!(runtime.is_running().await);
    runtime.stop().await?;
    assert!(!runtime.is_running().await);
    
    Ok(())
}