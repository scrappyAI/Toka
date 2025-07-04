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
//! - **Property-Based Testing**: Kernel invariants and behavioral properties
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
        self.run_with_timeout(DEFAULT_TEST_TIMEOUT).await
    }
    
    /// Run the test with custom timeout
    async fn run_with_timeout(&self, test_timeout: Duration) -> IntegrationTestResult {
        let start_time = std::time::Instant::now();
        let test_name = self.name().to_string();
        
        // Setup phase
        let env = match timeout(test_timeout, self.setup()).await {
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
        let result = match timeout(test_timeout, self.execute(&env)).await {
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

/// Test suite configuration and management
#[derive(Debug, Clone)]
pub struct TestSuiteConfig {
    pub name: String,
    pub description: String,
    pub parallel_execution: bool,
    pub timeout_override: Option<Duration>,
    pub required_setup: Vec<String>,
}

impl TestSuiteConfig {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parallel_execution: false,
            timeout_override: None,
            required_setup: Vec::new(),
        }
    }
    
    pub fn with_parallel_execution(mut self) -> Self {
        self.parallel_execution = true;
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_override = Some(timeout);
        self
    }
    
    pub fn with_setup_requirement(mut self, requirement: impl Into<String>) -> Self {
        self.required_setup.push(requirement.into());
        self
    }
}

/// Comprehensive test runner for all integration test suites
pub struct IntegrationTestRunner {
    suites: Vec<(TestSuiteConfig, Vec<Box<dyn IntegrationTest + Send + Sync>>)>,
}

impl IntegrationTestRunner {
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
        }
    }
    
    /// Add a test suite to the runner
    pub fn add_suite(
        mut self,
        config: TestSuiteConfig,
        tests: Vec<Box<dyn IntegrationTest + Send + Sync>>,
    ) -> Self {
        self.suites.push((config, tests));
        self
    }
    
    /// Run all test suites
    pub async fn run_all(&self) -> IntegrationTestReport {
        let overall_start = std::time::Instant::now();
        let mut suite_results = Vec::new();
        
        println!("🚀 Starting Toka OS v0.3.0 Integration Test Suite");
        println!("=========================================================");
        
        for (config, tests) in &self.suites {
            println!("\n📋 Running Suite: {}", config.name);
            println!("   Description: {}", config.description);
            println!("   Tests: {}", tests.len());
            
            let suite_start = std::time::Instant::now();
            let mut test_results = Vec::new();
            
            if config.parallel_execution {
                // Run tests in parallel
                let mut handles = Vec::new();
                
                for test in tests {
                    let timeout = config.timeout_override.unwrap_or(DEFAULT_TEST_TIMEOUT);
                    let handle = tokio::spawn(async move {
                        test.run_with_timeout(timeout).await
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    match handle.await {
                        Ok(result) => test_results.push(result),
                        Err(e) => {
                            test_results.push(IntegrationTestResult {
                                test_name: "unknown".to_string(),
                                success: false,
                                duration: Duration::from_secs(0),
                                error_message: Some(format!("Task join error: {}", e)),
                                performance_metrics: None,
                            });
                        }
                    }
                }
            } else {
                // Run tests sequentially
                for test in tests {
                    let timeout = config.timeout_override.unwrap_or(DEFAULT_TEST_TIMEOUT);
                    let result = test.run_with_timeout(timeout).await;
                    
                    // Print individual test result
                    let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
                    println!("     {}: {}", result.test_name, status);
                    
                    if let Some(error) = &result.error_message {
                        println!("       Error: {}", error);
                    }
                    
                    test_results.push(result);
                }
            }
            
            let suite_duration = suite_start.elapsed();
            let passed = test_results.iter().filter(|r| r.success).count();
            let failed = test_results.iter().filter(|r| !r.success).count();
            
            println!("   Results: {} passed, {} failed, Duration: {:?}", passed, failed, suite_duration);
            
            suite_results.push(TestSuiteResult {
                config: config.clone(),
                test_results,
                duration: suite_duration,
            });
        }
        
        let overall_duration = overall_start.elapsed();
        
        IntegrationTestReport {
            suite_results,
            overall_duration,
        }
    }
    
    /// Create the default test runner with all standard test suites
    pub fn with_all_suites() -> Self {
        Self::new()
            .add_suite(
                TestSuiteConfig::new("Runtime-Storage Integration", "Test runtime coordination with different storage backends"),
                runtime_storage::all_runtime_storage_tests()
            )
            .add_suite(
                TestSuiteConfig::new("Agent Lifecycle", "End-to-end agent management workflows")
                    .with_timeout(Duration::from_secs(60)),
                agent_lifecycle::all_agent_lifecycle_tests()
            )
            .add_suite(
                TestSuiteConfig::new("Property-Based Testing", "Kernel invariants and behavioral properties")
                    .with_parallel_execution(),
                property_based::all_property_based_tests()
            )
            .add_suite(
                TestSuiteConfig::new("Performance Baselines", "Performance benchmarks and regression detection")
                    .with_timeout(PERFORMANCE_TEST_TIMEOUT),
                performance::all_performance_tests()
            )
    }
}

impl Default for IntegrationTestRunner {
    fn default() -> Self {
        Self::with_all_suites()
    }
}

/// Test suite execution result
#[derive(Debug)]
pub struct TestSuiteResult {
    pub config: TestSuiteConfig,
    pub test_results: Vec<IntegrationTestResult>,
    pub duration: Duration,
}

impl TestSuiteResult {
    pub fn passed_count(&self) -> usize {
        self.test_results.iter().filter(|r| r.success).count()
    }
    
    pub fn failed_count(&self) -> usize {
        self.test_results.iter().filter(|r| !r.success).count()
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.test_results.is_empty() {
            return 1.0;
        }
        self.passed_count() as f64 / self.test_results.len() as f64
    }
}

/// Overall integration test report
#[derive(Debug)]
pub struct IntegrationTestReport {
    pub suite_results: Vec<TestSuiteResult>,
    pub overall_duration: Duration,
}

impl IntegrationTestReport {
    pub fn total_tests(&self) -> usize {
        self.suite_results.iter().map(|s| s.test_results.len()).sum()
    }
    
    pub fn total_passed(&self) -> usize {
        self.suite_results.iter().map(|s| s.passed_count()).sum()
    }
    
    pub fn total_failed(&self) -> usize {
        self.suite_results.iter().map(|s| s.failed_count()).sum()
    }
    
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_tests() == 0 {
            return 1.0;
        }
        self.total_passed() as f64 / self.total_tests() as f64
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 Integration Test Summary");
        println!("============================");
        println!("Total Duration: {:?}", self.overall_duration);
        println!("Test Suites: {}", self.suite_results.len());
        println!("Total Tests: {}", self.total_tests());
        println!("Passed: {} ✅", self.total_passed());
        println!("Failed: {} ❌", self.total_failed());
        println!("Success Rate: {:.1}%", self.overall_success_rate() * 100.0);
        
        println!("\n📋 Suite Breakdown:");
        for suite in &self.suite_results {
            let status = if suite.failed_count() == 0 { "✅" } else { "❌" };
            println!(
                "  {} {}: {}/{} passed ({:.1}%) - {:?}",
                status,
                suite.config.name,
                suite.passed_count(),
                suite.test_results.len(),
                suite.success_rate() * 100.0,
                suite.duration
            );
        }
        
        // Print failed tests details
        if self.total_failed() > 0 {
            println!("\n❌ Failed Tests:");
            for suite in &self.suite_results {
                for test in &suite.test_results {
                    if !test.success {
                        println!("  - {} ({}): {}", 
                            test.test_name, 
                            suite.config.name,
                            test.error_message.as_deref().unwrap_or("Unknown error")
                        );
                    }
                }
            }
        }
        
        // Print performance metrics summary
        let mut performance_metrics = Vec::new();
        for suite in &self.suite_results {
            for test in &suite.test_results {
                if let Some(metrics) = &test.performance_metrics {
                    performance_metrics.push((test.test_name.clone(), metrics.clone()));
                }
            }
        }
        
        if !performance_metrics.is_empty() {
            println!("\n📈 Performance Summary:");
            let avg_ops_per_sec: f64 = performance_metrics.iter()
                .map(|(_, m)| m.operations_per_second)
                .sum::<f64>() / performance_metrics.len() as f64;
            let avg_latency: f64 = performance_metrics.iter()
                .map(|(_, m)| m.latency_p95_ms)
                .sum::<f64>() / performance_metrics.len() as f64;
            let avg_memory: f64 = performance_metrics.iter()
                .map(|(_, m)| m.memory_usage_mb)
                .sum::<f64>() / performance_metrics.len() as f64;
            
            println!("  Average Throughput: {:.2} ops/sec", avg_ops_per_sec);
            println!("  Average Latency P95: {:.2}ms", avg_latency);
            println!("  Average Memory Usage: {:.2}MB", avg_memory);
        }
        
        println!("\n{}", if self.total_failed() == 0 { 
            "🎉 All integration tests passed!" 
        } else { 
            "⚠️  Some integration tests failed. Review the details above." 
        });
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