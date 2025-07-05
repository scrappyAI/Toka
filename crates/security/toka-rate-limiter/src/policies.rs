//! Rate limiting policies for advanced security scenarios

use crate::{
    RateLimitResult, RateLimitError, AuthRateLimitContext,
    algorithms::RateLimitPolicy,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration, Timelike, Datelike};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{warn, debug};

/// Composite policy that applies multiple policies in sequence
pub struct CompositePolicy {
    policies: Vec<Arc<dyn RateLimitPolicy>>,
}

impl CompositePolicy {
    /// Create a new composite policy
    pub fn new(policies: Vec<Arc<dyn RateLimitPolicy>>) -> Self {
        Self { policies }
    }

    /// Add a policy to the composite
    pub fn add_policy(mut self, policy: Arc<dyn RateLimitPolicy>) -> Self {
        self.policies.push(policy);
        self
    }
}

#[async_trait]
impl RateLimitPolicy for CompositePolicy {
    async fn apply(
        &self,
        context: &AuthRateLimitContext,
        mut result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        for policy in &self.policies {
            result = policy.apply(context, result).await?;
        }
        Ok(result)
    }
}

/// Geographical rate limiting policy based on IP location
/// 
/// This policy can apply different rate limits based on the geographical
/// location of the client IP address. Useful for complying with regional
/// regulations or reducing risk from high-risk regions.
pub struct GeographicalPolicy {
    /// Default rate limit multiplier
    default_multiplier: f64,
    /// Rate limit multipliers by country code
    country_multipliers: HashMap<String, f64>,
    /// Countries that are completely blocked
    blocked_countries: Vec<String>,
}

impl GeographicalPolicy {
    /// Create a new geographical policy
    pub fn new(default_multiplier: f64) -> Self {
        Self {
            default_multiplier,
            country_multipliers: HashMap::new(),
            blocked_countries: Vec::new(),
        }
    }

    /// Set multiplier for a specific country
    pub fn set_country_multiplier(mut self, country_code: impl Into<String>, multiplier: f64) -> Self {
        self.country_multipliers.insert(country_code.into(), multiplier);
        self
    }

    /// Block a country completely
    pub fn block_country(mut self, country_code: impl Into<String>) -> Self {
        self.blocked_countries.push(country_code.into());
        self
    }

    /// Get country code from IP address (simplified implementation)
    fn get_country_code(&self, _ip: std::net::IpAddr) -> Option<String> {
        // In a real implementation, this would use a GeoIP database
        // For now, we'll return None to indicate unknown location
        None
    }

    /// Check if country is blocked
    fn is_country_blocked(&self, country_code: &str) -> bool {
        self.blocked_countries.contains(&country_code.to_string())
    }

    /// Get multiplier for country
    fn get_country_multiplier(&self, country_code: &str) -> f64 {
        self.country_multipliers
            .get(country_code)
            .copied()
            .unwrap_or(self.default_multiplier)
    }
}

#[async_trait]
impl RateLimitPolicy for GeographicalPolicy {
    async fn apply(
        &self,
        context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        if let Some(ip) = context.ip_address {
            if let Some(country) = self.get_country_code(ip) {
                // Check if country is blocked
                if self.is_country_blocked(&country) {
                    warn!(
                        ip = %ip,
                        country = %country,
                        "Request blocked due to geographical restrictions"
                    );
                    
                    return Ok(RateLimitResult::Limited {
                        retry_after: Duration::hours(24), // Very long retry
                        limit: 0,
                        current_usage: 1,
                    });
                }

                // Apply country-specific multiplier
                let multiplier = self.get_country_multiplier(&country);
                
                return Ok(match result {
                    RateLimitResult::Allowed { remaining, reset_after, limit } => {
                        let adjusted_remaining = (remaining as f64 * multiplier) as u64;
                        debug!(
                            ip = %ip,
                            country = %country,
                            multiplier = multiplier,
                            original_remaining = remaining,
                            adjusted_remaining = adjusted_remaining,
                            "Applied geographical rate limit adjustment"
                        );
                        
                        RateLimitResult::Allowed {
                            remaining: adjusted_remaining,
                            reset_after,
                            limit,
                        }
                    }
                    RateLimitResult::Limited { retry_after, limit, current_usage } => {
                        let adjusted_retry = Duration::milliseconds(
                            (retry_after.num_milliseconds() as f64 / multiplier) as i64
                        );
                        
                        RateLimitResult::Limited {
                            retry_after: adjusted_retry,
                            limit,
                            current_usage,
                        }
                    }
                });
            }
        }

        Ok(result)
    }
}

/// Time-based rate limiting policy
/// 
/// Applies different rate limits based on time of day, day of week, etc.
/// Useful for reducing limits during off-hours or high-traffic periods.
pub struct TemporalPolicy {
    /// Rate limit configurations by time period
    time_configs: Vec<TemporalConfig>,
    /// Default configuration if no time-specific config matches
    default_config: Option<TemporalConfig>,
}

/// Time-based configuration
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    /// Name of this configuration
    pub name: String,
    /// Hour range (0-23)
    pub hour_range: Option<(u32, u32)>,
    /// Day of week range (1=Monday, 7=Sunday)
    pub day_range: Option<(u32, u32)>,
    /// Rate limit multiplier
    pub multiplier: f64,
    /// Whether this is a restrictive period
    pub is_restrictive: bool,
}

impl TemporalPolicy {
    /// Create a new temporal policy
    pub fn new() -> Self {
        Self {
            time_configs: Vec::new(),
            default_config: None,
        }
    }

    /// Add a time-based configuration
    pub fn add_config(mut self, config: TemporalConfig) -> Self {
        self.time_configs.push(config);
        self
    }

    /// Set default configuration
    pub fn with_default(mut self, config: TemporalConfig) -> Self {
        self.default_config = Some(config);
        self
    }

    /// Add off-hours configuration (reduced limits during night)
    pub fn with_off_hours(self, start_hour: u32, end_hour: u32, multiplier: f64) -> Self {
        self.add_config(TemporalConfig {
            name: "off_hours".to_string(),
            hour_range: Some((start_hour, end_hour)),
            day_range: None,
            multiplier,
            is_restrictive: true,
        })
    }

    /// Add weekend configuration
    pub fn with_weekend(self, multiplier: f64) -> Self {
        self.add_config(TemporalConfig {
            name: "weekend".to_string(),
            hour_range: None,
            day_range: Some((6, 7)), // Saturday and Sunday
            multiplier,
            is_restrictive: false,
        })
    }

    /// Find the current applicable configuration
    fn get_current_config(&self, now: DateTime<Utc>) -> Option<&TemporalConfig> {
        let hour = now.hour();
        let weekday = now.weekday().number_from_monday();

        for config in &self.time_configs {
            let hour_matches = config.hour_range
                .map(|(start, end)| {
                    if start <= end {
                        hour >= start && hour <= end
                    } else {
                        // Handles ranges that cross midnight (e.g., 22-06)
                        hour >= start || hour <= end
                    }
                })
                .unwrap_or(true);

            let day_matches = config.day_range
                .map(|(start, end)| weekday >= start && weekday <= end)
                .unwrap_or(true);

            if hour_matches && day_matches {
                return Some(config);
            }
        }

        self.default_config.as_ref()
    }
}

impl Default for TemporalPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitPolicy for TemporalPolicy {
    async fn apply(
        &self,
        _context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = Utc::now();
        
        if let Some(config) = self.get_current_config(now) {
            debug!(
                config_name = %config.name,
                multiplier = config.multiplier,
                is_restrictive = config.is_restrictive,
                hour = now.hour(),
                weekday = now.weekday().number_from_monday(),
                "Applying temporal rate limit policy"
            );

            return Ok(match result {
                RateLimitResult::Allowed { remaining, reset_after, limit } => {
                    let adjusted_remaining = (remaining as f64 * config.multiplier) as u64;
                    
                    RateLimitResult::Allowed {
                        remaining: adjusted_remaining,
                        reset_after,
                        limit,
                    }
                }
                RateLimitResult::Limited { retry_after, limit, current_usage } => {
                    let adjusted_retry = if config.is_restrictive {
                        // Extend retry time during restrictive periods
                        Duration::milliseconds(
                            (retry_after.num_milliseconds() as f64 * (2.0 - config.multiplier)) as i64
                        )
                    } else {
                        retry_after
                    };
                    
                    RateLimitResult::Limited {
                        retry_after: adjusted_retry,
                        limit,
                        current_usage,
                    }
                }
            });
        }

        Ok(result)
    }
}

/// Adaptive rate limiting policy based on system health
/// 
/// Automatically adjusts rate limits based on system metrics like
/// CPU usage, memory consumption, response times, etc.
pub struct AdaptiveSystemPolicy {
    /// Current system health metrics
    health_metrics: Arc<RwLock<SystemHealthMetrics>>,
    /// Thresholds for different adaptation levels
    thresholds: AdaptiveThresholds,
}

/// System health metrics
#[derive(Debug, Clone, Default)]
pub struct SystemHealthMetrics {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0 - 100.0)
    pub memory_usage: f64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Error rate percentage (0.0 - 100.0)
    pub error_rate: f64,
    /// Last update time
    pub last_updated: DateTime<Utc>,
}

/// Thresholds for adaptive rate limiting
#[derive(Debug, Clone)]
pub struct AdaptiveThresholds {
    /// CPU usage threshold for triggering adaptation
    pub cpu_threshold: f64,
    /// Memory usage threshold
    pub memory_threshold: f64,
    /// Response time threshold in milliseconds
    pub response_time_threshold: f64,
    /// Error rate threshold
    pub error_rate_threshold: f64,
    /// Rate limit reduction factor when thresholds are exceeded
    pub reduction_factor: f64,
}

impl Default for AdaptiveThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            response_time_threshold: 1000.0,
            error_rate_threshold: 5.0,
            reduction_factor: 0.5,
        }
    }
}

impl AdaptiveSystemPolicy {
    /// Create a new adaptive system policy
    pub fn new(thresholds: AdaptiveThresholds) -> Self {
        Self {
            health_metrics: Arc::new(RwLock::new(SystemHealthMetrics::default())),
            thresholds,
        }
    }

    /// Update system health metrics
    pub async fn update_metrics(&self, metrics: SystemHealthMetrics) {
        let mut health = self.health_metrics.write().await;
        *health = metrics;
        
        debug!(
            cpu = health.cpu_usage,
            memory = health.memory_usage,
            response_time = health.avg_response_time,
            error_rate = health.error_rate,
            "Updated system health metrics"
        );
    }

    /// Check if system is under stress
    async fn is_system_stressed(&self) -> bool {
        let metrics = self.health_metrics.read().await;
        
        metrics.cpu_usage > self.thresholds.cpu_threshold ||
        metrics.memory_usage > self.thresholds.memory_threshold ||
        metrics.avg_response_time > self.thresholds.response_time_threshold ||
        metrics.error_rate > self.thresholds.error_rate_threshold
    }

    /// Calculate adaptation factor based on system stress
    async fn calculate_adaptation_factor(&self) -> f64 {
        let metrics = self.health_metrics.read().await;
        
        let stress_factors = vec![
            metrics.cpu_usage / self.thresholds.cpu_threshold,
            metrics.memory_usage / self.thresholds.memory_threshold,
            metrics.avg_response_time / self.thresholds.response_time_threshold,
            metrics.error_rate / self.thresholds.error_rate_threshold,
        ];
        
        let max_stress = stress_factors.into_iter()
            .map(|f| f.max(1.0))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);
        
        if max_stress > 1.0 {
            (self.thresholds.reduction_factor / max_stress).max(0.1)
        } else {
            1.0
        }
    }
}

#[async_trait]
impl RateLimitPolicy for AdaptiveSystemPolicy {
    async fn apply(
        &self,
        _context: &AuthRateLimitContext,
        result: RateLimitResult,
    ) -> Result<RateLimitResult, RateLimitError> {
        if self.is_system_stressed().await {
            let adaptation_factor = self.calculate_adaptation_factor().await;
            
            warn!(
                adaptation_factor = adaptation_factor,
                "System under stress, applying adaptive rate limiting"
            );
            
            return Ok(match result {
                RateLimitResult::Allowed { remaining, reset_after, limit } => {
                    let adjusted_remaining = (remaining as f64 * adaptation_factor) as u64;
                    
                    RateLimitResult::Allowed {
                        remaining: adjusted_remaining,
                        reset_after,
                        limit,
                    }
                }
                RateLimitResult::Limited { retry_after, limit, current_usage } => {
                    let extended_retry = Duration::milliseconds(
                        (retry_after.num_milliseconds() as f64 / adaptation_factor) as i64
                    );
                    
                    RateLimitResult::Limited {
                        retry_after: extended_retry,
                        limit,
                        current_usage,
                    }
                }
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RateLimitResult;

    #[tokio::test]
    async fn test_composite_policy() {
        let policy1 = Arc::new(GeographicalPolicy::new(0.5));
        let policy2 = Arc::new(TemporalPolicy::new());
        
        let composite = CompositePolicy::new(vec![policy1, policy2]);
        let context = AuthRateLimitContext::new();
        
        let result = RateLimitResult::Allowed {
            remaining: 100,
            reset_after: Duration::minutes(1),
            limit: 100,
        };
        
        let applied = composite.apply(&context, result).await.unwrap();
        assert!(applied.is_allowed());
    }

    #[tokio::test]
    async fn test_geographical_policy() {
        let policy = GeographicalPolicy::new(1.0)
            .set_country_multiplier("US", 1.0)
            .set_country_multiplier("RU", 0.5)
            .block_country("XX");
        
        let context = AuthRateLimitContext::new()
            .with_ip("192.168.1.1".parse().unwrap());
        
        let result = RateLimitResult::Allowed {
            remaining: 100,
            reset_after: Duration::minutes(1),
            limit: 100,
        };
        
        let applied = policy.apply(&context, result).await.unwrap();
        assert!(applied.is_allowed());
    }

    #[tokio::test]
    async fn test_temporal_policy() {
        let policy = TemporalPolicy::new()
            .with_off_hours(22, 6, 0.5)
            .with_weekend(1.5);
        
        let context = AuthRateLimitContext::new();
        
        let result = RateLimitResult::Allowed {
            remaining: 100,
            reset_after: Duration::minutes(1),
            limit: 100,
        };
        
        let applied = policy.apply(&context, result).await.unwrap();
        assert!(applied.is_allowed());
    }

    #[tokio::test]
    async fn test_adaptive_system_policy() {
        let policy = AdaptiveSystemPolicy::new(AdaptiveThresholds::default());
        
        // Update with high stress metrics
        policy.update_metrics(SystemHealthMetrics {
            cpu_usage: 90.0,
            memory_usage: 95.0,
            avg_response_time: 2000.0,
            error_rate: 10.0,
            last_updated: Utc::now(),
        }).await;
        
        let context = AuthRateLimitContext::new();
        
        let result = RateLimitResult::Allowed {
            remaining: 100,
            reset_after: Duration::minutes(1),
            limit: 100,
        };
        
        let applied = policy.apply(&context, result).await.unwrap();
        
        // Should reduce remaining capacity due to system stress
        if let RateLimitResult::Allowed { remaining, .. } = applied {
            assert!(remaining < 100);
        }
    }
} 