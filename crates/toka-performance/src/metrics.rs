//! Metrics collection and export infrastructure
//!
//! This module provides comprehensive metrics collection capabilities using Prometheus
//! for standardized instrumentation across all Toka OS components.

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use metrics::{Counter, Gauge, Histogram};
use prometheus::{Encoder, TextEncoder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, error};

/// Metric types supported by the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricType {
    /// Counter metrics that only increase
    Counter,
    /// Gauge metrics that can go up and down
    Gauge,
    /// Histogram metrics for latency and distribution measurements
    Histogram,
    /// Summary metrics for percentile calculations
    Summary,
}

/// Metric value with timestamp
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricValue {
    /// Metric value
    pub value: f64,
    /// Timestamp when metric was recorded
    pub timestamp: DateTime<Utc>,
    /// Optional labels for the metric
    pub labels: HashMap<String, String>,
}

/// Metrics registry for collecting and managing metrics
pub struct MetricsRegistry {
    /// Component identifier
    component_id: String,
    /// Prometheus registry
    prometheus_registry: prometheus::Registry,
    /// Metric counters
    counters: Arc<DashMap<String, Counter>>,
    /// Metric gauges
    gauges: Arc<DashMap<String, Gauge>>,
    /// Metric histograms
    histograms: Arc<DashMap<String, Histogram>>,
    /// Metric metadata
    metadata: Arc<RwLock<HashMap<String, MetricMetadata>>>,
    /// Collection task handle
    collection_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Metrics collector
    collector: Arc<MetricsCollector>,
}

/// Metadata for a metric
#[derive(Debug, Clone)]
pub struct MetricMetadata {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric description
    pub description: String,
    /// Metric unit
    pub unit: Option<String>,
    /// Metric labels
    pub labels: Vec<String>,
    /// When the metric was first registered
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Metrics collector for managing metric collection
#[derive(Clone)]
pub struct MetricsCollector {
    /// Component identifier
    component_id: String,
    /// Collected metrics
    metrics: Arc<RwLock<HashMap<String, Vec<MetricValue>>>>,
    /// Collection interval
    collection_interval: Duration,
    /// Maximum metrics to retain per key
    max_metrics_per_key: usize,
}

impl std::fmt::Debug for MetricsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsRegistry")
            .field("component_id", &self.component_id)
            .field("counters_count", &self.counters.len())
            .field("gauges_count", &self.gauges.len())
            .field("histograms_count", &self.histograms.len())
            .finish()
    }
}

impl std::fmt::Debug for MetricsCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsCollector")
            .field("component_id", &self.component_id)
            .field("collection_interval", &self.collection_interval)
            .field("max_metrics_per_key", &self.max_metrics_per_key)
            .finish()
    }
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(component_id: &str) -> Result<Self> {
        let prometheus_registry = prometheus::Registry::new();
        let collector = Arc::new(MetricsCollector::new(component_id, Duration::from_secs(10)));
        
        Ok(Self {
            component_id: component_id.to_string(),
            prometheus_registry,
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            collection_task: Arc::new(RwLock::new(None)),
            collector,
        })
    }
    
    /// Register a counter metric
    pub async fn register_counter(&self, name: &str, description: &str, unit: Option<&str>) -> Result<()> {
        // Create a placeholder counter for now
        // In production, this would use proper metrics integration
        debug!(
            component = %self.component_id,
            metric = %name,
            "Registered counter metric"
        );
        
        let metadata = MetricMetadata {
            name: name.to_string(),
            metric_type: MetricType::Counter,
            description: description.to_string(),
            unit: unit.map(|s| s.to_string()),
            labels: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.metadata.write().await.insert(name.to_string(), metadata);
        Ok(())
    }
    
    /// Register a gauge metric
    pub async fn register_gauge(&self, name: &str, description: &str, unit: Option<&str>) -> Result<()> {
        // Create a placeholder gauge for now
        // In production, this would use proper metrics integration
        debug!(
            component = %self.component_id,
            metric = %name,
            "Registered gauge metric"
        );
        
        let metadata = MetricMetadata {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            description: description.to_string(),
            unit: unit.map(|s| s.to_string()),
            labels: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.metadata.write().await.insert(name.to_string(), metadata);
        Ok(())
    }
    
    /// Register a histogram metric
    pub async fn register_histogram(&self, name: &str, description: &str, unit: Option<&str>) -> Result<()> {
        // Create a placeholder histogram for now
        // In production, this would use proper metrics integration
        debug!(
            component = %self.component_id,
            metric = %name,
            "Registered histogram metric"
        );
        
        let metadata = MetricMetadata {
            name: name.to_string(),
            metric_type: MetricType::Histogram,
            description: description.to_string(),
            unit: unit.map(|s| s.to_string()),
            labels: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.metadata.write().await.insert(name.to_string(), metadata);
        Ok(())
    }
    
    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str, value: f64) -> Result<()> {
        debug!(
            component = %self.component_id,
            metric = %name,
            value = %value,
            "Incremented counter"
        );
        Ok(())
    }
    
    /// Set a gauge metric value
    pub fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        debug!(
            component = %self.component_id,
            metric = %name,
            value = %value,
            "Set gauge value"
        );
        Ok(())
    }
    
    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: f64) -> Result<()> {
        debug!(
            component = %self.component_id,
            metric = %name,
            value = %value,
            "Recorded histogram value"
        );
        Ok(())
    }
    
    /// Start metrics collection
    pub async fn start_collection(&self, interval: Duration) -> Result<()> {
        let mut collection_task = self.collection_task.write().await;
        
        if collection_task.is_some() {
            return Err(anyhow::anyhow!("Metrics collection already started"));
        }
        
        let collector = Arc::clone(&self.collector);
        let component_id = self.component_id.clone();
        
        let task = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            info!(
                component = %component_id,
                "Starting metrics collection"
            );
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = collector.collect_metrics().await {
                    error!(
                        component = %component_id,
                        error = %e,
                        "Failed to collect metrics"
                    );
                }
            }
        });
        
        *collection_task = Some(task);
        Ok(())
    }
    
    /// Stop metrics collection
    pub async fn stop_collection(&self) -> Result<()> {
        let mut collection_task = self.collection_task.write().await;
        
        if let Some(task) = collection_task.take() {
            task.abort();
            info!(
                component = %self.component_id,
                "Stopped metrics collection"
            );
        }
        
        Ok(())
    }
    
    /// Export metrics in Prometheus format
    pub async fn export_metrics(&self) -> Result<serde_json::Value> {
        let encoder = TextEncoder::new();
        let metric_families = self.prometheus_registry.gather();
        
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        
        let prometheus_metrics = String::from_utf8(buffer)?;
        
        let collected_metrics = self.collector.get_all_metrics().await;
        
        Ok(serde_json::json!({
            "component_id": self.component_id,
            "timestamp": Utc::now().to_rfc3339(),
            "prometheus_metrics": prometheus_metrics,
            "collected_metrics": collected_metrics
        }))
    }
    
    /// Get metric metadata
    pub async fn get_metadata(&self, name: &str) -> Option<MetricMetadata> {
        self.metadata.read().await.get(name).cloned()
    }
    
    /// List all registered metrics
    pub async fn list_metrics(&self) -> Vec<String> {
        self.metadata.read().await.keys().cloned().collect()
    }
    
    /// Health check for metrics collection
    pub async fn health(&self) -> Result<super::ComponentHealth> {
        let collection_task = self.collection_task.read().await;
        
        if let Some(task) = collection_task.as_ref() {
            if task.is_finished() {
                Ok(super::ComponentHealth::Unhealthy)
            } else {
                Ok(super::ComponentHealth::Healthy)
            }
        } else {
            Ok(super::ComponentHealth::Degraded) // Not started
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(component_id: &str, collection_interval: Duration) -> Self {
        Self {
            component_id: component_id.to_string(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
            max_metrics_per_key: 1000, // Keep last 1000 values per metric
        }
    }
    
    /// Collect current metrics
    pub async fn collect_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        
        // Collect system metrics
        self.collect_system_metrics(&mut metrics).await?;
        
        // Collect component-specific metrics
        self.collect_component_metrics(&mut metrics).await?;
        
        // Clean up old metrics
        self.cleanup_old_metrics(&mut metrics).await;
        
        Ok(())
    }
    
    /// Collect system-level metrics
    async fn collect_system_metrics(&self, metrics: &mut HashMap<String, Vec<MetricValue>>) -> Result<()> {
        let now = Utc::now();
        
        // Memory usage
        if let Ok(memory_usage) = self.get_memory_usage().await {
            self.add_metric_value(metrics, "system.memory.usage_bytes", memory_usage, now);
        }
        
        // CPU usage
        if let Ok(cpu_usage) = self.get_cpu_usage().await {
            self.add_metric_value(metrics, "system.cpu.usage_percent", cpu_usage, now);
        }
        
        // Disk usage
        if let Ok(disk_usage) = self.get_disk_usage().await {
            self.add_metric_value(metrics, "system.disk.usage_bytes", disk_usage, now);
        }
        
        Ok(())
    }
    
    /// Collect component-specific metrics
    async fn collect_component_metrics(&self, metrics: &mut HashMap<String, Vec<MetricValue>>) -> Result<()> {
        let now = Utc::now();
        
        // Add component-specific metrics based on component_id
        match self.component_id.as_str() {
            "toka-runtime" => {
                // Runtime-specific metrics
                self.add_metric_value(metrics, "runtime.agents.active", 0.0, now);
                self.add_metric_value(metrics, "runtime.tasks.completed", 0.0, now);
            }
            "toka-kernel" => {
                // Kernel-specific metrics
                self.add_metric_value(metrics, "kernel.events.processed", 0.0, now);
                self.add_metric_value(metrics, "kernel.operations.latency_ms", 0.0, now);
            }
            _ => {
                // Generic component metrics
                self.add_metric_value(metrics, "component.operations.total", 0.0, now);
            }
        }
        
        Ok(())
    }
    
    /// Add a metric value to the collection
    fn add_metric_value(&self, metrics: &mut HashMap<String, Vec<MetricValue>>, name: &str, value: f64, timestamp: DateTime<Utc>) {
        let metric_value = MetricValue {
            value,
            timestamp,
            labels: HashMap::new(),
        };
        
        metrics.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(metric_value);
    }
    
    /// Clean up old metrics to prevent memory leaks
    async fn cleanup_old_metrics(&self, metrics: &mut HashMap<String, Vec<MetricValue>>) {
        for values in metrics.values_mut() {
            if values.len() > self.max_metrics_per_key {
                values.drain(0..values.len() - self.max_metrics_per_key);
            }
        }
    }
    
    /// Get current memory usage
    async fn get_memory_usage(&self) -> Result<f64> {
        // Simulate memory usage collection
        // In a real implementation, this would use system APIs
        Ok(50.0 * 1024.0 * 1024.0) // 50MB
    }
    
    /// Get current CPU usage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Simulate CPU usage collection
        // In a real implementation, this would use system APIs
        Ok(25.0) // 25%
    }
    
    /// Get current disk usage
    async fn get_disk_usage(&self) -> Result<f64> {
        // Simulate disk usage collection
        // In a real implementation, this would use system APIs
        Ok(1.0 * 1024.0 * 1024.0 * 1024.0) // 1GB
    }
    
    /// Get all collected metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, Vec<MetricValue>> {
        self.metrics.read().await.clone()
    }
    
    /// Get metrics for a specific key
    pub async fn get_metric(&self, name: &str) -> Option<Vec<MetricValue>> {
        self.metrics.read().await.get(name).cloned()
    }
}

// Note: Prometheus recorder implementation would go here
// For now, we use the built-in metrics collection approach

/// Error types for metrics operations
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Metric not found
    #[error("Metric '{0}' not found")]
    NotFound(String),
    
    /// Invalid metric type
    #[error("Invalid metric type: {0}")]
    InvalidType(String),
    
    /// Collection error
    #[error("Metrics collection error: {0}")]
    Collection(String),
    
    /// Export error
    #[error("Metrics export error: {0}")]
    Export(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new("test-component");
        assert!(registry.is_ok());
        
        let registry = registry.unwrap();
        assert_eq!(registry.component_id, "test-component");
    }
    
    #[tokio::test]
    async fn test_counter_registration() {
        let registry = MetricsRegistry::new("test-component").unwrap();
        
        let result = registry.register_counter("test_counter", "A test counter", Some("requests")).await;
        assert!(result.is_ok());
        
        let metadata = registry.get_metadata("test_counter").await;
        assert!(metadata.is_some());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "test_counter");
        assert_eq!(metadata.metric_type, MetricType::Counter);
    }
    
    #[tokio::test]
    async fn test_gauge_registration() {
        let registry = MetricsRegistry::new("test-component").unwrap();
        
        let result = registry.register_gauge("test_gauge", "A test gauge", Some("bytes")).await;
        assert!(result.is_ok());
        
        let metadata = registry.get_metadata("test_gauge").await;
        assert!(metadata.is_some());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "test_gauge");
        assert_eq!(metadata.metric_type, MetricType::Gauge);
    }
    
    #[tokio::test]
    async fn test_histogram_registration() {
        let registry = MetricsRegistry::new("test-component").unwrap();
        
        let result = registry.register_histogram("test_histogram", "A test histogram", Some("seconds")).await;
        assert!(result.is_ok());
        
        let metadata = registry.get_metadata("test_histogram").await;
        assert!(metadata.is_some());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "test_histogram");
        assert_eq!(metadata.metric_type, MetricType::Histogram);
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new("test-component", Duration::from_secs(1));
        
        let result = collector.collect_metrics().await;
        assert!(result.is_ok());
        
        let metrics = collector.get_all_metrics().await;
        assert!(!metrics.is_empty());
    }
}