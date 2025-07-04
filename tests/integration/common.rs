//! Common utilities and environment management for integration tests

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test environment configuration and state management
#[derive(Debug)]
pub struct TestEnvironment {
    /// Unique test environment ID
    pub id: String,
    
    /// Temporary directory for test data
    pub temp_dir: TempDir,
    
    /// Test configuration values
    pub config: Value,
    
    /// Shared test state
    pub state: Arc<RwLock<HashMap<String, Value>>>,
    
    /// Runtime components (when needed)
    pub runtime_handle: Option<tokio::runtime::Handle>,
    
    /// Test metrics collector
    pub metrics: TestMetrics,
}

impl TestEnvironment {
    /// Create a new test environment with default configuration
    pub fn new(test_id: impl Into<String>) -> Result<Self> {
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory for test environment")?;
        
        let config = serde_json::json!({
            "storage": {
                "backend": "memory",
                "path": temp_dir.path().join("test.db"),
                "max_connections": 10
            },
            "runtime": {
                "max_agents": 100,
                "task_timeout_ms": 30000,
                "cleanup_interval_ms": 5000
            },
            "testing": {
                "deterministic_ids": true,
                "mock_time": false,
                "trace_level": "debug"
            }
        });
        
        Ok(Self {
            id: test_id.into(),
            temp_dir,
            config,
            state: Arc::new(RwLock::new(HashMap::new())),
            runtime_handle: None,
            metrics: TestMetrics::new(),
        })
    }
    
    /// Create test environment with custom configuration
    pub fn with_config(test_id: impl Into<String>, config: Value) -> Result<Self> {
        let mut env = Self::new(test_id)?;
        env.config = config;
        Ok(env)
    }
    
    /// Get a configuration value by path (e.g., "storage.backend")
    pub fn get_config(&self, path: &str) -> Option<&Value> {
        path.split('.')
            .fold(Some(&self.config), |acc, key| {
                acc?.as_object()?.get(key)
            })
    }
    
    /// Set a value in the shared test state
    pub async fn set_state(&self, key: impl Into<String>, value: Value) {
        let mut state = self.state.write().await;
        state.insert(key.into(), value);
    }
    
    /// Get a value from the shared test state
    pub async fn get_state(&self, key: &str) -> Option<Value> {
        let state = self.state.read().await;
        state.get(key).cloned()
    }
    
    /// Get the temporary directory path for test data
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
    
    /// Create a sub-directory in the test environment
    pub fn create_subdir(&self, name: &str) -> Result<PathBuf> {
        let path = self.temp_path().join(name);
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create subdirectory: {}", name))?;
        Ok(path)
    }
    
    /// Record a metric for this test environment
    pub fn record_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.record(name.into(), value);
    }
    
    /// Get all recorded metrics
    pub fn get_metrics(&self) -> &TestMetrics {
        &self.metrics
    }
}

/// Test metrics collection and analysis
#[derive(Debug, Clone)]
pub struct TestMetrics {
    metrics: HashMap<String, Vec<f64>>,
    start_time: std::time::Instant,
}

impl TestMetrics {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Record a metric value
    pub fn record(&mut self, name: String, value: f64) {
        self.metrics.entry(name).or_insert_with(Vec::new).push(value);
    }
    
    /// Get the latest value for a metric
    pub fn latest(&self, name: &str) -> Option<f64> {
        self.metrics.get(name)?.last().copied()
    }
    
    /// Get the average value for a metric
    pub fn average(&self, name: &str) -> Option<f64> {
        let values = self.metrics.get(name)?;
        if values.is_empty() {
            return None;
        }
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
    
    /// Get the 95th percentile for a metric
    pub fn percentile_95(&self, name: &str) -> Option<f64> {
        let mut values = self.metrics.get(name)?.clone();
        if values.is_empty() {
            return None;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (values.len() as f64 * 0.95) as usize;
        values.get(index).copied()
    }
    
    /// Get elapsed time since metrics started
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Export metrics to performance metrics structure
    pub fn to_performance_metrics(&self) -> Option<crate::PerformanceMetrics> {
        Some(crate::PerformanceMetrics {
            operations_per_second: self.average("ops_per_second").unwrap_or(0.0),
            memory_usage_mb: self.latest("memory_mb").unwrap_or(0.0),
            latency_p95_ms: self.percentile_95("latency_ms").unwrap_or(0.0),
            error_rate: self.average("error_rate").unwrap_or(0.0),
        })
    }
}

impl Default for TestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring for integration tests
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    operation_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    start_time: std::time::Instant,
    memory_usage_mb: Arc<AtomicU64>, // Stored as integer MB
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            operation_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
            memory_usage_mb: Arc::new(AtomicU64::new(50)), // Start with 50MB baseline
        }
    }
    
    /// Record a successful operation
    pub fn record_operation(&self) {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record an error
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get operations per second
    pub fn ops_per_second(&self) -> f64 {
        let operations = self.operation_count.load(Ordering::Relaxed) as f64;
        let duration = self.start_time.elapsed().as_secs_f64();
        
        if duration > 0.0 {
            operations / duration
        } else {
            0.0
        }
    }
    
    /// Get current memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_mb.load(Ordering::Relaxed) as f64
    }
    
    /// Update memory usage (simulated)
    pub fn update_memory_usage(&self, mb: f64) {
        self.memory_usage_mb.store(mb as u64, Ordering::Relaxed);
    }
    
    /// Get error rate (0.0 to 1.0)
    pub fn error_rate(&self) -> f64 {
        let errors = self.error_count.load(Ordering::Relaxed) as f64;
        let operations = self.operation_count.load(Ordering::Relaxed) as f64;
        
        if operations > 0.0 {
            errors / operations
        } else {
            0.0
        }
    }
    
    /// Get total operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count.load(Ordering::Relaxed)
    }
    
    /// Get total error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
    
    /// Get elapsed time since monitoring started
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Reset all counters
    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        // Note: start_time is not reset to maintain reference point
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data factory for generating consistent test objects
#[derive(Debug, Clone)]
pub struct TestDataFactory {
    counter: Arc<AtomicUsize>,
}

impl TestDataFactory {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Generate a unique test ID
    pub fn next_id(&self) -> usize {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Create a test agent configuration
    pub fn create_agent_config(&self, name: Option<&str>) -> Value {
        let id = self.next_id();
        serde_json::json!({
            "agent_id": format!("test-agent-{}", id),
            "name": name.unwrap_or(&format!("Test Agent {}", id)),
            "capabilities": ["test", "integration"],
            "max_memory": "64MB",
            "timeout": "30s",
            "priority": "normal"
        })
    }
    
    /// Create a test task configuration
    pub fn create_task_config(&self, task_type: &str) -> Value {
        let id = self.next_id();
        serde_json::json!({
            "task_id": format!("test-task-{}", id),
            "task_type": task_type,
            "description": format!("Test task {} of type {}", id, task_type),
            "timeout_ms": 10000,
            "retries": 3,
            "priority": "normal"
        })
    }
    
    /// Create test event data
    pub fn create_event_data(&self, event_type: &str) -> Value {
        let id = self.next_id();
        serde_json::json!({
            "event_id": format!("test-event-{}", id),
            "event_type": event_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": {
                "test_sequence": id,
                "event_data": format!("Test data for {}", event_type)
            },
            "metadata": {
                "source": "integration_test",
                "test_run": true
            }
        })
    }
    
    /// Create test configuration object
    pub fn create_test_config(&self, config_type: &str) -> Value {
        let id = self.next_id();
        serde_json::json!({
            "config_id": format!("test-config-{}", id),
            "config_type": config_type,
            "version": "1.0",
            "settings": {
                "test_mode": true,
                "sequence": id,
                "timeout_ms": 5000
            }
        })
    }
}

impl Default for TestDataFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Test result builder for creating consistent test results
pub struct TestResultBuilder {
    success: bool,
    error_message: Option<String>,
    metrics: Option<crate::PerformanceMetrics>,
}

impl TestResultBuilder {
    pub fn new() -> Self {
        Self {
            success: true,
            error_message: None,
            metrics: None,
        }
    }
    
    pub fn success() -> Self {
        Self::new()
    }
    
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            error_message: Some(message.into()),
            metrics: None,
        }
    }
    
    pub fn with_metrics(mut self, metrics: crate::PerformanceMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }
    
    pub fn build(self) -> crate::TestResult {
        crate::TestResult {
            success: self.success,
            error: self.error_message.unwrap_or_default(),
            performance_metrics: self.metrics,
        }
    }
}

impl Default for TestResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common test operations
pub mod helpers {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    /// Simulate workload with configurable characteristics
    pub async fn simulate_workload(
        operations: usize,
        operation_delay_ms: u64,
        error_rate: f64,
        monitor: &PerformanceMonitor,
    ) -> Result<()> {
        for i in 0..operations {
            // Simulate operation delay
            sleep(Duration::from_millis(operation_delay_ms)).await;
            
            // Simulate occasional errors based on error rate
            if (i as f64 / operations as f64) < error_rate {
                monitor.record_error();
            } else {
                monitor.record_operation();
            }
        }
        
        Ok(())
    }
    
    /// Create a stress test scenario
    pub async fn stress_test_scenario(
        concurrent_workers: usize,
        operations_per_worker: usize,
        base_delay_ms: u64,
        factory: &TestDataFactory,
        monitor: &PerformanceMonitor,
    ) -> Result<()> {
        let mut handles = Vec::new();
        
        for worker_id in 0..concurrent_workers {
            let factory_clone = factory.clone();
            let monitor_clone = monitor.clone();
            
            let handle = tokio::spawn(async move {
                for operation in 0..operations_per_worker {
                    // Create test data
                    let _event = factory_clone.create_event_data("stress_test");
                    
                    // Variable delay to simulate real-world conditions
                    let delay = base_delay_ms + (worker_id as u64 % 10);
                    sleep(Duration::from_millis(delay)).await;
                    
                    // Occasional errors in stress conditions
                    if operation % 50 == 0 && worker_id % 3 == 0 {
                        monitor_clone.record_error();
                    } else {
                        monitor_clone.record_operation();
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            handle.await.context("Stress test worker failed")?;
        }
        
        Ok(())
    }
    
    /// Validate performance against thresholds
    pub fn validate_performance_thresholds(
        metrics: &crate::PerformanceMetrics,
        min_ops_per_second: f64,
        max_latency_ms: f64,
        max_memory_mb: f64,
        max_error_rate: f64,
    ) -> Result<()> {
        if metrics.operations_per_second < min_ops_per_second {
            anyhow::bail!(
                "Operations per second {} below threshold {}",
                metrics.operations_per_second,
                min_ops_per_second
            );
        }
        
        if metrics.latency_p95_ms > max_latency_ms {
            anyhow::bail!(
                "Latency {} ms exceeds threshold {} ms",
                metrics.latency_p95_ms,
                max_latency_ms
            );
        }
        
        if metrics.memory_usage_mb > max_memory_mb {
            anyhow::bail!(
                "Memory usage {} MB exceeds threshold {} MB",
                metrics.memory_usage_mb,
                max_memory_mb
            );
        }
        
        if metrics.error_rate > max_error_rate {
            anyhow::bail!(
                "Error rate {} exceeds threshold {}",
                metrics.error_rate,
                max_error_rate
            );
        }
        
        Ok(())
    }
}

/// Assertion utilities for integration tests
pub mod assertions {
    use super::*;
    
    /// Assert that a test environment is properly configured
    pub fn assert_environment_valid(env: &TestEnvironment) -> Result<()> {
        if env.id.is_empty() {
            anyhow::bail!("Test environment ID cannot be empty");
        }
        
        if !env.temp_path().exists() {
            anyhow::bail!("Test environment temporary directory does not exist");
        }
        
        if env.config.is_null() {
            anyhow::bail!("Test environment configuration cannot be null");
        }
        
        Ok(())
    }
    
    /// Assert that performance metrics meet expected thresholds
    pub fn assert_performance_thresholds(
        metrics: &crate::PerformanceMetrics,
        min_ops_per_second: f64,
        max_memory_mb: f64,
        max_latency_p95_ms: f64,
        max_error_rate: f64,
    ) -> Result<()> {
        if metrics.operations_per_second < min_ops_per_second {
            anyhow::bail!(
                "Operations per second {} below threshold {}",
                metrics.operations_per_second,
                min_ops_per_second
            );
        }
        
        if metrics.memory_usage_mb > max_memory_mb {
            anyhow::bail!(
                "Memory usage {}MB exceeds threshold {}MB",
                metrics.memory_usage_mb,
                max_memory_mb
            );
        }
        
        if metrics.latency_p95_ms > max_latency_p95_ms {
            anyhow::bail!(
                "95th percentile latency {}ms exceeds threshold {}ms",
                metrics.latency_p95_ms,
                max_latency_p95_ms
            );
        }
        
        if metrics.error_rate > max_error_rate {
            anyhow::bail!(
                "Error rate {} exceeds threshold {}",
                metrics.error_rate,
                max_error_rate
            );
        }
        
        Ok(())
    }
    
    /// Assert that memory usage is within reasonable bounds
    pub fn assert_memory_bounds(current_mb: f64, baseline_mb: f64, max_growth_percent: f64) -> Result<()> {
        let growth_percent = ((current_mb - baseline_mb) / baseline_mb) * 100.0;
        
        if growth_percent > max_growth_percent {
            anyhow::bail!(
                "Memory growth {:.1}% exceeds threshold {:.1}%",
                growth_percent,
                max_growth_percent
            );
        }
        
        Ok(())
    }
    
    /// Assert that operation throughput meets minimum requirements
    pub fn assert_throughput_requirements(
        actual_ops_per_sec: f64,
        required_ops_per_sec: f64,
        tolerance_percent: f64,
    ) -> Result<()> {
        let min_required = required_ops_per_sec * (1.0 - tolerance_percent / 100.0);
        
        if actual_ops_per_sec < min_required {
            anyhow::bail!(
                "Throughput {:.2} ops/sec below minimum {:.2} ops/sec ({}% tolerance)",
                actual_ops_per_sec,
                min_required,
                tolerance_percent
            );
        }
        
        Ok(())
    }
}