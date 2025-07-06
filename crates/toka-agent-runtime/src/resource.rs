//! Resource management and enforcement for agent runtime.
//!
//! This module provides resource limit enforcement including CPU, memory, and timeout
//! constraints to ensure agents operate within their declared resource budgets.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::{debug, warn, error};

use toka_types::ResourceLimits;
use crate::{AgentRuntimeError, AgentRuntimeResult};

/// Manages and enforces resource limits for agent execution
pub struct ResourceManager {
    /// Resource limits configuration
    limits: ParsedResourceLimits,
    /// Current resource usage
    usage: ResourceUsage,
    /// Resource monitoring start time
    start_time: Instant,
}

/// Parsed resource limits with numeric values
#[derive(Debug, Clone)]
pub struct ParsedResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU usage (0.0 to 1.0)
    pub max_cpu_usage: f64,
    /// Maximum execution time
    pub max_execution_time: Duration,
}

/// Current resource usage tracking
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: Arc<AtomicU64>,
    /// Current CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Total execution time
    pub execution_time: Duration,
    /// Total LLM tokens consumed
    pub llm_tokens: Arc<AtomicU64>,
    /// Number of operations performed
    pub operations_count: Arc<AtomicU64>,
}

impl ResourceManager {
    /// Create a new resource manager with the given limits
    pub fn new(limits: ResourceLimits) -> Result<Self> {
        let parsed_limits = Self::parse_resource_limits(limits)?;
        
        debug!("Created resource manager with limits: memory={}MB, cpu={}%, timeout={:?}",
               parsed_limits.max_memory_bytes / 1024 / 1024,
               parsed_limits.max_cpu_usage * 100.0,
               parsed_limits.max_execution_time);

        Ok(Self {
            limits: parsed_limits,
            usage: ResourceUsage::new(),
            start_time: Instant::now(),
        })
    }

    /// Check if resources are available for operation
    pub fn check_availability(&self) -> AgentRuntimeResult<()> {
        self.check_memory_limit()?;
        self.check_cpu_limit()?;
        self.check_timeout_limit()?;
        Ok(())
    }

    /// Record resource usage for an operation
    pub fn record_usage(&mut self, tokens_used: u64, duration: Duration) -> AgentRuntimeResult<()> {
        // Update LLM token usage
        self.usage.llm_tokens.fetch_add(tokens_used, Ordering::Relaxed);
        
        // Update execution time
        self.usage.execution_time += duration;
        
        // Increment operation count
        self.usage.operations_count.fetch_add(1, Ordering::Relaxed);
        
        // Update CPU usage estimate (simplified)
        let total_time = self.start_time.elapsed();
        self.usage.cpu_usage = if total_time.as_secs_f64() > 0.0 {
            self.usage.execution_time.as_secs_f64() / total_time.as_secs_f64()
        } else {
            0.0
        };

        debug!("Resource usage updated: tokens={}, duration={:?}, cpu={:.1}%", 
               tokens_used, duration, self.usage.cpu_usage * 100.0);

        // Check limits after usage update
        self.check_availability()?;

        Ok(())
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, memory_bytes: u64) -> AgentRuntimeResult<()> {
        self.usage.memory_bytes.store(memory_bytes, Ordering::Relaxed);
        self.check_memory_limit()?;
        Ok(())
    }

    /// Get current resource usage
    pub fn get_usage(&self) -> ResourceUsageSnapshot {
        ResourceUsageSnapshot {
            memory_bytes: self.usage.memory_bytes.load(Ordering::Relaxed),
            cpu_usage: self.usage.cpu_usage,
            execution_time: self.usage.execution_time,
            llm_tokens: self.usage.llm_tokens.load(Ordering::Relaxed),
            operations_count: self.usage.operations_count.load(Ordering::Relaxed),
            uptime: self.start_time.elapsed(),
        }
    }

    /// Get resource limits
    pub fn get_limits(&self) -> &ParsedResourceLimits {
        &self.limits
    }

    /// Check if operation would exceed memory limit
    pub fn would_exceed_memory(&self, additional_bytes: u64) -> bool {
        let current_memory = self.usage.memory_bytes.load(Ordering::Relaxed);
        current_memory + additional_bytes > self.limits.max_memory_bytes
    }

    /// Check if operation would exceed timeout
    pub fn would_exceed_timeout(&self, additional_duration: Duration) -> bool {
        self.usage.execution_time + additional_duration > self.limits.max_execution_time
    }

    /// Parse string-based resource limits into numeric values
    fn parse_resource_limits(limits: ResourceLimits) -> Result<ParsedResourceLimits> {
        let max_memory_bytes = Self::parse_memory_string(&limits.max_memory)?;
        let max_cpu_usage = Self::parse_cpu_string(&limits.max_cpu)?;
        let max_execution_time = Self::parse_duration_string(&limits.timeout)?;

        Ok(ParsedResourceLimits {
            max_memory_bytes,
            max_cpu_usage,
            max_execution_time,
        })
    }

    /// Parse memory string (e.g., "100MB", "1GB") to bytes
    fn parse_memory_string(memory_str: &str) -> Result<u64> {
        let memory_str = memory_str.to_uppercase();
        
        if let Some(pos) = memory_str.find("KB") {
            let num: u64 = memory_str[..pos].parse()?;
            Ok(num * 1024)
        } else if let Some(pos) = memory_str.find("MB") {
            let num: u64 = memory_str[..pos].parse()?;
            Ok(num * 1024 * 1024)
        } else if let Some(pos) = memory_str.find("GB") {
            let num: u64 = memory_str[..pos].parse()?;
            Ok(num * 1024 * 1024 * 1024)
        } else if let Some(pos) = memory_str.find("B") {
            let num: u64 = memory_str[..pos].parse()?;
            Ok(num)
        } else {
            // Try parsing as raw number (bytes)
            Ok(memory_str.parse()?)
        }
    }

    /// Parse CPU string (e.g., "50%", "0.5") to fraction
    fn parse_cpu_string(cpu_str: &str) -> Result<f64> {
        if cpu_str.ends_with('%') {
            let num: f64 = cpu_str.trim_end_matches('%').parse()?;
            Ok(num / 100.0)
        } else {
            let num: f64 = cpu_str.parse()?;
            if num > 1.0 {
                // Assume percentage if > 1.0
                Ok(num / 100.0)
            } else {
                Ok(num)
            }
        }
    }

    /// Parse duration string (e.g., "5m", "1h", "30s") to Duration
    fn parse_duration_string(duration_str: &str) -> Result<Duration> {
        let duration_str = duration_str.to_lowercase();
        
        if let Some(pos) = duration_str.find("ms") {
            let num: u64 = duration_str[..pos].parse()?;
            Ok(Duration::from_millis(num))
        } else if let Some(pos) = duration_str.find('s') {
            let num: u64 = duration_str[..pos].parse()?;
            Ok(Duration::from_secs(num))
        } else if let Some(pos) = duration_str.find('m') {
            let num: u64 = duration_str[..pos].parse()?;
            Ok(Duration::from_secs(num * 60))
        } else if let Some(pos) = duration_str.find('h') {
            let num: u64 = duration_str[..pos].parse()?;
            Ok(Duration::from_secs(num * 3600))
        } else {
            // Try parsing as raw seconds
            let num: u64 = duration_str.parse()?;
            Ok(Duration::from_secs(num))
        }
    }

    /// Check memory limit
    fn check_memory_limit(&self) -> AgentRuntimeResult<()> {
        let current_memory = self.usage.memory_bytes.load(Ordering::Relaxed);
        if current_memory > self.limits.max_memory_bytes {
            error!("Memory limit exceeded: {}MB > {}MB",
                   current_memory / 1024 / 1024,
                   self.limits.max_memory_bytes / 1024 / 1024);
            
            return Err(AgentRuntimeError::ResourceLimitExceeded {
                resource: "memory".to_string(),
                current: format!("{}MB", current_memory / 1024 / 1024),
                limit: format!("{}MB", self.limits.max_memory_bytes / 1024 / 1024),
            });
        }
        Ok(())
    }

    /// Check CPU limit
    fn check_cpu_limit(&self) -> AgentRuntimeResult<()> {
        if self.usage.cpu_usage > self.limits.max_cpu_usage {
            warn!("CPU limit exceeded: {:.1}% > {:.1}%",
                  self.usage.cpu_usage * 100.0,
                  self.limits.max_cpu_usage * 100.0);
            
            return Err(AgentRuntimeError::ResourceLimitExceeded {
                resource: "cpu".to_string(),
                current: format!("{:.1}%", self.usage.cpu_usage * 100.0),
                limit: format!("{:.1}%", self.limits.max_cpu_usage * 100.0),
            });
        }
        Ok(())
    }

    /// Check timeout limit
    fn check_timeout_limit(&self) -> AgentRuntimeResult<()> {
        if self.usage.execution_time > self.limits.max_execution_time {
            error!("Timeout limit exceeded: {:?} > {:?}",
                   self.usage.execution_time,
                   self.limits.max_execution_time);
            
            return Err(AgentRuntimeError::ResourceLimitExceeded {
                resource: "timeout".to_string(),
                current: format!("{:?}", self.usage.execution_time),
                limit: format!("{:?}", self.limits.max_execution_time),
            });
        }
        Ok(())
    }
}

impl ResourceUsage {
    /// Create new resource usage tracker
    fn new() -> Self {
        Self {
            memory_bytes: Arc::new(AtomicU64::new(0)),
            cpu_usage: 0.0,
            execution_time: Duration::ZERO,
            llm_tokens: Arc::new(AtomicU64::new(0)),
            operations_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Snapshot of resource usage at a point in time
#[derive(Debug, Clone)]
pub struct ResourceUsageSnapshot {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Total execution time
    pub execution_time: Duration,
    /// Total LLM tokens consumed
    pub llm_tokens: u64,
    /// Number of operations performed
    pub operations_count: u64,
    /// Uptime since resource manager creation
    pub uptime: Duration,
}

impl ResourceUsageSnapshot {
    /// Get memory usage in megabytes
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes as f64 / 1024.0 / 1024.0
    }

    /// Get CPU usage as percentage
    pub fn cpu_percentage(&self) -> f64 {
        self.cpu_usage * 100.0
    }

    /// Get operations per second
    pub fn operations_per_second(&self) -> f64 {
        if self.uptime.as_secs_f64() > 0.0 {
            self.operations_count as f64 / self.uptime.as_secs_f64()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_limits() -> ResourceLimits {
        ResourceLimits {
            max_memory: "100MB".to_string(),
            max_cpu: "50%".to_string(),
            timeout: "5m".to_string(),
        }
    }

    #[test]
    fn test_resource_manager_creation() {
        let limits = create_test_limits();
        let manager = ResourceManager::new(limits).unwrap();
        
        assert_eq!(manager.limits.max_memory_bytes, 100 * 1024 * 1024);
        assert_eq!(manager.limits.max_cpu_usage, 0.5);
        assert_eq!(manager.limits.max_execution_time, Duration::from_secs(300));
    }

    #[test]
    fn test_memory_parsing() {
        assert_eq!(ResourceManager::parse_memory_string("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(ResourceManager::parse_memory_string("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(ResourceManager::parse_memory_string("512KB").unwrap(), 512 * 1024);
        assert_eq!(ResourceManager::parse_memory_string("1024").unwrap(), 1024);
    }

    #[test]
    fn test_cpu_parsing() {
        assert_eq!(ResourceManager::parse_cpu_string("50%").unwrap(), 0.5);
        assert_eq!(ResourceManager::parse_cpu_string("0.75").unwrap(), 0.75);
        assert_eq!(ResourceManager::parse_cpu_string("90").unwrap(), 0.9);
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(ResourceManager::parse_duration_string("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(ResourceManager::parse_duration_string("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(ResourceManager::parse_duration_string("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(ResourceManager::parse_duration_string("1000").unwrap(), Duration::from_secs(1000));
    }

    #[test]
    fn test_resource_usage_tracking() {
        let limits = create_test_limits();
        let mut manager = ResourceManager::new(limits).unwrap();
        
        // Record some usage
        assert!(manager.record_usage(100, Duration::from_secs(1)).is_ok());
        
        let usage = manager.get_usage();
        assert_eq!(usage.llm_tokens, 100);
        assert_eq!(usage.operations_count, 1);
        assert!(usage.execution_time >= Duration::from_secs(1));
    }

    #[test]
    fn test_memory_limit_enforcement() {
        let limits = create_test_limits();
        let mut manager = ResourceManager::new(limits).unwrap();
        
        // Set memory usage below limit
        assert!(manager.update_memory_usage(50 * 1024 * 1024).is_ok());
        
        // Set memory usage above limit
        assert!(manager.update_memory_usage(150 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_would_exceed_checks() {
        let limits = create_test_limits();
        let manager = ResourceManager::new(limits).unwrap();
        
        // Check memory
        assert!(!manager.would_exceed_memory(50 * 1024 * 1024));
        assert!(manager.would_exceed_memory(150 * 1024 * 1024));
        
        // Check timeout
        assert!(!manager.would_exceed_timeout(Duration::from_secs(100)));
        assert!(manager.would_exceed_timeout(Duration::from_secs(400)));
    }

    #[test]
    fn test_resource_usage_snapshot() {
        let limits = create_test_limits();
        let mut manager = ResourceManager::new(limits).unwrap();
        
        manager.update_memory_usage(50 * 1024 * 1024).unwrap();
        manager.record_usage(200, Duration::from_secs(2)).unwrap();
        
        let snapshot = manager.get_usage();
        assert_eq!(snapshot.memory_mb(), 50.0);
        assert_eq!(snapshot.llm_tokens, 200);
    }
}