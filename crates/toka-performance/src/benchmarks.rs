//! Performance benchmarking and regression detection
//!
//! This module provides comprehensive performance benchmarking capabilities
//! for automated performance regression detection and baseline establishment.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

/// Performance benchmark suite
pub struct BenchmarkSuite {
    /// Component identifier
    component_id: String,
    /// Registered benchmarks
    benchmarks: RwLock<HashMap<String, Box<dyn Benchmark + Send + Sync>>>,
    /// Benchmark results history
    results_history: RwLock<HashMap<String, Vec<BenchmarkResult>>>,
    /// Performance baselines
    baselines: RwLock<HashMap<String, PerformanceBaseline>>,
}

impl std::fmt::Debug for BenchmarkSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BenchmarkSuite")
            .field("component_id", &self.component_id)
            .finish()
    }
}

/// Performance baseline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Baseline name
    pub name: String,
    /// Expected operations per second
    pub expected_ops_per_second: f64,
    /// Maximum acceptable latency (P95)
    pub max_latency_p95_ms: f64,
    /// Maximum memory usage
    pub max_memory_usage_mb: f64,
    /// Maximum error rate
    pub max_error_rate: f64,
    /// Baseline creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Operations per second
    pub ops_per_second: f64,
    /// Latency measurements
    pub latency_stats: LatencyStats,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Execution duration
    pub execution_duration: Duration,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metrics
    pub additional_metrics: HashMap<String, f64>,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Average latency
    pub avg_ms: f64,
    /// P50 latency
    pub p50_ms: f64,
    /// P95 latency
    pub p95_ms: f64,
    /// P99 latency
    pub p99_ms: f64,
    /// Maximum latency
    pub max_ms: f64,
    /// Minimum latency
    pub min_ms: f64,
}

/// Benchmark trait
#[async_trait::async_trait]
pub trait Benchmark {
    /// Get benchmark name
    fn name(&self) -> &str;
    
    /// Get benchmark description
    fn description(&self) -> &str;
    
    /// Run the benchmark
    async fn run(&self) -> Result<BenchmarkResult>;
    
    /// Setup benchmark environment
    async fn setup(&self) -> Result<()> {
        Ok(())
    }
    
    /// Cleanup benchmark environment
    async fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(component_id: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            benchmarks: RwLock::new(HashMap::new()),
            results_history: RwLock::new(HashMap::new()),
            baselines: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a benchmark
    pub async fn register_benchmark(&self, benchmark: Box<dyn Benchmark + Send + Sync>) -> Result<()> {
        let name = benchmark.name().to_string();
        
        debug!(
            component = %self.component_id,
            benchmark = %name,
            "Registering benchmark"
        );
        
        self.benchmarks.write().await.insert(name, benchmark);
        Ok(())
    }
    
    /// Run all benchmarks
    pub async fn run_all(&self) -> Result<Vec<BenchmarkResult>> {
        let benchmarks = self.benchmarks.read().await;
        let mut results = Vec::new();
        
        for (name, benchmark) in benchmarks.iter() {
            info!(
                component = %self.component_id,
                benchmark = %name,
                "Running benchmark"
            );
            
            let start_time = Instant::now();
            
            // Setup
            if let Err(e) = benchmark.setup().await {
                error!(
                    component = %self.component_id,
                    benchmark = %name,
                    error = %e,
                    "Benchmark setup failed"
                );
                continue;
            }
            
            // Run benchmark
            let result = match benchmark.run().await {
                Ok(result) => result,
                Err(e) => {
                    error!(
                        component = %self.component_id,
                        benchmark = %name,
                        error = %e,
                        "Benchmark execution failed"
                    );
                    continue;
                }
            };
            
            // Cleanup
            if let Err(e) = benchmark.cleanup().await {
                warn!(
                    component = %self.component_id,
                    benchmark = %name,
                    error = %e,
                    "Benchmark cleanup failed"
                );
            }
            
            let total_duration = start_time.elapsed();
            
            info!(
                component = %self.component_id,
                benchmark = %name,
                ops_per_second = %result.ops_per_second,
                latency_p95_ms = %result.latency_stats.p95_ms,
                memory_usage_mb = %result.memory_usage_mb,
                duration = ?total_duration,
                "Benchmark completed"
            );
            
            results.push(result.clone());
            
            // Store result in history
            self.results_history.write().await
                .entry(name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        Ok(results)
    }
    
    /// Run a specific benchmark
    pub async fn run_benchmark(&self, name: &str) -> Result<BenchmarkResult> {
        let benchmarks = self.benchmarks.read().await;
        
        let benchmark = benchmarks.get(name)
            .ok_or_else(|| anyhow::anyhow!("Benchmark '{}' not found", name))?;
        
        benchmark.setup().await?;
        let result = benchmark.run().await?;
        benchmark.cleanup().await?;
        
        // Store result in history
        self.results_history.write().await
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(result.clone());
        
        Ok(result)
    }
    
    /// Set performance baseline
    pub async fn set_baseline(&self, name: &str, baseline: PerformanceBaseline) -> Result<()> {
        self.baselines.write().await.insert(name.to_string(), baseline);
        Ok(())
    }
    
    /// Get performance baseline
    pub async fn get_baseline(&self, name: &str) -> Option<PerformanceBaseline> {
        self.baselines.read().await.get(name).cloned()
    }
    
    /// Get benchmark results history
    pub async fn get_results_history(&self, name: &str) -> Option<Vec<BenchmarkResult>> {
        self.results_history.read().await.get(name).cloned()
    }
    
    /// List all registered benchmarks
    pub async fn list_benchmarks(&self) -> Vec<String> {
        self.benchmarks.read().await.keys().cloned().collect()
    }
}

/// System performance benchmark
pub struct SystemPerformanceBenchmark {
    name: String,
    description: String,
    duration: Duration,
    target_ops: u64,
}

impl SystemPerformanceBenchmark {
    pub fn new(name: &str, description: &str, duration: Duration, target_ops: u64) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            duration,
            target_ops,
        }
    }
}

#[async_trait::async_trait]
impl Benchmark for SystemPerformanceBenchmark {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn run(&self) -> Result<BenchmarkResult> {
        let start_time = Instant::now();
        let mut operation_count = 0;
        let mut latency_measurements = Vec::new();
        let mut error_count = 0;
        
        // Run operations for the specified duration
        while start_time.elapsed() < self.duration {
            let op_start = Instant::now();
            
            // Simulate operation
            let success = self.simulate_operation().await;
            
            let op_duration = op_start.elapsed();
            latency_measurements.push(op_duration.as_secs_f64() * 1000.0); // Convert to ms
            
            if success {
                operation_count += 1;
            } else {
                error_count += 1;
            }
            
            // Small delay to prevent overwhelming the system
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        
        let total_duration = start_time.elapsed();
        let ops_per_second = operation_count as f64 / total_duration.as_secs_f64();
        let error_rate = error_count as f64 / (operation_count + error_count) as f64;
        
        // Calculate latency statistics
        latency_measurements.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let latency_stats = self.calculate_latency_stats(&latency_measurements);
        
        // Simulate memory usage
        let memory_usage_mb = 50.0 + (operation_count as f64 * 0.01);
        
        Ok(BenchmarkResult {
            name: self.name.clone(),
            ops_per_second,
            latency_stats,
            memory_usage_mb,
            error_rate,
            execution_duration: total_duration,
            timestamp: Utc::now(),
            additional_metrics: HashMap::new(),
        })
    }
}

impl SystemPerformanceBenchmark {
    async fn simulate_operation(&self) -> bool {
        // Simulate some work
        let work_duration = Duration::from_micros(fastrand::u64(50..=500));
        tokio::time::sleep(work_duration).await;
        
        // Simulate occasional failures (5% error rate)
        fastrand::f64() > 0.05
    }
    
    fn calculate_latency_stats(&self, measurements: &[f64]) -> LatencyStats {
        if measurements.is_empty() {
            return LatencyStats {
                avg_ms: 0.0,
                p50_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                max_ms: 0.0,
                min_ms: 0.0,
            };
        }
        
        let len = measurements.len();
        let avg_ms = measurements.iter().sum::<f64>() / len as f64;
        let p50_ms = measurements[len * 50 / 100];
        let p95_ms = measurements[len * 95 / 100];
        let p99_ms = measurements[len * 99 / 100];
        let max_ms = measurements.iter().fold(0.0_f64, |a, &b| a.max(b));
        let min_ms = measurements.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        LatencyStats {
            avg_ms,
            p50_ms,
            p95_ms,
            p99_ms,
            max_ms,
            min_ms,
        }
    }
}

/// Benchmark error types
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    /// Benchmark not found
    #[error("Benchmark '{0}' not found")]
    NotFound(String),
    
    /// Benchmark execution failed
    #[error("Benchmark execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Setup failed
    #[error("Benchmark setup failed: {0}")]
    SetupFailed(String),
    
    /// Cleanup failed
    #[error("Benchmark cleanup failed: {0}")]
    CleanupFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new("test-component");
        assert_eq!(suite.component_id, "test-component");
    }
    
    #[tokio::test]
    async fn test_benchmark_registration() {
        let suite = BenchmarkSuite::new("test-component");
        let benchmark = Box::new(SystemPerformanceBenchmark::new(
            "test_benchmark",
            "A test benchmark",
            Duration::from_secs(1),
            1000,
        ));
        
        let result = suite.register_benchmark(benchmark).await;
        assert!(result.is_ok());
        
        let benchmarks = suite.list_benchmarks().await;
        assert!(benchmarks.contains(&"test_benchmark".to_string()));
    }
    
    #[tokio::test]
    async fn test_benchmark_execution() {
        let suite = BenchmarkSuite::new("test-component");
        let benchmark = Box::new(SystemPerformanceBenchmark::new(
            "test_benchmark",
            "A test benchmark",
            Duration::from_millis(100),
            100,
        ));
        
        suite.register_benchmark(benchmark).await.unwrap();
        
        let result = suite.run_benchmark("test_benchmark").await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.name, "test_benchmark");
        assert!(result.ops_per_second > 0.0);
    }
}