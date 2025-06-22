//! Agent robustness and edge case tests
//! Tests agent behavior under stress, error conditions, and boundary cases

use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use toka_agents::{Agent, BaseAgent, EventBus, Observation};
use toka_secrets::Vault;
use tempfile::tempdir;

#[tokio::test]
async fn test_agent_with_extreme_belief_values() -> Result<()> {
    let mut agent = BaseAgent::new("extreme_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Test with extreme probability values
    let extreme_observations = vec![
        Observation {
            key: "extreme_positive".to_string(),
            evidence_strength: f64::MAX,
            supports: true,
        },
        Observation {
            key: "extreme_negative".to_string(),
            evidence_strength: f64::MAX,
            supports: false,
        },
        Observation {
            key: "zero_strength".to_string(),
            evidence_strength: 0.0,
            supports: true,
        },
        Observation {
            key: "tiny_strength".to_string(),
            evidence_strength: f64::MIN_POSITIVE,
            supports: true,
        },
    ];
    
    for obs in extreme_observations {
        // Should handle extreme values gracefully
        agent.observe(obs).await?;
    }
    
    // Check that probabilities are still in valid range [0, 1]
    for (_key, belief) in agent.beliefs().iter() {
        assert!(belief.probability >= 0.0 && belief.probability <= 1.0);
        assert!(!belief.probability.is_nan());
        assert!(!belief.probability.is_infinite());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_agent_with_massive_observation_volume() -> Result<()> {
    let mut agent = BaseAgent::new("volume_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Generate many observations rapidly
    for i in 0..10000 {
        let obs = Observation {
            key: format!("obs_{}", i % 100), // Reuse keys to test belief updates
            evidence_strength: 1.0 + (i as f64 * 0.001),
            supports: i % 2 == 0,
        };
        agent.observe(obs).await?;
    }
    
    // Agent should still be responsive
    let hypotheses = agent.hypothesize().await;
    assert!(!hypotheses.is_empty());
    
    // Memory usage should be reasonable (not growing unboundedly)
    assert!(agent.beliefs().len() <= 100); // Should have consolidated beliefs
    
    Ok(())
}

#[tokio::test]
async fn test_agent_threshold_boundary_conditions() -> Result<()> {
    let mut agent = BaseAgent::new_with_thresholds("boundary_test", 0.5, 0.3);
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Test observations that bring beliefs exactly to threshold boundaries
    let boundary_obs = Observation {
        key: "boundary_belief".to_string(),
        evidence_strength: 2.0, // Should push probability to ~0.67
        supports: true,
    };
    agent.observe(boundary_obs).await?;
    
    // Test with threshold values at extremes
    agent.set_action_threshold(0.0); // Everything triggers actions
    agent.set_planning_threshold(1.0); // Nothing triggers planning
    
    let actions = agent.act().await;
    let plans = agent.plan().await;
    
    // With threshold 0.0, should have actions
    assert!(!actions.is_empty());
    // With threshold 1.0, should have no plans
    assert!(plans.is_empty());
    
    // Test with invalid threshold values
    agent.set_action_threshold(-1.0); // Invalid
    agent.set_planning_threshold(2.0); // Invalid
    
    // Should still function (implementation should clamp or handle gracefully)
    let _actions = agent.act().await;
    let _plans = agent.plan().await;
    
    Ok(())
}

#[tokio::test]
async fn test_agent_event_processing_resilience() -> Result<()> {
    let mut agent = BaseAgent::new("resilience_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Test with various malformed event data
    let malformed_events: Vec<(String, String)> = vec![
        ("".to_string(), "".to_string()),
        ("very_long_event_type".repeat(1000), "data".to_string()),
        ("event".to_string(), "very_long_data".repeat(1000)),
        ("event\0with\0nulls".to_string(), "data\0with\0nulls".to_string()),
        ("event\nwith\nnewlines".to_string(), "data\nwith\nnewlines".to_string()),
        ("event🌍unicode".to_string(), "data🌍unicode".to_string()),
        ("event{}".to_string(), "{\"malicious\": \"json\"}".to_string()),
    ];
    
    for (event_type, event_data) in malformed_events {
        // Should not panic or fail catastrophically
        let result = agent.process_event(&event_type, &event_data).await;
        // May succeed or fail, but should not crash
        let _ = result;
    }
    
    // Agent should still be functional after malformed events
    let summary = agent.summarize();
    assert!(!summary.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_agent_state_persistence_edge_cases() -> Result<()> {
    let temp_dir = tempdir()?;
    let vault = Vault::new(temp_dir.path().to_str().unwrap())?;
    
    let mut agent = BaseAgent::new("persistence_test");
    
    // Fill agent with data
    for i in 0..100 {
        let obs = Observation {
            key: format!("persistent_belief_{}", i),
            evidence_strength: 1.0 + (i as f64 * 0.01),
            supports: i % 3 == 0,
        };
        agent.observe(obs).await?;
    }
    
    // Save state
    agent.save_state(&vault).await?;
    
    // Create new agent and try to load
    let mut new_agent = BaseAgent::new("different_id"); // Different ID
    new_agent.load_state(&vault).await?;
    
    // Should handle mismatched IDs gracefully
    let summary = new_agent.summarize();
    assert!(!summary.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_agent_concurrent_operations() -> Result<()> {
    let agent = std::sync::Arc::new(tokio::sync::Mutex::new(BaseAgent::new("concurrent_test")));
    let bus = EventBus::new_default();
    agent.lock().await.set_event_bus(bus);
    
    let mut handles = vec![];
    
    // Spawn multiple concurrent operations
    for i in 0..50 {
        let agent = std::sync::Arc::clone(&agent);
        let handle = tokio::spawn(async move {
            let obs = Observation {
                key: format!("concurrent_obs_{}", i),
                evidence_strength: 1.0 + (i as f64 * 0.01),
                supports: i % 2 == 0,
            };
            
            let mut agent = agent.lock().await;
            agent.observe(obs).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        handle.await??;
    }
    
    // Agent should have processed all observations
    let agent = agent.lock().await;
    assert!(agent.beliefs().len() > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_agent_memory_growth_under_stress() -> Result<()> {
    let mut agent = BaseAgent::new("memory_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Initial memory usage
    let initial_beliefs = agent.beliefs().len();
    
    // Add many unique beliefs
    for i in 0..1000 {
        let obs = Observation {
            key: format!("unique_belief_{}", i),
            evidence_strength: 1.0,
            supports: true,
        };
        agent.observe(obs).await?;
    }
    
    // Check memory growth
    let final_beliefs = agent.beliefs().len();
    assert!(final_beliefs > initial_beliefs);
    
    // But growth should be reasonable (not unbounded)
    assert!(final_beliefs <= 1000);
    
    // Test agent is still responsive
    let hypotheses = agent.hypothesize().await;
    assert!(hypotheses.len() <= final_beliefs);
    
    Ok(())
}

#[tokio::test]
async fn test_agent_with_invalid_timestamps() -> Result<()> {
    let mut agent = BaseAgent::new("timestamp_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Create observation with manually crafted timestamp
    let obs = Observation {
        key: "timestamp_test".to_string(),
        evidence_strength: 1.5,
        supports: true,
    };
    
    agent.observe(obs).await?;
    
    // Check that beliefs have reasonable timestamps
    for (_key, belief) in agent.beliefs().iter() {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Timestamp should be reasonable (within last hour and not in future)
        assert!(belief.last_updated <= current_time);
        assert!(belief.last_updated > current_time - 3600); // Within last hour
    }
    
    Ok(())
}

#[tokio::test]
async fn test_agent_action_and_plan_generation_stress() -> Result<()> {
    let mut agent = BaseAgent::new_with_thresholds("action_test", 0.1, 0.1); // Low thresholds
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Create many beliefs above threshold
    for i in 0..100 {
        let obs = Observation {
            key: format!("actionable_belief_{}", i),
            evidence_strength: 5.0, // High strength to exceed thresholds
            supports: true,
        };
        agent.observe(obs).await?;
    }
    
    // Generate actions and plans
    let actions = agent.act().await;
    let plans = agent.plan().await;
    
    // Should generate reasonable number of actions/plans, not excessive
    assert!(actions.len() > 0);
    assert!(actions.len() <= 100);
    assert!(plans.len() > 0);
    assert!(plans.len() <= 100);
    
    // Actions and plans should be meaningful
    for action in &actions {
        assert!(!action.is_empty());
        assert!(action.contains("hypothesis"));
    }
    
    for plan in &plans {
        assert!(!plan.is_empty());
        assert!(plan.contains("hypothesis"));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_agent_serialization_edge_cases() -> Result<()> {
    let mut agent = BaseAgent::new("serialization_test");
    
    // Add beliefs with special characters and edge case data
    let special_obs = vec![
        Observation {
            key: "key_with_unicode_🌍".to_string(),
            evidence_strength: 1.0,
            supports: true,
        },
        Observation {
            key: "key\nwith\nnewlines".to_string(),
            evidence_strength: 2.0,
            supports: false,
        },
        Observation {
            key: "key\"with\"quotes".to_string(),
            evidence_strength: 1.5,
            supports: true,
        },
        Observation {
            key: "key\\with\\backslashes".to_string(),
            evidence_strength: 0.5,
            supports: true,
        },
    ];
    
    for obs in special_obs {
        agent.observe(obs).await?;
    }
    
    // Test JSON serialization
    let json = serde_json::to_string(&agent)?;
    assert!(!json.is_empty());
    
    // Test deserialization
    let deserialized_agent: BaseAgent = serde_json::from_str(&json)?;
    
    // Should have same number of beliefs
    assert_eq!(agent.beliefs().len(), deserialized_agent.beliefs().len());
    
    Ok(())
}

#[tokio::test]
async fn test_agent_floating_point_precision() -> Result<()> {
    let mut agent = BaseAgent::new("precision_test");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus);
    
    // Test with very small probability differences
    let precise_obs = vec![
        Observation {
            key: "precise_belief".to_string(),
            evidence_strength: 1.0000001,
            supports: true,
        },
        Observation {
            key: "precise_belief".to_string(),
            evidence_strength: 1.0000002,
            supports: true,
        },
    ];
    
    for obs in precise_obs {
        agent.observe(obs).await?;
    }
    
    // Verify precision is maintained reasonably
    if let Some(belief) = agent.beliefs().get("precise_belief") {
        assert!(belief.probability > 0.5);
        assert!(belief.probability < 1.0);
        assert!(!belief.probability.is_nan());
    }
    
    Ok(())
}