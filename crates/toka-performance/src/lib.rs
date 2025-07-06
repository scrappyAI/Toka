//! # Toka Performance & Observability Foundation
//!
//! This crate provides comprehensive performance monitoring, observability, and analysis
//! capabilities for Toka OS. It includes metrics collection, distributed tracing,
//! performance benchmarking, and real-time monitoring dashboards.
//!
//! ## Key Features
//!
//! - **Metrics Collection**: Standardized instrumentation across all system components
//! - **Distributed Tracing**: End-to-end operation visibility with OpenTelemetry
//! - **Performance Benchmarking**: Automated performance regression detection
//! - **Real-time Monitoring**: Dashboard and alerting for performance anomalies
//! - **Capacity Planning**: Resource utilization analysis and forecasting
//! - **Profiling Integration**: CPU and memory profiling capabilities
//!
//! ## Architecture
//!
//! The performance foundation is organized into several key modules:
//!
//! - [`metrics`]: Metrics collection and export infrastructure
//! - [`tracing`]: Distributed tracing and span management
//! - [`benchmarks`]: Performance benchmarking and regression detection
//! - [`monitoring`]: Real-time monitoring and alerting
//! - [`profiling`]: CPU and memory profiling utilities
//! - [`dashboard`]: Performance dashboard and visualization
//!
//! ## Usage
//!
//! See the tests for comprehensive usage examples. The main entry point is
//! `PerformanceManager` which coordinates all performance monitoring capabilities.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod metrics;
pub mod tracing;
pub mod benchmarks;
pub mod monitoring;
pub mod profiling;
pub mod dashboard;
pub mod regression;
pub mod capacity;

// Re-export commonly used types
pub use metrics::{MetricsCollector, MetricsRegistry, MetricType};
pub use tracing::{TracingManager, TraceSpan, TraceContext};
pub use benchmarks::{BenchmarkSuite, BenchmarkResult, PerformanceBaseline};
pub use monitoring::{PerformanceMonitor, MonitoringConfig, AlertRule};
pub use regression::{RegressionDetector, RegressionAnalysis};
pub use dashboard::{Dashboard, DashboardConfig, MetricVisualization};

/// Main performance and observability manager
///
/// This is the central coordinator for all performance monitoring and observability
/// features in Toka OS. It manages metrics collection, distributed tracing,
/// performance benchmarking, and real-time monitoring.
pub struct PerformanceManager {
    /// Component identifier
    component_id: String,
    /// Metrics collection registry
    metrics: Arc<MetricsRegistry>,
    /// Distributed tracing manager
    tracing: Arc<TracingManager>,
    /// Performance monitoring instance
    monitor: Arc<PerformanceMonitor>,
    /// Benchmark suite for regression detection
    benchmarks: Arc<RwLock<BenchmarkSuite>>,
    /// Regression detection system
    regression_detector: Arc<RegressionDetector>,
    /// Performance dashboard
    dashboard: Arc<Dashboard>,
    /// Configuration
    config: PerformanceConfig,
}

impl std::fmt::Debug for PerformanceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceManager")
            .field("component_id", &self.component_id)
            .field("config", &self.config)
            .finish()
    }
}

/// Configuration for performance monitoring and observability
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
    /// Enable benchmarking
    pub benchmarking_enabled: bool,
    /// Enable regression detection
    pub regression_detection_enabled: bool,
    /// Enable performance dashboard
    pub dashboard_enabled: bool,
    /// Metrics collection interval
    pub metrics_interval_seconds: u64,
    /// Tracing sampling rate (0.0 to 1.0)
    pub tracing_sample_rate: f64,
    /// Performance monitoring configuration
    pub monitoring: monitoring::MonitoringConfig,
    /// Dashboard configuration
    pub dashboard: dashboard::DashboardConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            monitoring_enabled: true,
            benchmarking_enabled: true,
            regression_detection_enabled: true,
            dashboard_enabled: false, // Disabled by default for production
            metrics_interval_seconds: 10,
            tracing_sample_rate: 0.1, // 10% sampling rate
            monitoring: monitoring::MonitoringConfig::default(),
            dashboard: dashboard::DashboardConfig::default(),
        }
    }
}

impl PerformanceManager {
    /// Create a new performance manager for the specified component
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component being monitored
    /// * `config` - Performance monitoring configuration
    ///
    /// # Returns
    ///
    /// A new performance manager instance ready for use
    pub async fn new(component_id: impl Into<String>, config: PerformanceConfig) -> Result<Self> {
        let component_id = component_id.into();
        
        // Initialize metrics collection
        let metrics = Arc::new(MetricsRegistry::new(&component_id)?);
        
        // Initialize distributed tracing
        let tracing = Arc::new(TracingManager::new(&component_id, config.tracing_sample_rate).await?);
        
        // Initialize performance monitoring
        let monitor = Arc::new(PerformanceMonitor::new(&component_id, config.monitoring.clone()).await?);
        
        // Initialize benchmark suite
        let benchmarks = Arc::new(RwLock::new(BenchmarkSuite::new(&component_id)));
        
        // Initialize regression detector
        let regression_detector = Arc::new(RegressionDetector::new(&component_id));
        
        // Initialize dashboard
        let dashboard = Arc::new(Dashboard::new(&component_id, config.dashboard.clone()).await?);
        
        Ok(Self {
            component_id,
            metrics,
            tracing,
            monitor,
            benchmarks,
            regression_detector,
            dashboard,
            config,
        })
    }
    
    /// Get the component identifier
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
    
    /// Get the metrics registry
    pub fn metrics(&self) -> &MetricsRegistry {
        &self.metrics
    }
    
    /// Get the tracing manager
    pub fn tracing(&self) -> &TracingManager {
        &self.tracing
    }
    
    /// Get the performance monitor
    pub fn monitor(&self) -> &PerformanceMonitor {
        &self.monitor
    }
    
    /// Get the benchmark suite
    pub async fn benchmarks(&self) -> tokio::sync::RwLockReadGuard<'_, BenchmarkSuite> {
        self.benchmarks.read().await
    }
    
    /// Get the regression detector
    pub fn regression_detector(&self) -> &RegressionDetector {
        &self.regression_detector
    }
    
    /// Get the performance dashboard
    pub fn dashboard(&self) -> &Dashboard {
        &self.dashboard
    }
    
    /// Start performance monitoring for this component
    pub async fn start(&self) -> Result<()> {
        ::tracing::info!(
            component = %self.component_id,
            "Starting performance monitoring"
        );
        
        // Start metrics collection
        if self.config.metrics_enabled {
            self.metrics.start_collection(std::time::Duration::from_secs(self.config.metrics_interval_seconds)).await?;
        }
        
        // Start distributed tracing
        if self.config.tracing_enabled {
            self.tracing.start().await?;
        }
        
        // Start performance monitoring
        if self.config.monitoring_enabled {
            self.monitor.start().await?;
        }
        
        // Start dashboard if enabled
        if self.config.dashboard_enabled {
            self.dashboard.start().await?;
        }
        
        ::tracing::info!(
            component = %self.component_id,
            "Performance monitoring started successfully"
        );
        
        Ok(())
    }
    
    /// Stop performance monitoring
    pub async fn stop(&self) -> Result<()> {
        ::tracing::info!(
            component = %self.component_id,
            "Stopping performance monitoring"
        );
        
        // Stop dashboard
        if self.config.dashboard_enabled {
            self.dashboard.stop().await?;
        }
        
        // Stop performance monitoring
        if self.config.monitoring_enabled {
            self.monitor.stop().await?;
        }
        
        // Stop distributed tracing
        if self.config.tracing_enabled {
            self.tracing.stop().await?;
        }
        
        // Stop metrics collection
        if self.config.metrics_enabled {
            self.metrics.stop_collection().await?;
        }
        
        ::tracing::info!(
            component = %self.component_id,
            "Performance monitoring stopped"
        );
        
        Ok(())
    }
    
    /// Run performance benchmarks
    pub async fn run_benchmarks(&self) -> Result<Vec<BenchmarkResult>> {
        if !self.config.benchmarking_enabled {
            return Ok(Vec::new());
        }
        
        let benchmarks = self.benchmarks.read().await;
        let results = benchmarks.run_all().await?;
        
        // Check for regressions
        if self.config.regression_detection_enabled {
            for result in &results {
                if let Some(regression) = self.regression_detector.detect_regression(result)? {
                    ::tracing::warn!(
                        component = %self.component_id,
                        benchmark = %result.name,
                        regression = %regression.description,
                        "Performance regression detected"
                    );
                    
                    // Trigger alert
                    self.monitor.trigger_alert(
                        &format!("Performance regression in {}", result.name),
                        &regression.description,
                        monitoring::AlertSeverity::Warning,
                    ).await?;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> Result<serde_json::Value> {
        let metrics = self.metrics.export_metrics().await?;
        Ok(metrics)
    }
    
    /// Get performance health status
    pub async fn health_status(&self) -> Result<HealthStatus> {
        let metrics_health = self.metrics.health().await?;
        let tracing_health = self.tracing.health().await?;
        let monitor_health = self.monitor.health().await?;
        
        let overall_health = if metrics_health.is_healthy() && tracing_health.is_healthy() && monitor_health.is_healthy() {
            ComponentHealth::Healthy
        } else {
            ComponentHealth::Degraded
        };
        
        Ok(HealthStatus {
            overall: overall_health,
            metrics: metrics_health,
            tracing: tracing_health,
            monitoring: monitor_health,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Health status for performance monitoring components
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub overall: ComponentHealth,
    /// Metrics collection health
    pub metrics: ComponentHealth,
    /// Distributed tracing health
    pub tracing: ComponentHealth,
    /// Performance monitoring health
    pub monitoring: ComponentHealth,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Health status for individual components
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComponentHealth {
    /// Component is operating normally
    Healthy,
    /// Component is operational but degraded
    Degraded,
    /// Component is not operational
    Unhealthy,
}

impl ComponentHealth {
    /// Check if the component is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ComponentHealth::Healthy)
    }
    
    /// Check if the component is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, ComponentHealth::Healthy | ComponentHealth::Degraded)
    }
}

/// Error types for performance monitoring
#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    /// Metrics collection error
    #[error("Metrics collection error: {0}")]
    Metrics(#[from] metrics::MetricsError),
    
    /// Tracing error
    #[error("Tracing error: {0}")]
    Tracing(#[from] tracing::TracingError),
    
    /// Monitoring error
    #[error("Monitoring error: {0}")]
    Monitoring(#[from] monitoring::MonitoringError),
    
    /// Benchmarking error
    #[error("Benchmarking error: {0}")]
    Benchmarking(#[from] benchmarks::BenchmarkError),
    
    /// Dashboard error
    #[error("Dashboard error: {0}")]
    Dashboard(#[from] dashboard::DashboardError),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for performance operations
pub type PerformanceResult<T> = Result<T, PerformanceError>;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_performance_manager_creation() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new("test-component", config).await;
        
        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert_eq!(manager.component_id(), "test-component");
    }
    
    #[tokio::test]
    async fn test_performance_manager_lifecycle() {
        let config = PerformanceConfig {
            dashboard_enabled: false, // Disable dashboard for testing
            ..Default::default()
        };
        
        let manager = PerformanceManager::new("test-lifecycle", config).await.unwrap();
        
        // Start monitoring
        assert!(manager.start().await.is_ok());
        
        // Check health
        let health = manager.health_status().await.unwrap();
        assert!(health.overall.is_operational());
        
        // Stop monitoring
        assert!(manager.stop().await.is_ok());
    }
}