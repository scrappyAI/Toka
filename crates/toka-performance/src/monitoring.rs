//! Real-time monitoring and alerting
//!
//! This module provides real-time performance monitoring and alerting capabilities
//! for detecting performance anomalies and system issues.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Monitoring interval
    pub interval_seconds: u64,
    /// Alert thresholds
    pub thresholds: HashMap<String, f64>,
    /// Alert rules
    pub alert_rules: Vec<AlertRule>,
    /// History retention
    pub history_retention_minutes: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 10,
            thresholds: HashMap::new(),
            alert_rules: Vec::new(),
            history_retention_minutes: 60,
        }
    }
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Metric name to monitor
    pub metric_name: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert description
    pub description: String,
    /// Cooldown period
    pub cooldown_minutes: u64,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Greater than threshold
    GreaterThan(f64),
    /// Less than threshold
    LessThan(f64),
    /// Outside range
    OutsideRange(f64, f64),
    /// Rate of change
    RateOfChange(f64),
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
    /// Warning
    Warning,
}

/// Performance monitor
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Component identifier
    component_id: String,
    /// Configuration
    config: MonitoringConfig,
    /// Monitoring metrics
    metrics: Arc<RwLock<HashMap<String, VecDeque<MetricPoint>>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,
    /// Monitoring task
    monitoring_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert rule name
    pub rule_name: String,
    /// Alert message
    pub message: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Alert status
    pub status: AlertStatus,
    /// Metric value that triggered the alert
    pub trigger_value: f64,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Active alert
    Active,
    /// Resolved alert
    Resolved,
    /// Acknowledged alert
    Acknowledged,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub async fn new(component_id: &str, config: MonitoringConfig) -> Result<Self> {
        Ok(Self {
            component_id: component_id.to_string(),
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            monitoring_task: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let component_id = self.component_id.clone();
        let metrics = Arc::clone(&self.metrics);
        let active_alerts = Arc::clone(&self.active_alerts);
        let alert_history = Arc::clone(&self.alert_history);
        let config = self.config.clone();
        
        let task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.interval_seconds));
            
            tracing::info!(
                component = %component_id,
                "Starting performance monitoring"
            );
            
            loop {
                interval.tick().await;
                
                // Collect metrics
                if let Err(e) = Self::collect_performance_metrics(&metrics, &component_id).await {
                    tracing::error!(
                        component = %component_id,
                        error = %e,
                        "Failed to collect performance metrics"
                    );
                }
                
                // Check alert rules
                if let Err(e) = Self::check_alert_rules(&metrics, &active_alerts, &alert_history, &config).await {
                    tracing::error!(
                        component = %component_id,
                        error = %e,
                        "Failed to check alert rules"
                    );
                }
                
                // Clean up old metrics
                if let Err(e) = Self::cleanup_old_metrics(&metrics, &config).await {
                    tracing::error!(
                        component = %component_id,
                        error = %e,
                        "Failed to cleanup old metrics"
                    );
                }
            }
        });
        
        *self.monitoring_task.write().await = Some(task);
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<()> {
        let mut task = self.monitoring_task.write().await;
        if let Some(task) = task.take() {
            task.abort();
        }
        Ok(())
    }
    
    /// Record a metric value
    pub async fn record_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        
        let metric_point = MetricPoint {
            timestamp: Utc::now(),
            value,
            labels,
        };
        
        metrics.entry(name.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(metric_point);
        
        Ok(())
    }
    
    /// Trigger an alert
    pub async fn trigger_alert(&self, message: &str, description: &str, severity: AlertSeverity) -> Result<()> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            rule_name: "manual".to_string(),
            message: message.to_string(),
            description: description.to_string(),
            severity,
            timestamp: Utc::now(),
            status: AlertStatus::Active,
            trigger_value: 0.0,
        };
        
        // Add to active alerts
        self.active_alerts.write().await.insert(alert.id.clone(), alert.clone());
        
        // Add to history
        self.alert_history.write().await.push(alert.clone());
        
        tracing::warn!(
            component = %self.component_id,
            alert_id = %alert.id,
            message = %message,
            "Alert triggered"
        );
        
        Ok(())
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> HashMap<String, VecDeque<MetricPoint>> {
        self.metrics.read().await.clone()
    }
    
    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().await.values().cloned().collect()
    }
    
    /// Health check
    pub async fn health(&self) -> Result<super::ComponentHealth> {
        let task = self.monitoring_task.read().await;
        
        if let Some(task) = task.as_ref() {
            if task.is_finished() {
                Ok(super::ComponentHealth::Unhealthy)
            } else {
                Ok(super::ComponentHealth::Healthy)
            }
        } else {
            Ok(super::ComponentHealth::Degraded)
        }
    }
    
    /// Collect performance metrics
    async fn collect_performance_metrics(
        metrics: &Arc<RwLock<HashMap<String, VecDeque<MetricPoint>>>>,
        _component_id: &str,
    ) -> Result<()> {
        let mut metrics_guard = metrics.write().await;
        let now = Utc::now();
        
        // Collect CPU usage
        let cpu_usage = Self::get_cpu_usage().await?;
        let cpu_point = MetricPoint {
            timestamp: now,
            value: cpu_usage,
            labels: HashMap::new(),
        };
        metrics_guard.entry("cpu_usage_percent".to_string())
            .or_insert_with(VecDeque::new)
            .push_back(cpu_point);
        
        // Collect memory usage
        let memory_usage = Self::get_memory_usage().await?;
        let memory_point = MetricPoint {
            timestamp: now,
            value: memory_usage,
            labels: HashMap::new(),
        };
        metrics_guard.entry("memory_usage_bytes".to_string())
            .or_insert_with(VecDeque::new)
            .push_back(memory_point);
        
        Ok(())
    }
    
    /// Check alert rules
    async fn check_alert_rules(
        metrics: &Arc<RwLock<HashMap<String, VecDeque<MetricPoint>>>>,
        active_alerts: &Arc<RwLock<HashMap<String, Alert>>>,
        alert_history: &Arc<RwLock<Vec<Alert>>>,
        config: &MonitoringConfig,
    ) -> Result<()> {
        let metrics_guard = metrics.read().await;
        
        for rule in &config.alert_rules {
            if let Some(metric_points) = metrics_guard.get(&rule.metric_name) {
                if let Some(latest_point) = metric_points.back() {
                    let should_alert = match &rule.condition {
                        AlertCondition::GreaterThan(threshold) => latest_point.value > *threshold,
                        AlertCondition::LessThan(threshold) => latest_point.value < *threshold,
                        AlertCondition::OutsideRange(min, max) => {
                            latest_point.value < *min || latest_point.value > *max
                        }
                        AlertCondition::RateOfChange(threshold) => {
                            if metric_points.len() >= 2 {
                                let prev_point = &metric_points[metric_points.len() - 2];
                                let rate = (latest_point.value - prev_point.value) / 
                                          (latest_point.timestamp - prev_point.timestamp).num_seconds() as f64;
                                rate.abs() > *threshold
                            } else {
                                false
                            }
                        }
                    };
                    
                    if should_alert {
                        let alert = Alert {
                            id: uuid::Uuid::new_v4().to_string(),
                            rule_name: rule.name.clone(),
                            message: format!("Alert: {}", rule.description),
                            description: rule.description.clone(),
                            severity: rule.severity.clone(),
                            timestamp: Utc::now(),
                            status: AlertStatus::Active,
                            trigger_value: latest_point.value,
                        };
                        
                        active_alerts.write().await.insert(alert.id.clone(), alert.clone());
                        alert_history.write().await.push(alert);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Clean up old metrics
    async fn cleanup_old_metrics(
        metrics: &Arc<RwLock<HashMap<String, VecDeque<MetricPoint>>>>,
        config: &MonitoringConfig,
    ) -> Result<()> {
        let mut metrics_guard = metrics.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::minutes(config.history_retention_minutes as i64);
        
        for metric_points in metrics_guard.values_mut() {
            while let Some(point) = metric_points.front() {
                if point.timestamp < cutoff_time {
                    metric_points.pop_front();
                } else {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get CPU usage
    async fn get_cpu_usage() -> Result<f64> {
        // Simulate CPU usage
        Ok(25.0)
    }
    
    /// Get memory usage
    async fn get_memory_usage() -> Result<f64> {
        // Simulate memory usage
        Ok(50.0 * 1024.0 * 1024.0)
    }
}

/// Monitoring error types
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Metric collection error
    #[error("Metric collection error: {0}")]
    Collection(String),
    
    /// Alert processing error
    #[error("Alert processing error: {0}")]
    Alert(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = PerformanceMonitor::new("test-component", config).await;
        assert!(monitor.is_ok());
    }
    
    #[tokio::test]
    async fn test_metric_recording() {
        let config = MonitoringConfig::default();
        let monitor = PerformanceMonitor::new("test-component", config).await.unwrap();
        
        let labels = HashMap::new();
        let result = monitor.record_metric("test_metric", 100.0, labels).await;
        assert!(result.is_ok());
        
        let metrics = monitor.get_metrics().await;
        assert!(metrics.contains_key("test_metric"));
    }
}