//! Toka OS v0.3.0 Integration Test Suite
//! 
//! This crate provides comprehensive integration testing capabilities for Toka OS,
//! validating cross-crate interactions and end-to-end workflows.

#![allow(dead_code)] // Integration tests may have unused utilities

pub mod mod_integration {
    include!("mod.rs");
}

pub use mod_integration::*;

// Re-export common testing utilities
pub use common::{TestEnvironment, TestDataFactory, PerformanceMonitor};
pub use runtime_storage::all_runtime_storage_tests;

/// Main integration test runner
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn run_all_integration_tests() {
        // Initialize tracing for test output
        let _ = tracing_subscriber::fmt::try_init();
        
        println!("🚀 Starting Toka OS v0.3.0 Integration Test Suite");
        
        // Run runtime-storage integration tests
        let runtime_storage_tests = all_runtime_storage_tests();
        let results = run_integration_tests![
            runtime_storage_tests[0],
            runtime_storage_tests[1],
            runtime_storage_tests[2],
            runtime_storage_tests[3],
            runtime_storage_tests[4],
        ];
        
        // Analyze results
        let passed = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();
        let total_duration: std::time::Duration = results.iter().map(|r| r.duration).sum();
        
        println!("\n📊 Integration Test Summary:");
        println!("   Total tests: {}", results.len());
        println!("   Passed: {} ✅", passed);
        println!("   Failed: {} ❌", failed);
        println!("   Total duration: {:?}", total_duration);
        
        if failed > 0 {
            println!("\n❌ Failed tests:");
            for result in &results {
                if !result.success {
                    println!("   - {}: {:?}", result.test_name, result.error_message);
                }
            }
        }
        
        // Performance summary
        let performance_tests: Vec<_> = results.iter()
            .filter(|r| r.performance_metrics.is_some())
            .collect();
            
        if !performance_tests.is_empty() {
            println!("\n📈 Performance Metrics Summary:");
            for result in performance_tests {
                if let Some(metrics) = &result.performance_metrics {
                    println!("   {}: {:.2} ops/sec, {:.2}MB memory, {:.2}ms p95", 
                        result.test_name,
                        metrics.operations_per_second,
                        metrics.memory_usage_mb,
                        metrics.latency_p95_ms
                    );
                }
            }
        }
        
        // Assert all tests passed
        assert_eq!(failed, 0, "Some integration tests failed");
        
        println!("\n🎉 All integration tests passed!");
    }
}