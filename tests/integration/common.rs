//! Common utilities and environment management for integration tests

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
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

/// Test data factory for generating consistent test objects
pub struct TestDataFactory {
    counter: std::sync::atomic::AtomicU64,
}

impl TestDataFactory {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// Generate a unique test ID
    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
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
}

impl Default for TestDataFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring utilities for tests
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    operation_count: std::sync::atomic::AtomicU64,
    error_count: std::sync::atomic::AtomicU64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            operation_count: std::sync::atomic::AtomicU64::new(0),
            error_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// Record a successful operation
    pub fn record_operation(&self) {
        self.operation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Record an error
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Calculate operations per second
    pub fn ops_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.operation_count.load(std::sync::atomic::Ordering::Relaxed) as f64 / elapsed
    }
    
    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        let total_ops = self.operation_count.load(std::sync::atomic::Ordering::Relaxed);
        let errors = self.error_count.load(std::sync::atomic::Ordering::Relaxed);
        
        if total_ops == 0 {
            return 0.0;
        }
        
        errors as f64 / total_ops as f64
    }
    
    /// Get memory usage in MB (simplified implementation)
    pub fn memory_usage_mb(&self) -> f64 {
        // This is a simplified implementation
        // In a real scenario, you'd want to use more sophisticated memory tracking
        let process_info = std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output();
            
        if let Ok(output) = process_info {
            if let Ok(rss_str) = String::from_utf8(output.stdout) {
                if let Ok(rss_kb) = rss_str.trim().parse::<f64>() {
                    return rss_kb / 1024.0; // Convert KB to MB
                }
            }
        }
        
        0.0 // Fallback if memory detection fails
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for test assertions and validation
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
}