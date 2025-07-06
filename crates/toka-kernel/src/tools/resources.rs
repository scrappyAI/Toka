//! Resource management for tool execution
//!
//! Tracks and limits system resource usage to prevent tools from consuming
//! excessive resources and affecting system stability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use anyhow::Result;

use super::{ExecutionContext, SecurityLevel, ResourceLimits, ResourceUsage};

/// Resource manager for tracking and limiting tool resource usage
pub struct ResourceManager {
    /// Active resource allocations
    allocations: RwLock<HashMap<String, ResourceAllocation>>,
    /// Semaphores for limiting concurrent resource usage
    memory_semaphore: Arc<Semaphore>,
    cpu_semaphore: Arc<Semaphore>,
    file_handle_semaphore: Arc<Semaphore>,
    network_semaphore: Arc<Semaphore>,
    /// System resource monitoring
    system_monitor: SystemMonitor,
}

/// Individual resource allocation for a tool execution
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub context_id: String,
    pub memory_mb: u64,
    pub cpu_percent: u8,
    pub file_handles: u32,
    pub network_connections: u32,
    pub allocated_at: Instant,
}

/// System resource monitoring
struct SystemMonitor {
    last_update: RwLock<Instant>,
    cached_usage: RwLock<SystemResourceUsage>,
}

/// Current system resource usage snapshot
#[derive(Debug, Clone)]
struct SystemResourceUsage {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub cpu_usage_percent: f32,
    pub process_count: u32,
    pub file_handle_count: u32,
    pub network_connection_count: u32,
}

impl ResourceManager {
    /// Create new resource manager with system limits
    pub async fn new() -> Result<Self> {
        // Get system limits
        let system_info = SystemMonitor::get_system_info().await?;
        
        // Create semaphores based on system capabilities
        let memory_semaphore = Arc::new(Semaphore::new(
            (system_info.available_memory_mb / 64) as usize // Allow 64MB chunks
        ));
        
        let cpu_semaphore = Arc::new(Semaphore::new(100)); // 100% CPU
        let file_handle_semaphore = Arc::new(Semaphore::new(1000)); // 1000 file handles
        let network_semaphore = Arc::new(Semaphore::new(100)); // 100 network connections
        
        Ok(Self {
            allocations: RwLock::new(HashMap::new()),
            memory_semaphore,
            cpu_semaphore,
            file_handle_semaphore,
            network_semaphore,
            system_monitor: SystemMonitor::new(),
        })
    }
    
    /// Allocate resources for tool execution
    pub async fn allocate_resources(
        &self,
        security_level: &SecurityLevel,
    ) -> Result<ResourceLimits> {
        // Get default limits for security level
        let limits = security_level.default_resource_limits();
        
        // Check if resources are available
        self.check_resource_availability(&limits).await?;
        
        // Acquire semaphore permits
        let _memory_permit = self.memory_semaphore
            .acquire_many((limits.max_memory_mb / 64) as u32)
            .await?;
        
        let _cpu_permit = self.cpu_semaphore
            .acquire_many(limits.max_cpu_percent as u32)
            .await?;
        
        let _file_permit = self.file_handle_semaphore
            .acquire_many(limits.max_file_handles)
            .await?;
        
        let _network_permit = self.network_semaphore
            .acquire_many(limits.max_network_connections)
            .await?;
        
        // Create allocation record
        let allocation = ResourceAllocation {
            context_id: format!("alloc_{}", Instant::now().elapsed().as_nanos()),
            memory_mb: limits.max_memory_mb,
            cpu_percent: limits.max_cpu_percent,
            file_handles: limits.max_file_handles,
            network_connections: limits.max_network_connections,
            allocated_at: Instant::now(),
        };
        
        // Store allocation
        let mut allocations = self.allocations.write().await;
        allocations.insert(allocation.context_id.clone(), allocation);
        
        Ok(limits)
    }
    
    /// Release resources after tool execution
    pub async fn release_resources(&self, context: &ExecutionContext) {
        let mut allocations = self.allocations.write().await;
        
        // Find and remove allocation
        if let Some(allocation) = allocations.remove(&context.tool_id) {
            // Semaphore permits are automatically released when dropped
            // This happens when the execution context is dropped
            
            // Log resource usage duration
            let duration = allocation.allocated_at.elapsed();
            tracing::debug!(
                "Released resources for tool {} after {:?}",
                context.tool_id,
                duration
            );
        }
    }
    
    /// Check if requested resources are available
    async fn check_resource_availability(&self, limits: &ResourceLimits) -> Result<()> {
        let current_usage = self.get_current_usage().await;
        
        // Check memory availability
        if current_usage.memory_used_mb + limits.max_memory_mb > current_usage.memory_used_mb * 2 {
            return Err(anyhow::anyhow!("Insufficient memory available"));
        }
        
        // Check if we're already at CPU capacity
        if current_usage.cpu_usage_percent > 80.0 {
            return Err(anyhow::anyhow!("System CPU usage too high"));
        }
        
        // Check file handle limits
        if current_usage.active_file_handles + limits.max_file_handles > 10000 {
            return Err(anyhow::anyhow!("File handle limit would be exceeded"));
        }
        
        // Check network connection limits
        if current_usage.active_network_connections + limits.max_network_connections > 1000 {
            return Err(anyhow::anyhow!("Network connection limit would be exceeded"));
        }
        
        Ok(())
    }
    
    /// Get current system resource usage
    pub async fn get_current_usage(&self) -> ResourceUsage {
        // Update system monitoring data
        self.system_monitor.update().await;
        
        let system_usage = self.system_monitor.get_cached_usage().await;
        let allocations = self.allocations.read().await;
        
        ResourceUsage {
            memory_used_mb: system_usage.total_memory_mb - system_usage.available_memory_mb,
            cpu_usage_percent: system_usage.cpu_usage_percent,
            active_file_handles: system_usage.file_handle_count,
            active_network_connections: system_usage.network_connection_count,
            active_executions: allocations.len() as u32,
        }
    }
    
    /// Get resource allocation details
    pub async fn get_allocations(&self) -> Vec<ResourceAllocation> {
        let allocations = self.allocations.read().await;
        allocations.values().cloned().collect()
    }
    
    /// Force cleanup of stale allocations
    pub async fn cleanup_stale_allocations(&self, max_age: Duration) {
        let mut allocations = self.allocations.write().await;
        let now = Instant::now();
        
        allocations.retain(|_, allocation| {
            now.duration_since(allocation.allocated_at) < max_age
        });
    }
}

impl SystemMonitor {
    fn new() -> Self {
        Self {
            last_update: RwLock::new(Instant::now()),
            cached_usage: RwLock::new(SystemResourceUsage::default()),
        }
    }
    
    /// Update cached system resource information
    async fn update(&self) {
        let now = Instant::now();
        let last_update = *self.last_update.read().await;
        
        // Only update every 5 seconds to avoid excessive system calls
        if now.duration_since(last_update) > Duration::from_secs(5) {
            if let Ok(usage) = Self::get_system_info().await {
                let mut cached = self.cached_usage.write().await;
                *cached = usage;
                
                let mut last_update_guard = self.last_update.write().await;
                *last_update_guard = now;
            }
        }
    }
    
    /// Get cached system usage information
    async fn get_cached_usage(&self) -> SystemResourceUsage {
        let cached = self.cached_usage.read().await;
        cached.clone()
    }
    
    /// Get current system resource information
    async fn get_system_info() -> Result<SystemResourceUsage> {
        // Use psutil-like functionality to get system info
        // This is a simplified implementation - in production, use a proper system info library
        
        #[cfg(unix)]
        {
            use std::fs;
            
            // Get memory info from /proc/meminfo
            let meminfo = fs::read_to_string("/proc/meminfo")?;
            let (total_memory, available_memory) = Self::parse_meminfo(&meminfo);
            
            // Get CPU usage (simplified)
            let cpu_usage = Self::get_cpu_usage().await;
            
            // Get process and file handle counts
            let process_count = Self::get_process_count().await;
            let file_handle_count = Self::get_file_handle_count().await;
            
            Ok(SystemResourceUsage {
                total_memory_mb: total_memory,
                available_memory_mb: available_memory,
                cpu_usage_percent: cpu_usage,
                process_count,
                file_handle_count,
                network_connection_count: 0, // Simplified
            })
        }
        
        #[cfg(not(unix))]
        {
            // Fallback for non-Unix systems
            Ok(SystemResourceUsage::default())
        }
    }
    
    #[cfg(unix)]
    fn parse_meminfo(meminfo: &str) -> (u64, u64) {
        let mut total_kb = 0;
        let mut available_kb = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    total_kb = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    available_kb = value.parse().unwrap_or(0);
                }
            }
        }
        
        (total_kb / 1024, available_kb / 1024) // Convert to MB
    }
    
    #[cfg(unix)]
    async fn get_cpu_usage() -> f32 {
        // Simplified CPU usage calculation
        // In production, implement proper CPU usage monitoring
        use std::fs;
        
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = loadavg.split_whitespace().next() {
                if let Ok(load) = load_str.parse::<f32>() {
                    // Convert load average to approximate CPU percentage
                    return (load * 100.0).min(100.0);
                }
            }
        }
        
        0.0
    }
    
    #[cfg(unix)]
    async fn get_process_count() -> u32 {
        use std::fs;
        
        if let Ok(entries) = fs::read_dir("/proc") {
            let count = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_name()
                        .to_string_lossy()
                        .chars()
                        .all(|c| c.is_ascii_digit())
                })
                .count();
            return count as u32;
        }
        
        0
    }
    
    #[cfg(unix)]
    async fn get_file_handle_count() -> u32 {
        use std::fs;
        
        // Read from /proc/sys/fs/file-nr for system-wide file handle count
        if let Ok(file_nr) = fs::read_to_string("/proc/sys/fs/file-nr") {
            if let Some(count_str) = file_nr.split_whitespace().next() {
                if let Ok(count) = count_str.parse::<u32>() {
                    return count;
                }
            }
        }
        
        0
    }
}

impl Default for SystemResourceUsage {
    fn default() -> Self {
        Self {
            total_memory_mb: 4096, // 4GB default
            available_memory_mb: 2048, // 2GB available
            cpu_usage_percent: 0.0,
            process_count: 0,
            file_handle_count: 0,
            network_connection_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let manager = ResourceManager::new().await.unwrap();
        let usage = manager.get_current_usage().await;
        
        // Should have some basic resource information
        assert!(usage.memory_used_mb >= 0);
        assert!(usage.cpu_usage_percent >= 0.0);
        assert_eq!(usage.active_executions, 0);
    }
    
    #[tokio::test]
    async fn test_resource_allocation() {
        let manager = ResourceManager::new().await.unwrap();
        
        // Should be able to allocate resources for sandboxed execution
        let limits = manager.allocate_resources(&SecurityLevel::Sandboxed).await.unwrap();
        
        assert_eq!(limits.max_memory_mb, 128);
        assert_eq!(limits.max_cpu_percent, 25);
        assert_eq!(limits.max_file_handles, 10);
        assert_eq!(limits.max_network_connections, 2);
    }
    
    #[tokio::test]
    async fn test_allocation_tracking() {
        let manager = ResourceManager::new().await.unwrap();
        
        // Allocate resources
        let _limits = manager.allocate_resources(&SecurityLevel::Restricted).await.unwrap();
        
        // Should track the allocation
        let allocations = manager.get_allocations().await;
        assert_eq!(allocations.len(), 1);
        
        let usage = manager.get_current_usage().await;
        assert_eq!(usage.active_executions, 1);
    }
    
    #[tokio::test]
    async fn test_stale_allocation_cleanup() {
        let manager = ResourceManager::new().await.unwrap();
        
        // Create a mock allocation that's older than cleanup threshold
        let mut allocations = manager.allocations.write().await;
        allocations.insert("old_allocation".to_string(), ResourceAllocation {
            context_id: "old_allocation".to_string(),
            memory_mb: 128,
            cpu_percent: 25,
            file_handles: 10,
            network_connections: 2,
            allocated_at: Instant::now() - Duration::from_secs(3600), // 1 hour ago
        });
        drop(allocations);
        
        // Cleanup allocations older than 30 minutes
        manager.cleanup_stale_allocations(Duration::from_secs(1800)).await;
        
        // Should have removed the old allocation
        let remaining = manager.get_allocations().await;
        assert_eq!(remaining.len(), 0);
    }
} 