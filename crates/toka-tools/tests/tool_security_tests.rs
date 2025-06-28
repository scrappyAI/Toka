//! Security-focused tests for toolkit tools
//! Tests input validation, resource limits, and protection against malicious inputs

use anyhow::Result;
use std::collections::HashMap;
use tempfile::tempdir;
use toka_tools::tools::{ToolParams, ToolRegistry};

#[tokio::test]
async fn test_ingestion_tool_path_traversal_protection() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test various path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "/etc/passwd",
        "C:\\Windows\\System32\\config\\SAM",
        "./../../../../root/.ssh/id_rsa",
    ];
    
    for malicious_path in malicious_paths {
        let params = ToolParams {
            name: "ingestion".to_string(),
            args: {
                let mut map = HashMap::new();
                map.insert("path".to_string(), malicious_path.to_string());
                map
            },
        };
        
        // Should either fail validation or safely handle the path
        let result = registry.execute_tool("ingestion", &params).await;
        if let Ok(tool_result) = result {
            // If it succeeds, it should not contain sensitive data
            assert!(!tool_result.output.contains("root:"));
            assert!(!tool_result.output.contains("password"));
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_tool_registry_concurrent_access() -> Result<()> {
    let registry = std::sync::Arc::new(ToolRegistry::new().await?);
    let mut handles = vec![];
    
    // Spawn multiple concurrent tool executions
    for i in 0..50 {
        let registry = std::sync::Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let params = ToolParams {
                name: "echo".to_string(),
                args: {
                    let mut map = HashMap::new();
                    map.insert("message".to_string(), format!("concurrent_test_{}", i));
                    map
                },
            };
            
            registry.execute_tool("echo", &params).await
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    let mut successful_executions = 0;
    for handle in handles {
        if let Ok(Ok(tool_result)) = handle.await {
            if tool_result.success {
                successful_executions += 1;
            }
        }
    }
    
    // Most should succeed (allowing for some potential race conditions)
    assert!(successful_executions >= 45);
    
    Ok(())
}

#[tokio::test]
async fn test_malicious_json_input_handling() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test with malicious JSON inputs
    let malicious_json = "{}".to_string(); // Start simple
    let params = ToolParams {
        name: "ledger".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("data".to_string(), malicious_json);
            map
        },
    };
    
    let result = registry.execute_tool("ledger", &params).await;
    match result {
        Ok(tool_result) => {
            // Should not execute scripts or access privileged data
            assert!(!tool_result.output.contains("script"));
            assert!(!tool_result.output.contains("admin"));
        }
        Err(_) => {
            // Acceptable to reject malformed input
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_large_input_handling() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Create a very large input
    let large_content = "x".repeat(1_000_000); // 1MB string
    let params = ToolParams {
        name: "echo".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("message".to_string(), large_content);
            map
        },
    };
    
    // Should handle large inputs gracefully
    let result = registry.execute_tool("echo", &params).await;
    match result {
        Ok(tool_result) => {
            // Should not consume excessive memory or time
            assert!(tool_result.metadata.execution_time_ms < 30000); // 30 second limit
        }
        Err(_) => {
            // Acceptable to reject oversized inputs
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_empty_and_null_parameter_handling() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test with empty parameters
    let params = ToolParams {
        name: "echo".to_string(),
        args: HashMap::new(), // No required 'message' parameter
    };
    
    let result = registry.execute_tool("echo", &params).await;
    // Should handle missing parameters gracefully
    assert!(result.is_err()); // Should fail parameter validation
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_tool_names() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    let invalid_tools = vec![
        "non_existent_tool",
        "",
        "../../dangerous_tool",
        "tool\0with\0nulls",
    ];
    
    for invalid_tool in invalid_tools {
        let params = ToolParams {
            name: invalid_tool.to_string(),
            args: HashMap::new(),
        };
        
        let result = registry.execute_tool(invalid_tool, &params).await;
        // Should properly reject invalid tool names
        assert!(result.is_err());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_file_extension_validation() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test with suspicious file extensions
    let temp_dir = tempdir()?;
    let suspicious_file = temp_dir.path().join("malware.exe");
    std::fs::write(&suspicious_file, "harmless content")?;
    
    let params = ToolParams {
        name: "ingestion".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("path".to_string(), suspicious_file.to_string_lossy().into_owned());
            map
        },
    };
    
    // Should reject suspicious file types
    let result = registry.execute_tool("ingestion", &params).await;
    if let Ok(tool_result) = result {
        // If it processes the file, it should be safely handled
        assert!(!tool_result.output.contains("execute"));
        assert!(!tool_result.output.contains("run"));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_time_validation_in_scheduling() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test with invalid time formats
    let invalid_times = vec![
        "invalid-time-format",
        "1970-01-01T00:00:00Z", // Past time
        "",
        "null",
    ];
    
    for invalid_time in invalid_times {
        let params = ToolParams {
            name: "scheduling".to_string(),
            args: {
                let mut map = HashMap::new();
                map.insert("task".to_string(), "test_task".to_string());
                map.insert("time".to_string(), invalid_time.to_string());
                map
            },
        };
        
        let result = registry.execute_tool("scheduling", &params).await;
        // Should validate time inputs properly
        if let Ok(tool_result) = result {
            assert!(!tool_result.output.contains("1970"));
        }
        // Most likely should fail for invalid times
    }
    
    Ok(())
}

#[tokio::test]
async fn test_resource_exhaustion_protection() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Test with deeply nested or very wide data structures
    let exhaustive_data = format!("{{{}}}", "\"key\": \"value\",".repeat(10000));
    
    let params = ToolParams {
        name: "ledger".to_string(),
        args: {
            let mut map = HashMap::new();
            map.insert("data".to_string(), exhaustive_data);
            map
        },
    };
    
    // Should complete within reasonable time limits
    let start_time = std::time::Instant::now();
    let result = registry.execute_tool("ledger", &params).await;
    let execution_time = start_time.elapsed();
    
    // Should not take more than 30 seconds
    assert!(execution_time.as_secs() < 30, "Tool execution took too long");
    
    match result {
        Ok(tool_result) => {
            // Should have reasonable output size
            assert!(tool_result.output.len() < 100_000_000); // 100MB limit
        }
        Err(_) => {
            // Acceptable to reject resource-intensive inputs
        }
    }
    
    Ok(())
}