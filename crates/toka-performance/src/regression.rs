//! Performance regression detection and analysis
//!
//! This module provides automated performance regression detection capabilities
//! to identify performance degradations across system components.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::benchmarks::{BenchmarkResult, PerformanceBaseline};

/// Regression detector for performance analysis
#[derive(Debug)]
pub struct RegressionDetector {
    /// Component identifier
    component_id: String,
    /// Performance baselines
    baselines: HashMap<String, PerformanceBaseline>,
    /// Historical results for trend analysis
    historical_results: HashMap<String, Vec<BenchmarkResult>>,
    /// Regression detection configuration
    config: RegressionConfig,
}

/// Configuration for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Throughput regression threshold (e.g., 0.8 means 20% degradation)
    pub throughput_threshold: f64,
    /// Latency regression threshold (e.g., 1.2 means 20% increase)
    pub latency_threshold: f64,
    /// Memory usage regression threshold (e.g., 1.3 means 30% increase)
    pub memory_threshold: f64,
    /// Error rate regression threshold (e.g., 2.0 means 2x increase)
    pub error_rate_threshold: f64,
    /// Minimum historical samples for trend analysis
    pub min_samples_for_trend: usize,
    /// Trend analysis window size
    pub trend_window_size: usize,
    /// Trend decline threshold (e.g., 0.1 means 10% decline)
    pub trend_decline_threshold: f64,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            throughput_threshold: 0.8,  // 20% degradation
            latency_threshold: 1.2,     // 20% increase
            memory_threshold: 1.3,      // 30% increase
            error_rate_threshold: 2.0,  // 2x increase
            min_samples_for_trend: 5,
            trend_window_size: 10,
            trend_decline_threshold: 0.1, // 10% decline
        }
    }
}

/// Regression analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Benchmark name
    pub benchmark_name: String,
    /// Regression type detected
    pub regression_type: RegressionType,
    /// Regression severity
    pub severity: RegressionSeverity,
    /// Detailed description
    pub description: String,
    /// Current value
    pub current_value: f64,
    /// Expected/baseline value
    pub expected_value: f64,
    /// Percentage change
    pub percentage_change: f64,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Types of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    /// Throughput degradation
    ThroughputDegradation,
    /// Latency increase
    LatencyIncrease,
    /// Memory usage increase
    MemoryIncrease,
    /// Error rate increase
    ErrorRateIncrease,
    /// Performance trend decline
    TrendDecline,
}

/// Severity levels for regressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Minor regression
    Minor,
    /// Moderate regression
    Moderate,
    /// Major regression
    Major,
    /// Critical regression
    Critical,
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new(component_id: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            baselines: HashMap::new(),
            historical_results: HashMap::new(),
            config: RegressionConfig::default(),
        }
    }
    
    /// Set regression detection configuration
    pub fn set_config(&mut self, config: RegressionConfig) {
        self.config = config;
    }
    
    /// Add a performance baseline
    pub fn add_baseline(&mut self, name: &str, baseline: PerformanceBaseline) {
        self.baselines.insert(name.to_string(), baseline);
    }
    
    /// Record a benchmark result for historical analysis
    pub fn record_result(&mut self, result: BenchmarkResult) {
        let benchmark_name = result.name.clone();
        
        self.historical_results
            .entry(benchmark_name.clone())
            .or_insert_with(Vec::new)
            .push(result);
        
        // Limit historical results to prevent memory growth
        if let Some(results) = self.historical_results.get_mut(&benchmark_name) {
            if results.len() > 100 {
                results.drain(0..results.len() - 100);
            }
        }
    }
    
    /// Detect regression in a benchmark result
    pub fn detect_regression(&self, result: &BenchmarkResult) -> Result<Option<RegressionAnalysis>> {
        if let Some(baseline) = self.baselines.get(&result.name) {
            // Check throughput regression
            if result.ops_per_second < baseline.expected_ops_per_second * self.config.throughput_threshold {
                let percentage_change = ((baseline.expected_ops_per_second - result.ops_per_second) 
                                       / baseline.expected_ops_per_second) * 100.0;
                
                return Ok(Some(RegressionAnalysis {
                    benchmark_name: result.name.clone(),
                    regression_type: RegressionType::ThroughputDegradation,
                    severity: self.calculate_severity(percentage_change),
                    description: format!(
                        "Throughput degradation: {:.2} ops/sec < {:.2} ops/sec (baseline)",
                        result.ops_per_second,
                        baseline.expected_ops_per_second
                    ),
                    current_value: result.ops_per_second,
                    expected_value: baseline.expected_ops_per_second,
                    percentage_change,
                    timestamp: Utc::now(),
                    context: HashMap::new(),
                }));
            }
            
            // Check latency regression
            if result.latency_stats.p95_ms > baseline.max_latency_p95_ms * self.config.latency_threshold {
                let percentage_change = ((result.latency_stats.p95_ms - baseline.max_latency_p95_ms) 
                                       / baseline.max_latency_p95_ms) * 100.0;
                
                return Ok(Some(RegressionAnalysis {
                    benchmark_name: result.name.clone(),
                    regression_type: RegressionType::LatencyIncrease,
                    severity: self.calculate_severity(percentage_change),
                    description: format!(
                        "Latency increase: {:.2}ms > {:.2}ms (baseline)",
                        result.latency_stats.p95_ms,
                        baseline.max_latency_p95_ms
                    ),
                    current_value: result.latency_stats.p95_ms,
                    expected_value: baseline.max_latency_p95_ms,
                    percentage_change,
                    timestamp: Utc::now(),
                    context: HashMap::new(),
                }));
            }
            
            // Check memory regression
            if result.memory_usage_mb > baseline.max_memory_usage_mb * self.config.memory_threshold {
                let percentage_change = ((result.memory_usage_mb - baseline.max_memory_usage_mb) 
                                       / baseline.max_memory_usage_mb) * 100.0;
                
                return Ok(Some(RegressionAnalysis {
                    benchmark_name: result.name.clone(),
                    regression_type: RegressionType::MemoryIncrease,
                    severity: self.calculate_severity(percentage_change),
                    description: format!(
                        "Memory usage increase: {:.2}MB > {:.2}MB (baseline)",
                        result.memory_usage_mb,
                        baseline.max_memory_usage_mb
                    ),
                    current_value: result.memory_usage_mb,
                    expected_value: baseline.max_memory_usage_mb,
                    percentage_change,
                    timestamp: Utc::now(),
                    context: HashMap::new(),
                }));
            }
            
            // Check error rate regression
            if result.error_rate > baseline.max_error_rate * self.config.error_rate_threshold {
                let percentage_change = ((result.error_rate - baseline.max_error_rate) 
                                       / baseline.max_error_rate) * 100.0;
                
                return Ok(Some(RegressionAnalysis {
                    benchmark_name: result.name.clone(),
                    regression_type: RegressionType::ErrorRateIncrease,
                    severity: self.calculate_severity(percentage_change),
                    description: format!(
                        "Error rate increase: {:.4} > {:.4} (baseline)",
                        result.error_rate,
                        baseline.max_error_rate
                    ),
                    current_value: result.error_rate,
                    expected_value: baseline.max_error_rate,
                    percentage_change,
                    timestamp: Utc::now(),
                    context: HashMap::new(),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Analyze performance trends
    pub fn analyze_trend(&self, benchmark_name: &str) -> Result<Option<RegressionAnalysis>> {
        let results = self.historical_results.get(benchmark_name)
            .ok_or_else(|| anyhow::anyhow!("No historical results for benchmark '{}'", benchmark_name))?;
        
        if results.len() < self.config.min_samples_for_trend {
            return Ok(None);
        }
        
        let recent_results = if results.len() > self.config.trend_window_size {
            &results[results.len() - self.config.trend_window_size..]
        } else {
            results
        };
        
        // Analyze throughput trend
        let throughput_values: Vec<f64> = recent_results.iter()
            .map(|r| r.ops_per_second)
            .collect();
        
        if self.is_declining_trend(&throughput_values) {
            let first_value = throughput_values.first().unwrap();
            let last_value = throughput_values.last().unwrap();
            let percentage_decline = ((first_value - last_value) / first_value) * 100.0;
            
            if percentage_decline > self.config.trend_decline_threshold * 100.0 {
                return Ok(Some(RegressionAnalysis {
                    benchmark_name: benchmark_name.to_string(),
                    regression_type: RegressionType::TrendDecline,
                    severity: self.calculate_severity(percentage_decline),
                    description: format!(
                        "Performance trend decline: {:.1}% reduction over {} measurements",
                        percentage_decline,
                        recent_results.len()
                    ),
                    current_value: *last_value,
                    expected_value: *first_value,
                    percentage_change: percentage_decline,
                    timestamp: Utc::now(),
                    context: HashMap::new(),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Check if values show a declining trend
    fn is_declining_trend(&self, values: &[f64]) -> bool {
        if values.len() < 2 {
            return false;
        }
        
        // Check if the majority of consecutive pairs show decline
        let declining_pairs = values.windows(2)
            .filter(|w| w[1] < w[0])
            .count();
        
        let total_pairs = values.len() - 1;
        
        // Consider it a declining trend if more than 70% of pairs show decline
        declining_pairs as f64 / total_pairs as f64 > 0.7
    }
    
    /// Calculate regression severity based on percentage change
    fn calculate_severity(&self, percentage_change: f64) -> RegressionSeverity {
        if percentage_change > 50.0 {
            RegressionSeverity::Critical
        } else if percentage_change > 30.0 {
            RegressionSeverity::Major
        } else if percentage_change > 15.0 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Minor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmarks::LatencyStats;
    
    #[test]
    fn test_regression_detector_creation() {
        let detector = RegressionDetector::new("test-component");
        assert_eq!(detector.component_id, "test-component");
    }
    
    #[test]
    fn test_throughput_regression_detection() {
        let mut detector = RegressionDetector::new("test-component");
        
        let baseline = PerformanceBaseline {
            name: "test_benchmark".to_string(),
            expected_ops_per_second: 1000.0,
            max_latency_p95_ms: 10.0,
            max_memory_usage_mb: 100.0,
            max_error_rate: 0.01,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        detector.add_baseline("test_benchmark", baseline);
        
        let result = BenchmarkResult {
            name: "test_benchmark".to_string(),
            ops_per_second: 700.0, // 30% degradation
            latency_stats: LatencyStats {
                avg_ms: 5.0,
                p50_ms: 5.0,
                p95_ms: 8.0,
                p99_ms: 10.0,
                max_ms: 15.0,
                min_ms: 1.0,
            },
            memory_usage_mb: 90.0,
            error_rate: 0.005,
            execution_duration: std::time::Duration::from_secs(30),
            timestamp: Utc::now(),
            additional_metrics: HashMap::new(),
        };
        
        let regression = detector.detect_regression(&result).unwrap();
        assert!(regression.is_some());
        
        let regression = regression.unwrap();
        assert!(matches!(regression.regression_type, RegressionType::ThroughputDegradation));
        // 30% degradation should be classified as Moderate (>15% but <=30%)
        assert!(matches!(regression.severity, RegressionSeverity::Moderate));
    }
    
    #[test]
    fn test_trend_analysis() {
        let mut detector = RegressionDetector::new("test-component");
        
        // Add declining results
        for i in 0..10 {
            let result = BenchmarkResult {
                name: "test_benchmark".to_string(),
                ops_per_second: 1000.0 - (i as f64 * 50.0), // Declining throughput
                latency_stats: LatencyStats {
                    avg_ms: 5.0,
                    p50_ms: 5.0,
                    p95_ms: 8.0,
                    p99_ms: 10.0,
                    max_ms: 15.0,
                    min_ms: 1.0,
                },
                memory_usage_mb: 90.0,
                error_rate: 0.005,
                execution_duration: std::time::Duration::from_secs(30),
                timestamp: Utc::now(),
                additional_metrics: HashMap::new(),
            };
            
            detector.record_result(result);
        }
        
        let trend_analysis = detector.analyze_trend("test_benchmark").unwrap();
        assert!(trend_analysis.is_some());
        
        let trend_analysis = trend_analysis.unwrap();
        assert!(matches!(trend_analysis.regression_type, RegressionType::TrendDecline));
    }
}