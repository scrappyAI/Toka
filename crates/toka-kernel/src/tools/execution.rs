//! Execution monitoring and control for tool operations
//!
//! Provides real-time monitoring of tool execution including timeout handling,
//! resource usage tracking, and execution statistics collection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, oneshot};
use tokio::time::timeout;
use anyhow::Result;

use crate::tools::{
    ExecutionContext, KernelError, ExecutionStats, 
    ResourceLimits, SecurityLevel, capabilities::CapabilitySet
};

/// Execution monitor for tracking tool operations
#[derive(Clone)]
pub struct ExecutionMonitor {
    /// Active executions being monitored
    active_executions: Arc<RwLock<HashMap<String, ExecutionTracker>>>,
    /// Execution statistics
    statistics: Arc<RwLock<ExecutionStatistics>>,
}

/// Individual execution tracker
#[derive(Debug)]
struct ExecutionTracker {
    context: ExecutionContext,
    start_time: Instant,
    cancellation_sender: Option<oneshot::Sender<()>>,
    resource_snapshots: Vec<ResourceSnapshot>,
}

/// Resource usage snapshot during execution
#[derive(Debug, Clone)]
struct ResourceSnapshot {
    timestamp: Instant,
    memory_usage_mb: u64,
    cpu_usage_percent: f32,
    active_file_handles: u32,
    active_network_connections: u32,
}

/// Internal execution statistics tracking
#[derive(Debug, Clone)]
struct ExecutionStatistics {
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    timeout_executions: u64,
    security_violations: u32,
    resource_violations: u32,
    execution_durations: Vec<Duration>,
}

impl ExecutionMonitor {
    /// Create new execution monitor
    pub async fn new() -> Result<Self> {
        Ok(Self {
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(ExecutionStatistics::default())),
        })
    }
    
    /// Register a new execution for monitoring
    pub async fn register_execution(&self, context: &ExecutionContext) {
        let tracker = ExecutionTracker {
            context: context.clone(),
            start_time: Instant::now(),
            cancellation_sender: None,
            resource_snapshots: Vec::new(),
        };
        
        let mut executions = self.active_executions.write().await;
        executions.insert(context.tool_id.clone(), tracker);
        
        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_executions += 1;
    }
    
    /// Monitor execution with timeout and resource tracking
    pub async fn monitor_execution<F, T>(
        &self,
        context: &ExecutionContext,
        operation: F,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
    {
        let tool_id = context.tool_id.clone();
        let max_duration = context.resource_limits.max_execution_time;
        
        // Set up cancellation channel
        let (cancel_tx, cancel_rx) = oneshot::channel();
        
        // Update tracker with cancellation sender
        {
            let mut executions = self.active_executions.write().await;
            if let Some(tracker) = executions.get_mut(&tool_id) {
                tracker.cancellation_sender = Some(cancel_tx);
            }
        }
        
        // Start resource monitoring task
        let monitor_handle = self.start_resource_monitoring(&tool_id).await;
        
        // Execute with timeout
        let result = timeout(max_duration, async {
            tokio::select! {
                result = operation => result,
                _ = cancel_rx => Err(anyhow::anyhow!("Execution cancelled")),
            }
        }).await;
        
        // Stop resource monitoring
        monitor_handle.abort();
        
        // Process result and update statistics
        match result {
            Ok(Ok(value)) => {
                self.record_successful_execution(&tool_id).await;
                Ok(value)
            },
            Ok(Err(error)) => {
                self.record_failed_execution(&tool_id, error.to_string()).await;
                Err(KernelError::SecurityViolation { 
                    reason: error.to_string() 
                }.into())
            },
            Err(_timeout) => {
                self.record_timeout_execution(&tool_id).await;
                Err(KernelError::ExecutionTimeout { 
                    duration: max_duration 
                }.into())
            }
        }
    }
    
    /// Start monitoring resource usage for an execution
    async fn start_resource_monitoring(&self, tool_id: &str) -> tokio::task::JoinHandle<()> {
        let tool_id = tool_id.to_string();
        let executions = Arc::clone(&self.active_executions);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                // Check if execution is still active
                let mut executions_guard = executions.write().await;
                if let Some(tracker) = executions_guard.get_mut(&tool_id) {
                    // Take resource snapshot
                    let snapshot = ResourceSnapshot {
                        timestamp: Instant::now(),
                        memory_usage_mb: Self::get_current_memory_usage().await,
                        cpu_usage_percent: Self::get_current_cpu_usage().await,
                        active_file_handles: Self::get_current_file_handles().await,
                        active_network_connections: Self::get_current_network_connections().await,
                    };
                    
                    tracker.resource_snapshots.push(snapshot);
                    
                    // Limit snapshot history
                    if tracker.resource_snapshots.len() > 300 { // 5 minutes at 1s intervals
                        tracker.resource_snapshots.drain(0..50);
                    }
                } else {
                    // Execution no longer active, stop monitoring
                    break;
                }
                drop(executions_guard);
            }
        })
    }
    
    /// Cancel an active execution
    pub async fn cancel_execution(&self, tool_id: &str) -> Result<()> {
        let mut executions = self.active_executions.write().await;
        
        if let Some(tracker) = executions.get_mut(tool_id) {
            if let Some(sender) = tracker.cancellation_sender.take() {
                let _ = sender.send(());
                return Ok(());
            }
        }
        
        Err(anyhow::anyhow!("Execution not found or already completed"))
    }
    
    /// Get current active executions
    pub async fn get_active_executions(&self) -> Vec<ExecutionContext> {
        let executions = self.active_executions.read().await;
        executions.values()
            .map(|tracker| tracker.context.clone())
            .collect()
    }
    
    /// Get execution statistics
    pub async fn get_statistics(&self) -> ExecutionStats {
        let stats = self.statistics.read().await;
        
        let average_execution_time = if !stats.execution_durations.is_empty() {
            let total_duration: Duration = stats.execution_durations.iter().sum();
            total_duration / stats.execution_durations.len() as u32
        } else {
            Duration::from_secs(0)
        };
        
        ExecutionStats {
            total_executions: stats.total_executions,
            successful_executions: stats.successful_executions,
            failed_executions: stats.failed_executions,
            average_execution_time,
            resource_violations: stats.resource_violations,
            security_violations: stats.security_violations,
        }
    }
    
    /// Get resource usage history for a tool
    pub async fn get_resource_history(&self, tool_id: &str) -> Vec<ResourceSnapshot> {
        let executions = self.active_executions.read().await;
        
        if let Some(tracker) = executions.get(tool_id) {
            tracker.resource_snapshots.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Record successful execution completion
    async fn record_successful_execution(&self, tool_id: &str) {
        let duration = self.complete_execution(tool_id).await;
        
        let mut stats = self.statistics.write().await;
        stats.successful_executions += 1;
        
        if let Some(duration) = duration {
            stats.execution_durations.push(duration);
            
            // Keep only recent durations for average calculation
            if stats.execution_durations.len() > 1000 {
                stats.execution_durations.drain(0..100);
            }
        }
    }
    
    /// Record failed execution
    async fn record_failed_execution(&self, tool_id: &str, _error: String) {
        let _duration = self.complete_execution(tool_id).await;
        
        let mut stats = self.statistics.write().await;
        stats.failed_executions += 1;
    }
    
    /// Record timeout execution
    async fn record_timeout_execution(&self, tool_id: &str) {
        let _duration = self.complete_execution(tool_id).await;
        
        let mut stats = self.statistics.write().await;
        stats.failed_executions += 1;
        stats.timeout_executions += 1;
    }
    
    /// Complete execution tracking and return duration
    async fn complete_execution(&self, tool_id: &str) -> Option<Duration> {
        let mut executions = self.active_executions.write().await;
        
        if let Some(tracker) = executions.remove(tool_id) {
            Some(tracker.start_time.elapsed())
        } else {
            None
        }
    }
    
    /// Record security violation
    pub async fn record_security_violation(&self) {
        let mut stats = self.statistics.write().await;
        stats.security_violations += 1;
    }
    
    /// Record resource violation
    pub async fn record_resource_violation(&self) {
        let mut stats = self.statistics.write().await;
        stats.resource_violations += 1;
    }
    
    /// Cleanup completed executions (called periodically)
    pub async fn cleanup_completed_executions(&self) {
        let mut executions = self.active_executions.write().await;
        let now = Instant::now();
        
        // Remove executions that have been running for more than 1 hour
        executions.retain(|_, tracker| {
            now.duration_since(tracker.start_time) < Duration::from_secs(3600)
        });
    }
    
    // System resource monitoring helpers
    async fn get_current_memory_usage() -> u64 {
        // Simplified memory usage - in production use proper system monitoring
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                return kb / 1024; // Convert to MB
                            }
                        }
                    }
                }
            }
        }
        0
    }
    
    async fn get_current_cpu_usage() -> f32 {
        // Simplified CPU usage - in production implement proper CPU monitoring
        0.0
    }
    
    async fn get_current_file_handles() -> u32 {
        // Simplified file handle count - in production use proper monitoring
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(entries) = fs::read_dir("/proc/self/fd") {
                return entries.count() as u32;
            }
        }
        0
    }
    
    async fn get_current_network_connections() -> u32 {
        // Simplified network connection count - in production use proper monitoring
        0
    }
}

impl Default for ExecutionStatistics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_executions: 0,
            security_violations: 0,
            resource_violations: 0,
            execution_durations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{SecurityLevel, ResourceLimits, capabilities::CapabilitySet};

    #[tokio::test]
    async fn test_execution_monitor_creation() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        let stats = monitor.get_statistics().await;
        
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 0);
    }
    
    #[tokio::test]
    async fn test_execution_registration() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        
        let context = ExecutionContext {
            tool_id: "test_tool".to_string(),
            session_id: "test_session".to_string(),
            capabilities: CapabilitySet::new(),
            resource_limits: ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: Instant::now(),
        };
        
        monitor.register_execution(&context).await;
        
        let active = monitor.get_active_executions().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].tool_id, "test_tool");
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_executions, 1);
    }
    
    #[tokio::test]
    async fn test_successful_execution_monitoring() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        
        let context = ExecutionContext {
            tool_id: "test_tool".to_string(),
            session_id: "test_session".to_string(),
            capabilities: CapabilitySet::new(),
            resource_limits: ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: Instant::now(),
        };
        
        monitor.register_execution(&context).await;
        
        // Simulate successful operation
        let result = monitor.monitor_execution(&context, async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok("success")
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.failed_executions, 0);
        
        // Should no longer be in active executions
        let active = monitor.get_active_executions().await;
        assert_eq!(active.len(), 0);
    }
    
    #[tokio::test]
    async fn test_failed_execution_monitoring() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        
        let context = ExecutionContext {
            tool_id: "test_tool".to_string(),
            session_id: "test_session".to_string(),
            capabilities: CapabilitySet::new(),
            resource_limits: ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: Instant::now(),
        };
        
        monitor.register_execution(&context).await;
        
        // Simulate failed operation
        let result = monitor.monitor_execution(&context, async {
            Err(anyhow::anyhow!("operation failed"))
        }).await;
        
        assert!(result.is_err());
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 1);
    }
    
    #[tokio::test]
    async fn test_execution_timeout() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        
        let mut context = ExecutionContext {
            tool_id: "test_tool".to_string(),
            session_id: "test_session".to_string(),
            capabilities: CapabilitySet::new(),
            resource_limits: ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: Instant::now(),
        };
        
        // Set very short timeout
        context.resource_limits.max_execution_time = Duration::from_millis(50);
        
        monitor.register_execution(&context).await;
        
        // Simulate long-running operation
        let result = monitor.monitor_execution(&context, async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok("should timeout")
        }).await;
        
        assert!(result.is_err());
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.failed_executions, 1);
    }
    
    #[tokio::test]
    async fn test_execution_cancellation() {
        let monitor = ExecutionMonitor::new().await.unwrap();
        
        let context = ExecutionContext {
            tool_id: "test_tool".to_string(),
            session_id: "test_session".to_string(),
            capabilities: CapabilitySet::new(),
            resource_limits: ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: Instant::now(),
        };
        
        monitor.register_execution(&context).await;
        
        // Start long-running operation in background
        let monitor_clone = monitor.clone();
        let context_clone = context.clone();
        let execution_handle = tokio::spawn(async move {
            monitor_clone.monitor_execution(&context_clone, async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok("long operation")
            }).await
        });
        
        // Give it time to start
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Cancel the execution
        let cancel_result = monitor.cancel_execution("test_tool").await;
        assert!(cancel_result.is_ok());
        
        // Execution should be cancelled
        let result = execution_handle.await.unwrap();
        assert!(result.is_err());
    }
} 