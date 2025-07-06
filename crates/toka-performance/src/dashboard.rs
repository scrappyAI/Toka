//! Performance dashboard and visualization
//!
//! This module provides real-time performance dashboards and visualization
//! capabilities for monitoring system performance and health.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable dashboard
    pub enabled: bool,
    /// Dashboard refresh interval
    pub refresh_interval_seconds: u64,
    /// Maximum data points to display
    pub max_data_points: usize,
    /// Dashboard theme
    pub theme: DashboardTheme,
    /// Widget configurations
    pub widgets: Vec<WidgetConfig>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            refresh_interval_seconds: 30,
            max_data_points: 100,
            theme: DashboardTheme::Dark,
            widgets: Vec::new(),
        }
    }
}

/// Dashboard theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// High contrast theme
    HighContrast,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Widget ID
    pub id: String,
    /// Widget type
    pub widget_type: WidgetType,
    /// Widget title
    pub title: String,
    /// Metric name to display
    pub metric_name: String,
    /// Widget position
    pub position: WidgetPosition,
    /// Widget size
    pub size: WidgetSize,
    /// Widget specific configuration
    pub config: serde_json::Value,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    /// Line chart for time series data
    LineChart,
    /// Bar chart for categorical data
    BarChart,
    /// Gauge for single value display
    Gauge,
    /// Counter for cumulative values
    Counter,
    /// Table for tabular data
    Table,
    /// Heatmap for correlation data
    Heatmap,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Performance dashboard
#[derive(Debug)]
pub struct Dashboard {
    /// Component identifier
    component_id: String,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboard data
    data: Arc<RwLock<DashboardData>>,
    /// Dashboard server task
    server_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Component ID
    pub component_id: String,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Metric visualizations
    pub visualizations: HashMap<String, MetricVisualization>,
    /// System overview
    pub system_overview: SystemOverview,
    /// Performance summary
    pub performance_summary: PerformanceSummary,
}

/// Metric visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricVisualization {
    /// Visualization ID
    pub id: String,
    /// Visualization type
    pub visualization_type: VisualizationType,
    /// Metric name
    pub metric_name: String,
    /// Data points
    pub data_points: Vec<DataPoint>,
    /// Visualization configuration
    pub config: VisualizationConfig,
}

/// Visualization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    /// Time series line chart
    TimeSeries,
    /// Real-time gauge
    Gauge,
    /// Histogram
    Histogram,
    /// Trend analysis
    Trend,
}

/// Data point for visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Optional label
    pub label: Option<String>,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Y-axis minimum value
    pub y_min: Option<f64>,
    /// Y-axis maximum value
    pub y_max: Option<f64>,
    /// Color scheme
    pub color_scheme: String,
    /// Show grid
    pub show_grid: bool,
    /// Show legend
    pub show_legend: bool,
}

/// System overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    /// System health status
    pub health_status: String,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Disk usage in GB
    pub disk_usage_gb: f64,
    /// Network I/O in MB/s
    pub network_io_mbps: f64,
    /// Active connections
    pub active_connections: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Performance summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Operations per second
    pub ops_per_second: f64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// P95 latency in ms
    pub p95_latency_ms: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Throughput trend
    pub throughput_trend: TrendDirection,
    /// Latency trend
    pub latency_trend: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving trend
    Up,
    /// Stable trend
    Stable,
    /// Degrading trend
    Down,
}

impl Dashboard {
    /// Create a new performance dashboard
    pub async fn new(component_id: &str, config: DashboardConfig) -> Result<Self> {
        let data = DashboardData {
            component_id: component_id.to_string(),
            last_updated: Utc::now(),
            visualizations: HashMap::new(),
            system_overview: SystemOverview {
                health_status: "Healthy".to_string(),
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
                disk_usage_gb: 0.0,
                network_io_mbps: 0.0,
                active_connections: 0,
                uptime_seconds: 0,
            },
            performance_summary: PerformanceSummary {
                ops_per_second: 0.0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                error_rate_percent: 0.0,
                throughput_trend: TrendDirection::Stable,
                latency_trend: TrendDirection::Stable,
            },
        };
        
        Ok(Self {
            component_id: component_id.to_string(),
            config,
            data: Arc::new(RwLock::new(data)),
            server_task: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Start the dashboard server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let component_id = self.component_id.clone();
        let data = Arc::clone(&self.data);
        let config = self.config.clone();
        
        let task = tokio::spawn(async move {
            tracing::info!(
                component = %component_id,
                "Starting dashboard server"
            );
            
            // Simulate dashboard server
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(config.refresh_interval_seconds)).await;
                
                // Update dashboard data
                if let Err(e) = Self::update_dashboard_data(&data, &component_id).await {
                    tracing::error!(
                        component = %component_id,
                        error = %e,
                        "Failed to update dashboard data"
                    );
                }
            }
        });
        
        *self.server_task.write().await = Some(task);
        Ok(())
    }
    
    /// Stop the dashboard server
    pub async fn stop(&self) -> Result<()> {
        let mut task = self.server_task.write().await;
        if let Some(task) = task.take() {
            task.abort();
        }
        Ok(())
    }
    
    /// Add a metric visualization
    pub async fn add_visualization(&self, visualization: MetricVisualization) -> Result<()> {
        let mut data = self.data.write().await;
        data.visualizations.insert(visualization.id.clone(), visualization);
        data.last_updated = Utc::now();
        Ok(())
    }
    
    /// Update metric data
    pub async fn update_metric(&self, metric_name: &str, value: f64, timestamp: DateTime<Utc>) -> Result<()> {
        let mut data = self.data.write().await;
        
        // Find visualizations for this metric
        for visualization in data.visualizations.values_mut() {
            if visualization.metric_name == metric_name {
                let data_point = DataPoint {
                    timestamp,
                    value,
                    label: None,
                };
                
                visualization.data_points.push(data_point);
                
                // Limit data points to prevent memory growth
                if visualization.data_points.len() > self.config.max_data_points {
                    visualization.data_points.drain(0..visualization.data_points.len() - self.config.max_data_points);
                }
            }
        }
        
        data.last_updated = Utc::now();
        Ok(())
    }
    
    /// Get dashboard data
    pub async fn get_data(&self) -> DashboardData {
        self.data.read().await.clone()
    }
    
    /// Get dashboard data as JSON
    pub async fn get_data_json(&self) -> Result<serde_json::Value> {
        let data = self.data.read().await;
        Ok(serde_json::to_value(&*data)?)
    }
    
    /// Update dashboard data
    async fn update_dashboard_data(
        data: &Arc<RwLock<DashboardData>>,
        _component_id: &str,
    ) -> Result<()> {
        let mut data_guard = data.write().await;
        
        // Update system overview
        data_guard.system_overview.cpu_usage_percent = Self::get_cpu_usage().await?;
        data_guard.system_overview.memory_usage_mb = Self::get_memory_usage().await?;
        data_guard.system_overview.disk_usage_gb = Self::get_disk_usage().await?;
        data_guard.system_overview.network_io_mbps = Self::get_network_io().await?;
        data_guard.system_overview.uptime_seconds += 30; // Increment by refresh interval
        
        // Update performance summary
        data_guard.performance_summary.ops_per_second = Self::get_ops_per_second().await?;
        data_guard.performance_summary.avg_latency_ms = Self::get_avg_latency().await?;
        data_guard.performance_summary.p95_latency_ms = Self::get_p95_latency().await?;
        data_guard.performance_summary.error_rate_percent = Self::get_error_rate().await?;
        
        data_guard.last_updated = Utc::now();
        Ok(())
    }
    
    // Simulate system metrics collection
    async fn get_cpu_usage() -> Result<f64> {
        Ok(25.0 + fastrand::f64() * 10.0) // 25-35%
    }
    
    async fn get_memory_usage() -> Result<f64> {
        Ok(50.0 + fastrand::f64() * 20.0) // 50-70 MB
    }
    
    async fn get_disk_usage() -> Result<f64> {
        Ok(1.0 + fastrand::f64() * 0.5) // 1.0-1.5 GB
    }
    
    async fn get_network_io() -> Result<f64> {
        Ok(0.5 + fastrand::f64() * 1.0) // 0.5-1.5 MB/s
    }
    
    async fn get_ops_per_second() -> Result<f64> {
        Ok(800.0 + fastrand::f64() * 400.0) // 800-1200 ops/sec
    }
    
    async fn get_avg_latency() -> Result<f64> {
        Ok(2.0 + fastrand::f64() * 3.0) // 2-5 ms
    }
    
    async fn get_p95_latency() -> Result<f64> {
        Ok(5.0 + fastrand::f64() * 5.0) // 5-10 ms
    }
    
    async fn get_error_rate() -> Result<f64> {
        Ok(fastrand::f64() * 2.0) // 0-2%
    }
}

/// Dashboard error types
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    /// Dashboard not enabled
    #[error("Dashboard is not enabled")]
    NotEnabled,
    
    /// Visualization not found
    #[error("Visualization '{0}' not found")]
    VisualizationNotFound(String),
    
    /// Configuration error
    #[error("Dashboard configuration error: {0}")]
    Config(String),
    
    /// Server error
    #[error("Dashboard server error: {0}")]
    Server(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = Dashboard::new("test-component", config).await;
        assert!(dashboard.is_ok());
    }
    
    #[tokio::test]
    async fn test_visualization_addition() {
        let config = DashboardConfig::default();
        let dashboard = Dashboard::new("test-component", config).await.unwrap();
        
        let visualization = MetricVisualization {
            id: "test_viz".to_string(),
            visualization_type: VisualizationType::TimeSeries,
            metric_name: "cpu_usage".to_string(),
            data_points: Vec::new(),
            config: VisualizationConfig {
                y_min: Some(0.0),
                y_max: Some(100.0),
                color_scheme: "blue".to_string(),
                show_grid: true,
                show_legend: true,
            },
        };
        
        let result = dashboard.add_visualization(visualization).await;
        assert!(result.is_ok());
        
        let data = dashboard.get_data().await;
        assert!(data.visualizations.contains_key("test_viz"));
    }
    
    #[tokio::test]
    async fn test_metric_update() {
        let config = DashboardConfig::default();
        let dashboard = Dashboard::new("test-component", config).await.unwrap();
        
        let visualization = MetricVisualization {
            id: "test_viz".to_string(),
            visualization_type: VisualizationType::TimeSeries,
            metric_name: "cpu_usage".to_string(),
            data_points: Vec::new(),
            config: VisualizationConfig {
                y_min: Some(0.0),
                y_max: Some(100.0),
                color_scheme: "blue".to_string(),
                show_grid: true,
                show_legend: true,
            },
        };
        
        dashboard.add_visualization(visualization).await.unwrap();
        
        let result = dashboard.update_metric("cpu_usage", 50.0, Utc::now()).await;
        assert!(result.is_ok());
        
        let data = dashboard.get_data().await;
        let viz = data.visualizations.get("test_viz").unwrap();
        assert_eq!(viz.data_points.len(), 1);
        assert_eq!(viz.data_points[0].value, 50.0);
    }
}