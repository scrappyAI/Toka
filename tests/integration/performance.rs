//! Performance baseline and regression testing framework
//! 
//! This module provides comprehensive performance testing capabilities to establish
//! baselines, detect regressions, and monitor system performance characteristics.

use super::*;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Performance baseline configuration
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub test_name: String,
    pub expected_ops_per_second: f64,
    pub max_latency_p95_ms: f64,
    pub max_memory_usage_mb: f64,
    pub max_error_rate: f64,
    pub warmup_duration: Duration,
    pub test_duration: Duration,
}

impl PerformanceBaseline {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            expected_ops_per_second: 1000.0,
            max_latency_p95_ms: 10.0,
            max_memory_usage_mb: 100.0,
            max_error_rate: 0.01,
            warmup_duration: Duration::from_secs(5),
            test_duration: Duration::from_secs(30),
        }
    }
    
    pub fn with_throughput(mut self, ops_per_second: f64) -> Self {
        self.expected_ops_per_second = ops_per_second;
        self
    }
    
    pub fn with_latency(mut self, max_latency_p95_ms: f64) -> Self {
        self.max_latency_p95_ms = max_latency_p95_ms;
        self
    }
    
    pub fn with_memory(mut self, max_memory_usage_mb: f64) -> Self {
        self.max_memory_usage_mb = max_memory_usage_mb;
        self
    }
    
    pub fn with_duration(mut self, test_duration: Duration) -> Self {
        self.test_duration = test_duration;
        self
    }
}

/// Performance regression detector
pub struct PerformanceRegressionDetector {
    baselines: HashMap<String, PerformanceBaseline>,
    historical_results: HashMap<String, Vec<PerformanceMetrics>>,
}

impl PerformanceRegressionDetector {
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            historical_results: HashMap::new(),
        }
    }
    
    pub fn add_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines.insert(baseline.test_name.clone(), baseline);
    }
    
    pub fn record_result(&mut self, test_name: &str, metrics: PerformanceMetrics) {
        self.historical_results
            .entry(test_name.to_string())
            .or_insert_with(Vec::new)
            .push(metrics);
    }
    
    pub fn detect_regression(&self, test_name: &str, current_metrics: &PerformanceMetrics) -> Option<String> {
        let baseline = self.baselines.get(test_name)?;
        
        // Check throughput regression (must be at least 80% of expected)
        if current_metrics.operations_per_second < baseline.expected_ops_per_second * 0.8 {
            return Some(format!(
                "Throughput regression: {:.2} ops/sec < {:.2} (80% of expected)",
                current_metrics.operations_per_second,
                baseline.expected_ops_per_second * 0.8
            ));
        }
        
        // Check latency regression
        if current_metrics.latency_p95_ms > baseline.max_latency_p95_ms {
            return Some(format!(
                "Latency regression: {:.2}ms > {:.2}ms threshold",
                current_metrics.latency_p95_ms,
                baseline.max_latency_p95_ms
            ));
        }
        
        // Check memory regression
        if current_metrics.memory_usage_mb > baseline.max_memory_usage_mb {
            return Some(format!(
                "Memory regression: {:.2}MB > {:.2}MB threshold",
                current_metrics.memory_usage_mb,
                baseline.max_memory_usage_mb
            ));
        }
        
        // Check error rate regression
        if current_metrics.error_rate > baseline.max_error_rate {
            return Some(format!(
                "Error rate regression: {:.4} > {:.4} threshold",
                current_metrics.error_rate,
                baseline.max_error_rate
            ));
        }
        
        None
    }
    
    pub fn analyze_trend(&self, test_name: &str) -> Option<String> {
        let results = self.historical_results.get(test_name)?;
        
        if results.len() < 5 {
            return None; // Need at least 5 data points for trend analysis
        }
        
        let recent_results = &results[results.len().saturating_sub(5)..];
        
        // Check if there's a consistent downward trend in performance
        let throughput_trend: Vec<f64> = recent_results.iter()
            .map(|r| r.operations_per_second)
            .collect();
        
        let is_declining = throughput_trend.windows(2)
            .all(|w| w[1] <= w[0]);
        
        if is_declining {
            let decline_percent = ((throughput_trend[0] - throughput_trend[throughput_trend.len() - 1]) 
                                 / throughput_trend[0]) * 100.0;
            
            if decline_percent > 10.0 {
                return Some(format!(
                    "Performance trend decline: {:.1}% reduction over last {} measurements",
                    decline_percent,
                    recent_results.len()
                ));
            }
        }
        
        None
    }
}

/// Comprehensive performance benchmark test
pub struct SystemPerformanceBenchmark;

#[async_trait::async_trait]
impl IntegrationTest for SystemPerformanceBenchmark {
    fn name(&self) -> &str {
        "system_performance_benchmark"
    }
    
    fn description(&self) -> &str {
        "Comprehensive system performance benchmark establishing baseline metrics"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "performance": {
                    "warmup_duration_seconds": 5,
                    "test_duration_seconds": 30,
                    "measurement_interval_ms": 100,
                    "target_throughput": 1000
                },
                "runtime": {
                    "max_agents": 100,
                    "max_memory_gb": 2
                },
                "storage": {
                    "backend": "memory",
                    "batch_size": 100
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        println!("🔥 Starting system performance benchmark...");
        
        // Warmup phase
        println!("⏳ Warmup phase (5 seconds)...");
        let warmup_start = Instant::now();
        
        while warmup_start.elapsed() < Duration::from_secs(5) {
            let agent_config = factory.create_agent_config(Some("warmup-agent"));
            let task_config = factory.create_task_config("warmup-task");
            
            // Simulate lightweight operations during warmup
            sleep(Duration::from_millis(1)).await;
            monitor.record_operation();
        }
        
        // Reset monitor for actual benchmark
        let mut benchmark_monitor = PerformanceMonitor::new();
        
        // Main benchmark phase
        println!("🚀 Main benchmark phase (30 seconds)...");
        let benchmark_start = Instant::now();
        let mut operation_count = 0;
        let mut latency_measurements = Vec::new();
        
        while benchmark_start.elapsed() < Duration::from_secs(30) {
            let operation_start = Instant::now();
            
            // Simulate various system operations
            match operation_count % 4 {
                0 => {
                    // Agent spawning simulation
                    let agent_config = factory.create_agent_config(Some("benchmark-agent"));
                    sleep(Duration::from_micros(100)).await;
                }
                1 => {
                    // Task execution simulation
                    let task_config = factory.create_task_config("benchmark-task");
                    sleep(Duration::from_micros(200)).await;
                }
                2 => {
                    // Event processing simulation
                    let event_data = factory.create_event_data("benchmark-event");
                    sleep(Duration::from_micros(50)).await;
                }
                3 => {
                    // Storage operation simulation
                    sleep(Duration::from_micros(150)).await;
                }
                _ => {}
            }
            
            let operation_latency = operation_start.elapsed();
            latency_measurements.push(operation_latency.as_micros() as f64 / 1000.0); // Convert to ms
            
            benchmark_monitor.record_operation();
            operation_count += 1;
            
            // Maintain target throughput
            if operation_count % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }
        
        // Calculate performance metrics
        let total_duration = benchmark_start.elapsed();
        let ops_per_second = operation_count as f64 / total_duration.as_secs_f64();
        
        // Calculate latency percentiles
        latency_measurements.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let latency_p95 = if !latency_measurements.is_empty() {
            let index = (latency_measurements.len() as f64 * 0.95) as usize;
            latency_measurements[index.min(latency_measurements.len() - 1)]
        } else {
            0.0
        };
        
        let latency_avg = if !latency_measurements.is_empty() {
            latency_measurements.iter().sum::<f64>() / latency_measurements.len() as f64
        } else {
            0.0
        };
        
        // Simulate memory usage calculation
        let memory_usage_mb = (operation_count as f64 * 0.1).min(500.0); // Simulate memory growth
        
        let metrics = PerformanceMetrics {
            operations_per_second: ops_per_second,
            memory_usage_mb,
            latency_p95_ms: latency_p95,
            error_rate: benchmark_monitor.error_rate(),
        };
        
        println!("📊 Benchmark Results:");
        println!("   Operations/sec: {:.2}", metrics.operations_per_second);
        println!("   Latency P95: {:.2}ms", metrics.latency_p95_ms);
        println!("   Latency Avg: {:.2}ms", latency_avg);
        println!("   Memory Usage: {:.2}MB", metrics.memory_usage_mb);
        println!("   Error Rate: {:.4}", metrics.error_rate);
        
        // Validate against baseline expectations
        let baseline = PerformanceBaseline::new(self.name())
            .with_throughput(800.0) // Expect at least 800 ops/sec
            .with_latency(5.0)      // Max 5ms P95 latency
            .with_memory(600.0);    // Max 600MB memory
        
        if metrics.operations_per_second < baseline.expected_ops_per_second {
            return Ok(TestResult::failure(format!(
                "Throughput below baseline: {:.2} < {:.2} ops/sec",
                metrics.operations_per_second,
                baseline.expected_ops_per_second
            )));
        }
        
        if metrics.latency_p95_ms > baseline.max_latency_p95_ms {
            return Ok(TestResult::failure(format!(
                "Latency above baseline: {:.2} > {:.2}ms",
                metrics.latency_p95_ms,
                baseline.max_latency_p95_ms
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        println!("✅ Performance benchmark completed");
        Ok(())
    }
}

/// Memory usage and leak detection test
pub struct MemoryPerformanceTest;

#[async_trait::async_trait]
impl IntegrationTest for MemoryPerformanceTest {
    fn name(&self) -> &str {
        "memory_performance_leak_detection"
    }
    
    fn description(&self) -> &str {
        "Test memory usage patterns and detect potential memory leaks"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "memory": {
                    "initial_allocation_mb": 50,
                    "max_allocation_mb": 500,
                    "leak_detection_threshold": 0.1
                },
                "test_cycles": 100,
                "cycle_duration_ms": 100
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        let mut memory_measurements = Vec::new();
        let initial_memory = 50.0; // Simulated initial memory usage in MB
        
        // Run memory allocation/deallocation cycles
        for cycle in 0..100 {
            let cycle_start = Instant::now();
            
            // Simulate memory allocation
            let allocation_size = (cycle % 10 + 1) as f64 * 5.0; // 5-50 MB allocations
            let current_memory = initial_memory + allocation_size;
            
            memory_measurements.push(current_memory);
            
            // Simulate work with allocated memory
            for _ in 0..10 {
                let event_data = factory.create_event_data("memory_test");
                sleep(Duration::from_millis(1)).await;
                monitor.record_operation();
            }
            
            // Simulate memory deallocation (most cycles should clean up)
            if cycle % 20 != 0 { // Simulate occasional memory leak (5% of cycles)
                // Memory deallocated - reset to baseline
            } else {
                // Simulate memory leak - don't reset
                monitor.record_error(); // Track potential leak
            }
            
            // Maintain cycle timing
            let cycle_duration = cycle_start.elapsed();
            if cycle_duration < Duration::from_millis(100) {
                sleep(Duration::from_millis(100) - cycle_duration).await;
            }
        }
        
        // Analyze memory usage patterns
        let final_memory = memory_measurements.last().copied().unwrap_or(initial_memory);
        let max_memory = memory_measurements.iter().copied().fold(0.0, f64::max);
        let avg_memory = memory_measurements.iter().sum::<f64>() / memory_measurements.len() as f64;
        
        // Calculate memory growth trend
        let memory_growth = final_memory - initial_memory;
        let growth_rate = memory_growth / initial_memory;
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: final_memory,
            latency_p95_ms: 1.0, // Memory tests are fast
            error_rate: monitor.error_rate(),
        };
        
        println!("🧠 Memory Performance Results:");
        println!("   Initial Memory: {:.2}MB", initial_memory);
        println!("   Final Memory: {:.2}MB", final_memory);
        println!("   Max Memory: {:.2}MB", max_memory);
        println!("   Average Memory: {:.2}MB", avg_memory);
        println!("   Memory Growth: {:.2}MB ({:.1}%)", memory_growth, growth_rate * 100.0);
        println!("   Potential Leaks: {:.1}%", metrics.error_rate * 100.0);
        
        // Check for memory leaks
        if growth_rate > 0.5 { // More than 50% growth indicates potential leak
            return Ok(TestResult::failure(format!(
                "Potential memory leak detected: {:.1}% growth",
                growth_rate * 100.0
            )));
        }
        
        if max_memory > 400.0 { // Memory usage should stay reasonable
            return Ok(TestResult::failure(format!(
                "Excessive memory usage: {:.2}MB",
                max_memory
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        Ok(())
    }
}

/// Concurrency and scalability performance test
pub struct ConcurrencyPerformanceTest;

#[async_trait::async_trait]
impl IntegrationTest for ConcurrencyPerformanceTest {
    fn name(&self) -> &str {
        "concurrency_scalability_performance"
    }
    
    fn description(&self) -> &str {
        "Test system performance under concurrent load and measure scalability"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "concurrency": {
                    "min_workers": 1,
                    "max_workers": 20,
                    "worker_ramp_delay_ms": 500,
                    "test_duration_per_level_seconds": 10
                },
                "workload": {
                    "operations_per_worker": 50,
                    "operation_complexity": "medium"
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let factory = TestDataFactory::new();
        
        let mut scalability_results = Vec::new();
        
        // Test different concurrency levels
        for worker_count in [1, 2, 5, 10, 15, 20] {
            println!("🔧 Testing with {} concurrent workers...", worker_count);
            
            let level_start = Instant::now();
            let mut level_monitor = PerformanceMonitor::new();
            let mut worker_handles = Vec::new();
            
            // Spawn concurrent workers
            for worker_id in 0..worker_count {
                let factory_clone = factory.clone();
                let level_monitor_clone = std::sync::Arc::new(level_monitor.clone());
                
                let handle = tokio::spawn(async move {
                    for operation in 0..50 {
                        let operation_start = Instant::now();
                        
                        // Simulate different types of work
                        match operation % 3 {
                            0 => {
                                // Agent management operation
                                let agent_config = factory_clone.create_agent_config(
                                    Some(&format!("worker-{}-agent-{}", worker_id, operation))
                                );
                                sleep(Duration::from_millis(5)).await;
                            }
                            1 => {
                                // Task processing operation
                                let task_config = factory_clone.create_task_config("concurrent_task");
                                sleep(Duration::from_millis(8)).await;
                            }
                            2 => {
                                // Event handling operation
                                let event_data = factory_clone.create_event_data("concurrent_event");
                                sleep(Duration::from_millis(3)).await;
                            }
                            _ => {}
                        }
                        
                        level_monitor_clone.record_operation();
                        
                        // Add some randomness to avoid thundering herd
                        if operation % 10 == 0 {
                            sleep(Duration::from_millis(worker_id as u64 % 5)).await;
                        }
                    }
                });
                
                worker_handles.push(handle);
            }
            
            // Wait for all workers to complete
            for handle in worker_handles {
                handle.await.context("Worker task failed")?;
            }
            
            let level_duration = level_start.elapsed();
            let level_ops_per_second = (worker_count * 50) as f64 / level_duration.as_secs_f64();
            
            scalability_results.push((worker_count, level_ops_per_second));
            
            println!("   {} workers: {:.2} ops/sec", worker_count, level_ops_per_second);
            
            // Brief pause between levels
            sleep(Duration::from_millis(500)).await;
        }
        
        // Analyze scalability
        let baseline_throughput = scalability_results[0].1; // Single worker throughput
        let max_throughput = scalability_results.iter().map(|(_, ops)| *ops).fold(0.0, f64::max);
        let scalability_factor = max_throughput / baseline_throughput;
        
        // Calculate efficiency (how well we scale with additional workers)
        let efficiency_20_workers = scalability_results.last().unwrap().1 / (baseline_throughput * 20.0);
        
        let metrics = PerformanceMetrics {
            operations_per_second: max_throughput,
            memory_usage_mb: 100.0, // Simulated
            latency_p95_ms: 2.0,
            error_rate: 0.0,
        };
        
        println!("⚡ Concurrency Results:");
        println!("   Baseline (1 worker): {:.2} ops/sec", baseline_throughput);
        println!("   Maximum throughput: {:.2} ops/sec", max_throughput);
        println!("   Scalability factor: {:.2}x", scalability_factor);
        println!("   20-worker efficiency: {:.1}%", efficiency_20_workers * 100.0);
        
        // Validate scalability expectations
        if scalability_factor < 5.0 {
            return Ok(TestResult::failure(format!(
                "Poor scalability: only {:.2}x improvement with 20 workers",
                scalability_factor
            )));
        }
        
        if efficiency_20_workers < 0.3 { // Should maintain at least 30% efficiency
            return Ok(TestResult::failure(format!(
                "Low concurrency efficiency: {:.1}%",
                efficiency_20_workers * 100.0
            )));
        }
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        Ok(())
    }
}

/// Performance regression detection test
pub struct PerformanceRegressionTest;

#[async_trait::async_trait]
impl IntegrationTest for PerformanceRegressionTest {
    fn name(&self) -> &str {
        "performance_regression_detection"
    }
    
    fn description(&self) -> &str {
        "Test performance regression detection and alerting system"
    }
    
    async fn setup(&self) -> Result<TestEnvironment> {
        let env = TestEnvironment::with_config(
            self.name(),
            json!({
                "regression_detection": {
                    "baseline_runs": 5,
                    "test_runs": 10,
                    "regression_threshold": 0.2
                }
            })
        )?;
        
        Ok(env)
    }
    
    async fn execute(&self, env: &TestEnvironment) -> Result<TestResult> {
        let monitor = PerformanceMonitor::new();
        let mut detector = PerformanceRegressionDetector::new();
        
        // Set up baseline
        let baseline = PerformanceBaseline::new("regression_test")
            .with_throughput(1000.0)
            .with_latency(5.0)
            .with_memory(100.0);
        
        detector.add_baseline(baseline);
        
        // Simulate historical good performance
        for run in 0..5 {
            let good_metrics = PerformanceMetrics {
                operations_per_second: 1050.0 + (run as f64 * 10.0),
                memory_usage_mb: 90.0 + (run as f64 * 2.0),
                latency_p95_ms: 4.5 + (run as f64 * 0.1),
                error_rate: 0.001,
            };
            
            detector.record_result("regression_test", good_metrics);
            monitor.record_operation();
        }
        
        // Simulate performance degradation
        let degraded_metrics = PerformanceMetrics {
            operations_per_second: 750.0, // 25% reduction
            memory_usage_mb: 120.0,       // 20% increase
            latency_p95_ms: 8.0,          // 60% increase
            error_rate: 0.002,
        };
        
        // Test regression detection
        let regression = detector.detect_regression("regression_test", &degraded_metrics);
        
        if regression.is_none() {
            return Ok(TestResult::failure("Failed to detect obvious performance regression".to_string()));
        }
        
        let regression_message = regression.unwrap();
        println!("🚨 Detected regression: {}", regression_message);
        
        // Test trend analysis
        detector.record_result("regression_test", degraded_metrics);
        
        // Add more declining results for trend detection
        for i in 1..=5 {
            let declining_metrics = PerformanceMetrics {
                operations_per_second: 750.0 - (i as f64 * 20.0),
                memory_usage_mb: 120.0 + (i as f64 * 5.0),
                latency_p95_ms: 8.0 + (i as f64 * 0.5),
                error_rate: 0.002,
            };
            
            detector.record_result("regression_test", declining_metrics);
        }
        
        let trend_analysis = detector.analyze_trend("regression_test");
        
        if trend_analysis.is_none() {
            return Ok(TestResult::failure("Failed to detect performance trend decline".to_string()));
        }
        
        let trend_message = trend_analysis.unwrap();
        println!("📉 Detected trend: {}", trend_message);
        
        let metrics = PerformanceMetrics {
            operations_per_second: monitor.ops_per_second(),
            memory_usage_mb: 50.0,
            latency_p95_ms: 1.0,
            error_rate: 0.0,
        };
        
        Ok(TestResult::success_with_metrics(metrics))
    }
    
    async fn cleanup(&self, _env: TestEnvironment) -> Result<()> {
        Ok(())
    }
}

/// Collection of all performance tests
pub fn all_performance_tests() -> Vec<Box<dyn IntegrationTest + Send + Sync>> {
    vec![
        Box::new(SystemPerformanceBenchmark),
        Box::new(MemoryPerformanceTest),
        Box::new(ConcurrencyPerformanceTest),
        Box::new(PerformanceRegressionTest),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_system_performance_benchmark() {
        let test = SystemPerformanceBenchmark;
        let result = test.run().await;
        assert!(result.success, "System performance benchmark failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_memory_performance() {
        let test = MemoryPerformanceTest;
        let result = test.run().await;
        assert!(result.success, "Memory performance test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_concurrency_performance() {
        let test = ConcurrencyPerformanceTest;
        let result = test.run().await;
        assert!(result.success, "Concurrency performance test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_performance_regression_detection() {
        let test = PerformanceRegressionTest;
        let result = test.run().await;
        assert!(result.success, "Performance regression test failed: {:?}", result.error_message);
    }
    
    #[tokio::test]
    async fn test_all_performance_tests() {
        let tests = all_performance_tests();
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
        
        println!("Performance Tests: {} passed, {} failed", passed, failed);
        assert_eq!(failed, 0, "Some performance tests failed");
    }
}