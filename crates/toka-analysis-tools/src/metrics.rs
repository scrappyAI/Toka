//! Metrics collection for analysis tools

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::security::ResourceUsage;

/// Analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    /// Tool name
    pub tool_name: String,
    /// Success status
    pub success: bool,
    /// Execution time
    pub execution_time: Duration,
    /// Security checks passed
    pub security_checks_passed: bool,
    /// Input validation passed
    pub input_validation_passed: bool,
    /// Output validation passed
    pub output_validation_passed: bool,
    /// Resource usage
    pub resource_usage: Option<ResourceUsage>,
    /// Output size in bytes
    pub output_size: usize,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl AnalysisMetrics {
    /// Create new metrics
    pub fn new(tool_name: String) -> Self {
        Self {
            tool_name,
            success: false,
            execution_time: Duration::from_secs(0),
            security_checks_passed: false,
            input_validation_passed: false,
            output_validation_passed: false,
            resource_usage: None,
            output_size: 0,
            error_message: None,
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    enabled: bool,
    metrics: Arc<RwLock<Vec<AnalysisMetrics>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Record execution metrics
    pub async fn record_execution(&self, metrics: &AnalysisMetrics) {
        if self.enabled {
            let mut metrics_store = self.metrics.write().await;
            metrics_store.push(metrics.clone());
        }
    }
    
    /// Get all metrics
    pub async fn get_metrics(&self) -> Vec<AnalysisMetrics> {
        self.metrics.read().await.clone()
    }
    
    /// Clear metrics
    pub async fn clear_metrics(&self) {
        self.metrics.write().await.clear();
    }
}