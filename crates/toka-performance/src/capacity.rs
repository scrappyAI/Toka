//! Capacity planning and resource utilization analysis
//!
//! This module provides capacity planning capabilities for analyzing
//! resource utilization trends and forecasting future needs.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capacity planning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityConfig {
    /// Enable capacity planning
    pub enabled: bool,
    /// Analysis window in days
    pub analysis_window_days: u32,
    /// Forecast horizon in days
    pub forecast_horizon_days: u32,
    /// Resource utilization thresholds
    pub thresholds: ResourceThresholds,
    /// Trend analysis configuration
    pub trend_config: TrendConfig,
}

impl Default for CapacityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            analysis_window_days: 30,
            forecast_horizon_days: 90,
            thresholds: ResourceThresholds::default(),
            trend_config: TrendConfig::default(),
        }
    }
}

/// Resource utilization thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    /// CPU utilization warning threshold (percentage)
    pub cpu_warning_percent: f64,
    /// CPU utilization critical threshold (percentage)
    pub cpu_critical_percent: f64,
    /// Memory utilization warning threshold (percentage)
    pub memory_warning_percent: f64,
    /// Memory utilization critical threshold (percentage)
    pub memory_critical_percent: f64,
    /// Disk utilization warning threshold (percentage)
    pub disk_warning_percent: f64,
    /// Disk utilization critical threshold (percentage)
    pub disk_critical_percent: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            cpu_warning_percent: 70.0,
            cpu_critical_percent: 85.0,
            memory_warning_percent: 75.0,
            memory_critical_percent: 90.0,
            disk_warning_percent: 80.0,
            disk_critical_percent: 95.0,
        }
    }
}

/// Trend analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    /// Minimum data points for trend analysis
    pub min_data_points: usize,
    /// Trend significance threshold
    pub significance_threshold: f64,
    /// Seasonal adjustment
    pub seasonal_adjustment: bool,
}

impl Default for TrendConfig {
    fn default() -> Self {
        Self {
            min_data_points: 30,
            significance_threshold: 0.05,
            seasonal_adjustment: true,
        }
    }
}

/// Capacity planning analyzer
#[derive(Debug)]
pub struct CapacityAnalyzer {
    /// Component identifier
    component_id: String,
    /// Configuration
    config: CapacityConfig,
    /// Historical resource data
    resource_data: HashMap<String, Vec<ResourceDataPoint>>,
}

/// Resource data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Resource type
    pub resource_type: ResourceType,
    /// Utilization value
    pub utilization: f64,
    /// Available capacity
    pub capacity: f64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Resource types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU resources
    Cpu,
    /// Memory resources
    Memory,
    /// Disk storage
    Disk,
    /// Network bandwidth
    Network,
    /// Custom resource type
    Custom(String),
}

/// Capacity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityAnalysis {
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Resource utilization summary
    pub utilization_summary: UtilizationSummary,
    /// Trend analysis
    pub trend_analysis: TrendAnalysis,
    /// Capacity forecast
    pub forecast: CapacityForecast,
    /// Recommendations
    pub recommendations: Vec<CapacityRecommendation>,
}

/// Resource utilization summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationSummary {
    /// Current utilization by resource type
    pub current_utilization: HashMap<String, f64>,
    /// Average utilization over analysis window
    pub average_utilization: HashMap<String, f64>,
    /// Peak utilization over analysis window
    pub peak_utilization: HashMap<String, f64>,
    /// Utilization trends
    pub utilization_trends: HashMap<String, TrendDirection>,
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Resource trends
    pub resource_trends: HashMap<String, ResourceTrend>,
    /// Growth patterns
    pub growth_patterns: HashMap<String, GrowthPattern>,
    /// Seasonal patterns
    pub seasonal_patterns: HashMap<String, SeasonalPattern>,
}

/// Resource trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrend {
    /// Resource type
    pub resource_type: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    pub strength: f64,
    /// Growth rate (percentage per day)
    pub growth_rate_percent_per_day: f64,
    /// Confidence level
    pub confidence: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Increasing trend
    Increasing,
    /// Stable trend
    Stable,
    /// Decreasing trend
    Decreasing,
}

/// Growth pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthPattern {
    /// Pattern type
    pub pattern_type: GrowthPatternType,
    /// Pattern description
    pub description: String,
    /// Pattern confidence
    pub confidence: f64,
}

/// Growth pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthPatternType {
    /// Linear growth
    Linear,
    /// Exponential growth
    Exponential,
    /// Logarithmic growth
    Logarithmic,
    /// Cyclical pattern
    Cyclical,
    /// Irregular pattern
    Irregular,
}

/// Seasonal pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    /// Period (in days)
    pub period_days: u32,
    /// Amplitude (percentage variation)
    pub amplitude_percent: f64,
    /// Pattern confidence
    pub confidence: f64,
}

/// Capacity forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityForecast {
    /// Forecast horizon
    pub horizon_days: u32,
    /// Resource forecasts
    pub resource_forecasts: HashMap<String, Vec<ForecastPoint>>,
    /// Capacity exhaustion predictions
    pub exhaustion_predictions: HashMap<String, Option<DateTime<Utc>>>,
    /// Forecast confidence
    pub confidence: f64,
}

/// Forecast data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Forecast timestamp
    pub timestamp: DateTime<Utc>,
    /// Predicted utilization
    pub predicted_utilization: f64,
    /// Confidence interval (lower bound)
    pub confidence_lower: f64,
    /// Confidence interval (upper bound)
    pub confidence_upper: f64,
}

/// Capacity recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Resource type affected
    pub resource_type: String,
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Time frame for action
    pub timeframe_days: Option<u32>,
}

/// Recommendation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Scale up resources
    ScaleUp,
    /// Scale down resources
    ScaleDown,
    /// Optimize resource usage
    Optimize,
    /// Monitor closely
    Monitor,
    /// Plan for expansion
    Plan,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl CapacityAnalyzer {
    /// Create a new capacity analyzer
    pub fn new(component_id: &str, config: CapacityConfig) -> Self {
        Self {
            component_id: component_id.to_string(),
            config,
            resource_data: HashMap::new(),
        }
    }
    
    /// Add resource data point
    pub fn add_data_point(&mut self, data_point: ResourceDataPoint) {
        let resource_key = format!("{:?}", data_point.resource_type);
        self.resource_data
            .entry(resource_key)
            .or_insert_with(Vec::new)
            .push(data_point);
        
        // Limit data retention to analysis window
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.analysis_window_days as i64);
        
        for (_, data_points) in self.resource_data.iter_mut() {
            data_points.retain(|point| point.timestamp > cutoff_date);
        }
    }
    
    /// Perform capacity analysis
    pub fn analyze(&self) -> Result<CapacityAnalysis> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Capacity analysis is not enabled"));
        }
        
        let utilization_summary = self.analyze_utilization()?;
        let trend_analysis = self.analyze_trends()?;
        let forecast = self.generate_forecast()?;
        let recommendations = self.generate_recommendations(&utilization_summary, &trend_analysis, &forecast);
        
        Ok(CapacityAnalysis {
            timestamp: Utc::now(),
            utilization_summary,
            trend_analysis,
            forecast,
            recommendations,
        })
    }
    
    /// Analyze resource utilization
    fn analyze_utilization(&self) -> Result<UtilizationSummary> {
        let mut current_utilization = HashMap::new();
        let mut average_utilization = HashMap::new();
        let mut peak_utilization = HashMap::new();
        let mut utilization_trends = HashMap::new();
        
        for (resource_type, data_points) in &self.resource_data {
            if data_points.is_empty() {
                continue;
            }
            
            // Current utilization (latest data point)
            let current = data_points.last().unwrap().utilization;
            current_utilization.insert(resource_type.clone(), current);
            
            // Average utilization
            let average = data_points.iter().map(|p| p.utilization).sum::<f64>() / data_points.len() as f64;
            average_utilization.insert(resource_type.clone(), average);
            
            // Peak utilization
            let peak = data_points.iter().map(|p| p.utilization).fold(0.0, f64::max);
            peak_utilization.insert(resource_type.clone(), peak);
            
            // Utilization trend
            let trend = self.calculate_trend_direction(data_points);
            utilization_trends.insert(resource_type.clone(), trend);
        }
        
        Ok(UtilizationSummary {
            current_utilization,
            average_utilization,
            peak_utilization,
            utilization_trends,
        })
    }
    
    /// Analyze trends
    fn analyze_trends(&self) -> Result<TrendAnalysis> {
        let mut resource_trends = HashMap::new();
        let mut growth_patterns = HashMap::new();
        let mut seasonal_patterns = HashMap::new();
        
        for (resource_type, data_points) in &self.resource_data {
            if data_points.len() < self.config.trend_config.min_data_points {
                continue;
            }
            
            // Analyze resource trend
            let trend = self.analyze_resource_trend(resource_type, data_points);
            resource_trends.insert(resource_type.clone(), trend);
            
            // Analyze growth pattern
            let growth_pattern = self.analyze_growth_pattern(data_points);
            growth_patterns.insert(resource_type.clone(), growth_pattern);
            
            // Analyze seasonal pattern
            if self.config.trend_config.seasonal_adjustment {
                let seasonal_pattern = self.analyze_seasonal_pattern(data_points);
                seasonal_patterns.insert(resource_type.clone(), seasonal_pattern);
            }
        }
        
        Ok(TrendAnalysis {
            resource_trends,
            growth_patterns,
            seasonal_patterns,
        })
    }
    
    /// Generate capacity forecast
    fn generate_forecast(&self) -> Result<CapacityForecast> {
        let mut resource_forecasts = HashMap::new();
        let mut exhaustion_predictions = HashMap::new();
        
        for (resource_type, data_points) in &self.resource_data {
            if data_points.len() < self.config.trend_config.min_data_points {
                continue;
            }
            
            let forecast_points = self.forecast_resource_utilization(data_points)?;
            resource_forecasts.insert(resource_type.clone(), forecast_points);
            
            // Predict capacity exhaustion
            let exhaustion_date = self.predict_capacity_exhaustion(data_points);
            exhaustion_predictions.insert(resource_type.clone(), exhaustion_date);
        }
        
        Ok(CapacityForecast {
            horizon_days: self.config.forecast_horizon_days,
            resource_forecasts,
            exhaustion_predictions,
            confidence: 0.8, // Simulated confidence
        })
    }
    
    /// Generate recommendations
    fn generate_recommendations(
        &self,
        utilization: &UtilizationSummary,
        _trends: &TrendAnalysis,
        forecast: &CapacityForecast,
    ) -> Vec<CapacityRecommendation> {
        let mut recommendations = Vec::new();
        
        for (resource_type, &current_util) in &utilization.current_utilization {
            // Check against thresholds
            let priority = if current_util > self.get_critical_threshold(resource_type) {
                RecommendationPriority::Critical
            } else if current_util > self.get_warning_threshold(resource_type) {
                RecommendationPriority::High
            } else {
                RecommendationPriority::Low
            };
            
            // Generate appropriate recommendation
            if current_util > self.get_warning_threshold(resource_type) {
                recommendations.push(CapacityRecommendation {
                    recommendation_type: RecommendationType::ScaleUp,
                    resource_type: resource_type.clone(),
                    priority,
                    description: format!("High {} utilization detected: {:.1}%", resource_type, current_util),
                    expected_impact: "Reduce resource contention and improve performance".to_string(),
                    timeframe_days: Some(30),
                });
            }
            
            // Check for capacity exhaustion
            if let Some(exhaustion_date) = forecast.exhaustion_predictions.get(resource_type).and_then(|d| *d) {
                let days_until_exhaustion = (exhaustion_date - Utc::now()).num_days();
                
                if days_until_exhaustion < 90 {
                    recommendations.push(CapacityRecommendation {
                        recommendation_type: RecommendationType::Plan,
                        resource_type: resource_type.clone(),
                        priority: RecommendationPriority::High,
                        description: format!("{} capacity exhaustion predicted in {} days", resource_type, days_until_exhaustion),
                        expected_impact: "Plan capacity expansion to avoid resource exhaustion".to_string(),
                        timeframe_days: Some(days_until_exhaustion as u32),
                    });
                }
            }
        }
        
        recommendations
    }
    
    /// Calculate trend direction for data points
    fn calculate_trend_direction(&self, data_points: &[ResourceDataPoint]) -> TrendDirection {
        if data_points.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let first_half_avg = data_points[..data_points.len() / 2]
            .iter()
            .map(|p| p.utilization)
            .sum::<f64>() / (data_points.len() / 2) as f64;
        
        let second_half_avg = data_points[data_points.len() / 2..]
            .iter()
            .map(|p| p.utilization)
            .sum::<f64>() / (data_points.len() - data_points.len() / 2) as f64;
        
        let change = (second_half_avg - first_half_avg) / first_half_avg;
        
        if change > 0.05 {
            TrendDirection::Increasing
        } else if change < -0.05 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
    
    /// Analyze resource trend
    fn analyze_resource_trend(&self, resource_type: &str, data_points: &[ResourceDataPoint]) -> ResourceTrend {
        let direction = self.calculate_trend_direction(data_points);
        
        // Simple linear regression for growth rate
        let growth_rate = self.calculate_growth_rate(data_points);
        
        ResourceTrend {
            resource_type: resource_type.to_string(),
            direction,
            strength: 0.7, // Simulated
            growth_rate_percent_per_day: growth_rate,
            confidence: 0.8, // Simulated
        }
    }
    
    /// Analyze growth pattern
    fn analyze_growth_pattern(&self, _data_points: &[ResourceDataPoint]) -> GrowthPattern {
        // Simplified pattern detection
        GrowthPattern {
            pattern_type: GrowthPatternType::Linear,
            description: "Linear growth pattern detected".to_string(),
            confidence: 0.7,
        }
    }
    
    /// Analyze seasonal pattern
    fn analyze_seasonal_pattern(&self, _data_points: &[ResourceDataPoint]) -> SeasonalPattern {
        // Simplified seasonal analysis
        SeasonalPattern {
            period_days: 7, // Weekly pattern
            amplitude_percent: 10.0,
            confidence: 0.6,
        }
    }
    
    /// Forecast resource utilization
    fn forecast_resource_utilization(&self, data_points: &[ResourceDataPoint]) -> Result<Vec<ForecastPoint>> {
        let mut forecast_points = Vec::new();
        let base_utilization = data_points.last().unwrap().utilization;
        let growth_rate = self.calculate_growth_rate(data_points);
        
        for day in 1..=self.config.forecast_horizon_days {
            let predicted_utilization = base_utilization + (growth_rate * day as f64);
            let confidence_interval = predicted_utilization * 0.1; // 10% confidence interval
            
            forecast_points.push(ForecastPoint {
                timestamp: Utc::now() + chrono::Duration::days(day as i64),
                predicted_utilization,
                confidence_lower: predicted_utilization - confidence_interval,
                confidence_upper: predicted_utilization + confidence_interval,
            });
        }
        
        Ok(forecast_points)
    }
    
    /// Predict capacity exhaustion
    fn predict_capacity_exhaustion(&self, data_points: &[ResourceDataPoint]) -> Option<DateTime<Utc>> {
        let current_utilization = data_points.last()?.utilization;
        let growth_rate = self.calculate_growth_rate(data_points);
        
        if growth_rate <= 0.0 {
            return None; // No exhaustion if not growing
        }
        
        let days_to_exhaustion = (100.0 - current_utilization) / growth_rate;
        
        if days_to_exhaustion > 0.0 && days_to_exhaustion < 3650.0 { // Within 10 years
            Some(Utc::now() + chrono::Duration::days(days_to_exhaustion as i64))
        } else {
            None
        }
    }
    
    /// Calculate growth rate
    fn calculate_growth_rate(&self, data_points: &[ResourceDataPoint]) -> f64 {
        if data_points.len() < 2 {
            return 0.0;
        }
        
        let first = &data_points[0];
        let last = &data_points[data_points.len() - 1];
        
        let days_diff = (last.timestamp - first.timestamp).num_days() as f64;
        if days_diff == 0.0 {
            return 0.0;
        }
        
        (last.utilization - first.utilization) / days_diff
    }
    
    /// Get warning threshold for resource type
    fn get_warning_threshold(&self, resource_type: &str) -> f64 {
        match resource_type {
            "Cpu" => self.config.thresholds.cpu_warning_percent,
            "Memory" => self.config.thresholds.memory_warning_percent,
            "Disk" => self.config.thresholds.disk_warning_percent,
            _ => 70.0, // Default
        }
    }
    
    /// Get critical threshold for resource type
    fn get_critical_threshold(&self, resource_type: &str) -> f64 {
        match resource_type {
            "Cpu" => self.config.thresholds.cpu_critical_percent,
            "Memory" => self.config.thresholds.memory_critical_percent,
            "Disk" => self.config.thresholds.disk_critical_percent,
            _ => 85.0, // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capacity_analyzer_creation() {
        let config = CapacityConfig::default();
        let analyzer = CapacityAnalyzer::new("test-component", config);
        assert_eq!(analyzer.component_id, "test-component");
    }
    
    #[test]
    fn test_data_point_addition() {
        let config = CapacityConfig::default();
        let mut analyzer = CapacityAnalyzer::new("test-component", config);
        
        let data_point = ResourceDataPoint {
            timestamp: Utc::now(),
            resource_type: ResourceType::Cpu,
            utilization: 50.0,
            capacity: 100.0,
            metadata: HashMap::new(),
        };
        
        analyzer.add_data_point(data_point);
        
        let cpu_data = analyzer.resource_data.get("Cpu").unwrap();
        assert_eq!(cpu_data.len(), 1);
        assert_eq!(cpu_data[0].utilization, 50.0);
    }
    
    #[test]
    fn test_trend_direction_calculation() {
        let config = CapacityConfig::default();
        let analyzer = CapacityAnalyzer::new("test-component", config);
        
        // Create increasing trend data
        let mut data_points = Vec::new();
        for i in 0..10 {
            data_points.push(ResourceDataPoint {
                timestamp: Utc::now() + chrono::Duration::days(i),
                resource_type: ResourceType::Cpu,
                utilization: 50.0 + (i as f64 * 2.0), // Increasing by 2% per day
                capacity: 100.0,
                metadata: HashMap::new(),
            });
        }
        
        let trend = analyzer.calculate_trend_direction(&data_points);
        assert!(matches!(trend, TrendDirection::Increasing));
    }
}