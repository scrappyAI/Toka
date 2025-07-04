//! Integration test framework for Toka OS v0.3.0
//! 
//! This module provides comprehensive integration testing capabilities
//! that validate cross-crate interactions and end-to-end workflows.
//! 
//! # Test Categories
//! 
//! - **Runtime-Storage Integration**: Tests runtime coordination with different storage backends
//! - **Agent Lifecycle**: End-to-end agent spawning, task execution, and termination
//! - **Event Bus Integration**: Cross-component event handling and persistence
//! - **Authentication Flow**: Complete auth workflows across all layers
//! - **Performance Baselines**: Automated performance regression detection

use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Common test configuration and setup utilities
pub mod common;

/// Runtime-Storage integration test suites
pub mod runtime_storage;

/// Agent lifecycle end-to-end tests
pub mod agent_lifecycle;

/// Event bus integration tests
pub mod event_bus;

/// Performance baseline and regression tests
pub mod performance;

/// Property-based testing utilities and generators
pub mod property_based;

/// Test environment management and cleanup
pub mod environment;

/// Integration test framework traits and utilities
pub use common::*;

/// Default timeout for integration tests
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Extended timeout for performance tests
pub const PERFORMANCE_TEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Test result with timing information
#[derive(Debug, Clone)]
pub struct IntegrationTestResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Performance metrics collected during tests
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operations_per_second: f64,
    pub memory_usage_mb: f64,
    pub latency_p95_ms: f64,
    pub error_rate: f64,
}

/// Integration test trait for standardized test execution
#[async_trait::async_trait]
pub trait IntegrationTest {
    /// Test name for reporting
    fn name(&self) -> &str;
    
    /// Test description
    fn description(&self) -> &str;
    
    /// Set up test environment
    async fn setup(&self) -> Result<TestEnvironment>;
    
    /// Execute the test
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult>;
    
    /// Clean up test environment
    async fn cleanup(&self, env: TestEnvironment) -> Result<()>;
    
    /// Run the complete test with timeout and error handling
    async fn run(&self) -> IntegrationTestResult {
        let start_time = std::time::Instant::now();
        let test_name = self.name().to_string();
        
        // Setup phase
        let env = match timeout(DEFAULT_TEST_TIMEOUT, self.setup()).await {
            Ok(Ok(env)) => env,
            Ok(Err(e)) => {
                return IntegrationTestResult {
                    test_name,
                    success: false,
                    duration: start_time.elapsed(),
                    error_message: Some(format!("Setup failed: {}", e)),
                    performance_metrics: None,
                };
            }
            Err(_) => {
                return IntegrationTestResult {
                    test_name,
                    success: false,
                    duration: start_time.elapsed(),
                    error_message: Some("Setup timeout".to_string()),
                    performance_metrics: None,
                };
            }
        };
        
        // Execute phase
        let result = match timeout(DEFAULT_TEST_TIMEOUT, self.execute(&env)).await {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let _ = self.cleanup(env).await;
                return IntegrationTestResult {
                    test_name,
                    success: false,
                    duration: start_time.elapsed(),
                    error_message: Some(format!("Execution failed: {}", e)),
                    performance_metrics: None,
                };
            }
            Err(_) => {
                let _ = self.cleanup(env).await;
                return IntegrationTestResult {
                    test_name,
                    success: false,
                    duration: start_time.elapsed(),
                    error_message: Some("Execution timeout".to_string()),
                    performance_metrics: None,
                };
            }
        };
        
        // Cleanup phase
        if let Err(e) = self.cleanup(env).await {
            eprintln!("Warning: Cleanup failed for test {}: {}", test_name, e);
        }
        
        IntegrationTestResult {
            test_name,
            success: result.success,
            duration: start_time.elapsed(),
            error_message: if result.success { None } else { Some(result.error) },
            performance_metrics: result.performance_metrics,
        }
    }
}

/// Test result from execution phase
#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub error: String,
    pub performance_metrics: Option<PerformanceMetrics>,
}

impl TestResult {
    pub fn success() -> Self {
        Self {
            success: true,
            error: String::new(),
            performance_metrics: None,
        }
    }
    
    pub fn success_with_metrics(metrics: PerformanceMetrics) -> Self {
        Self {
            success: true,
            error: String::new(),
            performance_metrics: Some(metrics),
        }
    }
    
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
            performance_metrics: None,
        }
    }
}

/// Macro for running a collection of integration tests
#[macro_export]
macro_rules! run_integration_tests {
    ($($test:expr),* $(,)?) => {{
        let mut results = Vec::new();
        $(
            let result = $test.run().await;
            println!("{}: {}", 
                result.test_name,
                if result.success { "✅ PASS" } else { "❌ FAIL" }
            );
            if let Some(error) = &result.error_message {
                println!("  Error: {}", error);
            }
            if let Some(metrics) = &result.performance_metrics {
                println!("  Performance: {:.2} ops/sec, {:.2}MB memory, {:.2}ms p95", 
                    metrics.operations_per_second,
                    metrics.memory_usage_mb,
                    metrics.latency_p95_ms
                );
            }
            results.push(result);
        )*
        results
    }};
}

/// Utility function to generate test data
pub fn generate_test_id() -> String {
    format!("test-{}", Uuid::new_v4())
}

/// Utility function to create test configuration
pub fn create_test_config() -> serde_json::Value {
    serde_json::json!({
        "test_mode": true,
        "timeout_ms": 30000,
        "max_retries": 3,
        "debug": true
    })
}