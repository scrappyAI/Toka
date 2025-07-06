//! CPU and memory profiling utilities
//!
//! This module provides CPU and memory profiling capabilities for
//! performance analysis and optimization.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enable CPU profiling
    pub cpu_profiling_enabled: bool,
    /// Enable memory profiling
    pub memory_profiling_enabled: bool,
    /// Profiling sample rate (samples per second)
    pub sample_rate_hz: u32,
    /// Maximum profiling duration
    pub max_duration_seconds: u64,
    /// Profile output directory
    pub output_directory: String,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            cpu_profiling_enabled: false,
            memory_profiling_enabled: false,
            sample_rate_hz: 100,
            max_duration_seconds: 300, // 5 minutes
            output_directory: "./profiles".to_string(),
        }
    }
}

/// CPU profiler
#[derive(Debug)]
pub struct CpuProfiler {
    /// Component identifier
    component_id: String,
    /// Profiling configuration
    config: ProfilingConfig,
    /// Profiling session
    session: Option<ProfilingSession>,
}

/// Memory profiler
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Component identifier
    component_id: String,
    /// Profiling configuration
    config: ProfilingConfig,
    /// Memory snapshots
    snapshots: Vec<MemorySnapshot>,
}

/// Profiling session
#[derive(Debug)]
pub struct ProfilingSession {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Option<Duration>,
    /// Profile type
    pub profile_type: ProfileType,
    /// Output file path
    pub output_path: String,
}

/// Profile types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileType {
    /// CPU profile
    Cpu,
    /// Memory profile
    Memory,
    /// Combined profile
    Combined,
}

/// Memory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Total memory usage in bytes
    pub total_memory_bytes: u64,
    /// Memory usage by category
    pub memory_by_category: HashMap<String, u64>,
    /// Memory allocations
    pub allocations: Vec<MemoryAllocation>,
}

/// Memory allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    /// Allocation size in bytes
    pub size_bytes: u64,
    /// Allocation type
    pub allocation_type: String,
    /// Stack trace (simplified)
    pub stack_trace: Vec<String>,
    /// Allocation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Profiling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingResult {
    /// Session ID
    pub session_id: String,
    /// Profile type
    pub profile_type: ProfileType,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// Duration
    pub duration: Duration,
    /// CPU usage statistics
    pub cpu_stats: Option<CpuStats>,
    /// Memory usage statistics
    pub memory_stats: Option<MemoryStats>,
    /// Output file path
    pub output_path: String,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    /// Average CPU usage percentage
    pub avg_cpu_percent: f64,
    /// Peak CPU usage percentage
    pub peak_cpu_percent: f64,
    /// CPU time by function
    pub cpu_time_by_function: HashMap<String, Duration>,
    /// Number of samples collected
    pub samples_collected: u64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Memory allocations count
    pub allocations_count: u64,
    /// Memory deallocations count
    pub deallocations_count: u64,
    /// Memory leaks detected
    pub potential_leaks: Vec<String>,
}

impl CpuProfiler {
    /// Create a new CPU profiler
    pub fn new(component_id: &str, config: ProfilingConfig) -> Self {
        Self {
            component_id: component_id.to_string(),
            config,
            session: None,
        }
    }
    
    /// Start CPU profiling
    pub fn start_profiling(&mut self, session_name: &str) -> Result<String> {
        if !self.config.cpu_profiling_enabled {
            return Err(anyhow::anyhow!("CPU profiling is not enabled"));
        }
        
        if self.session.is_some() {
            return Err(anyhow::anyhow!("CPU profiling session already active"));
        }
        
        let session_id = uuid::Uuid::new_v4().to_string();
        let output_path = format!("{}/cpu_profile_{}_{}.prof", 
            self.config.output_directory, 
            self.component_id, 
            session_id
        );
        
        let session = ProfilingSession {
            id: session_id.clone(),
            name: session_name.to_string(),
            start_time: Instant::now(),
            duration: None,
            profile_type: ProfileType::Cpu,
            output_path,
        };
        
        self.session = Some(session);
        
        tracing::info!(
            component = %self.component_id,
            session_id = %session_id,
            session_name = %session_name,
            "Started CPU profiling session"
        );
        
        Ok(session_id)
    }
    
    /// Stop CPU profiling
    pub fn stop_profiling(&mut self) -> Result<ProfilingResult> {
        let session = self.session.take()
            .ok_or_else(|| anyhow::anyhow!("No active CPU profiling session"))?;
        
        let duration = session.start_time.elapsed();
        
        // Simulate CPU profiling data collection
        let cpu_stats = CpuStats {
            avg_cpu_percent: 25.0 + fastrand::f64() * 20.0,
            peak_cpu_percent: 45.0 + fastrand::f64() * 30.0,
            cpu_time_by_function: self.simulate_cpu_time_by_function(),
            samples_collected: (duration.as_secs() * self.config.sample_rate_hz as u64),
        };
        
        let result = ProfilingResult {
            session_id: session.id,
            profile_type: ProfileType::Cpu,
            start_time: chrono::Utc::now() - chrono::Duration::from_std(duration)?,
            duration,
            cpu_stats: Some(cpu_stats),
            memory_stats: None,
            output_path: session.output_path,
        };
        
        tracing::info!(
            component = %self.component_id,
            session_id = %result.session_id,
            duration = ?duration,
            avg_cpu_percent = %result.cpu_stats.as_ref().unwrap().avg_cpu_percent,
            "Stopped CPU profiling session"
        );
        
        Ok(result)
    }
    
    /// Check if profiling is active
    pub fn is_profiling(&self) -> bool {
        self.session.is_some()
    }
    
    /// Simulate CPU time by function
    fn simulate_cpu_time_by_function(&self) -> HashMap<String, Duration> {
        let mut cpu_time = HashMap::new();
        
        cpu_time.insert("main".to_string(), Duration::from_millis(100));
        cpu_time.insert("process_request".to_string(), Duration::from_millis(50));
        cpu_time.insert("serialize_data".to_string(), Duration::from_millis(30));
        cpu_time.insert("network_io".to_string(), Duration::from_millis(25));
        cpu_time.insert("database_query".to_string(), Duration::from_millis(20));
        
        cpu_time
    }
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new(component_id: &str, config: ProfilingConfig) -> Self {
        Self {
            component_id: component_id.to_string(),
            config,
            snapshots: Vec::new(),
        }
    }
    
    /// Take a memory snapshot
    pub fn take_snapshot(&mut self) -> Result<()> {
        if !self.config.memory_profiling_enabled {
            return Err(anyhow::anyhow!("Memory profiling is not enabled"));
        }
        
        let snapshot = MemorySnapshot {
            timestamp: Utc::now(),
            total_memory_bytes: self.get_total_memory_usage(),
            memory_by_category: self.get_memory_by_category(),
            allocations: self.get_recent_allocations(),
        };
        
        self.snapshots.push(snapshot);
        
        // Limit snapshots to prevent memory growth
        if self.snapshots.len() > 1000 {
            self.snapshots.drain(0..self.snapshots.len() - 1000);
        }
        
        tracing::debug!(
            component = %self.component_id,
            total_memory_bytes = %self.snapshots.last().unwrap().total_memory_bytes,
            "Memory snapshot taken"
        );
        
        Ok(())
    }
    
    /// Analyze memory usage
    pub fn analyze_memory_usage(&self) -> Result<MemoryStats> {
        if self.snapshots.is_empty() {
            return Err(anyhow::anyhow!("No memory snapshots available"));
        }
        
        let total_memory: Vec<u64> = self.snapshots.iter()
            .map(|s| s.total_memory_bytes)
            .collect();
        
        let avg_memory_bytes = total_memory.iter().sum::<u64>() / total_memory.len() as u64;
        let peak_memory_bytes = total_memory.iter().copied().max().unwrap_or(0);
        
        let total_allocations: u64 = self.snapshots.iter()
            .map(|s| s.allocations.len() as u64)
            .sum();
        
        let potential_leaks = self.detect_potential_leaks();
        
        Ok(MemoryStats {
            avg_memory_bytes,
            peak_memory_bytes,
            allocations_count: total_allocations,
            deallocations_count: total_allocations - potential_leaks.len() as u64,
            potential_leaks,
        })
    }
    
    /// Get memory snapshots
    pub fn get_snapshots(&self) -> &[MemorySnapshot] {
        &self.snapshots
    }
    
    /// Clear snapshots
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }
    
    /// Simulate total memory usage
    fn get_total_memory_usage(&self) -> u64 {
        50 * 1024 * 1024 + fastrand::u64(0..20 * 1024 * 1024) // 50-70 MB
    }
    
    /// Simulate memory usage by category
    fn get_memory_by_category(&self) -> HashMap<String, u64> {
        let mut memory = HashMap::new();
        
        memory.insert("heap".to_string(), 30 * 1024 * 1024);
        memory.insert("stack".to_string(), 8 * 1024 * 1024);
        memory.insert("globals".to_string(), 5 * 1024 * 1024);
        memory.insert("buffers".to_string(), 10 * 1024 * 1024);
        
        memory
    }
    
    /// Simulate recent allocations
    fn get_recent_allocations(&self) -> Vec<MemoryAllocation> {
        let mut allocations = Vec::new();
        
        for i in 0..5 {
            allocations.push(MemoryAllocation {
                size_bytes: 1024 + fastrand::u64(0..8192),
                allocation_type: format!("type_{}", i % 3),
                stack_trace: vec![
                    "main".to_string(),
                    "allocate_buffer".to_string(),
                    "malloc".to_string(),
                ],
                timestamp: Utc::now(),
            });
        }
        
        allocations
    }
    
    /// Detect potential memory leaks
    fn detect_potential_leaks(&self) -> Vec<String> {
        // Simulate leak detection
        if self.snapshots.len() >= 10 {
            let recent_growth = self.snapshots.last().unwrap().total_memory_bytes 
                - self.snapshots[self.snapshots.len() - 10].total_memory_bytes;
            
            if recent_growth > 10 * 1024 * 1024 { // 10MB growth
                vec!["Potential memory leak in buffer allocation".to_string()]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

/// Profiling error types
#[derive(Debug, thiserror::Error)]
pub enum ProfilingError {
    /// Profiling not enabled
    #[error("Profiling is not enabled")]
    NotEnabled,
    
    /// Session already active
    #[error("Profiling session already active")]
    SessionActive,
    
    /// No active session
    #[error("No active profiling session")]
    NoActiveSession,
    
    /// Output error
    #[error("Profiling output error: {0}")]
    Output(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_profiler_creation() {
        let config = ProfilingConfig {
            cpu_profiling_enabled: true,
            ..Default::default()
        };
        let profiler = CpuProfiler::new("test-component", config);
        assert_eq!(profiler.component_id, "test-component");
        assert!(!profiler.is_profiling());
    }
    
    #[test]
    fn test_cpu_profiling_lifecycle() {
        let config = ProfilingConfig {
            cpu_profiling_enabled: true,
            ..Default::default()
        };
        let mut profiler = CpuProfiler::new("test-component", config);
        
        // Start profiling
        let session_id = profiler.start_profiling("test_session").unwrap();
        assert!(!session_id.is_empty());
        assert!(profiler.is_profiling());
        
        // Stop profiling
        let result = profiler.stop_profiling().unwrap();
        assert_eq!(result.session_id, session_id);
        assert!(matches!(result.profile_type, ProfileType::Cpu));
        assert!(!profiler.is_profiling());
    }
    
    #[test]
    fn test_memory_profiler_snapshots() {
        let config = ProfilingConfig {
            memory_profiling_enabled: true,
            ..Default::default()
        };
        let mut profiler = MemoryProfiler::new("test-component", config);
        
        // Take snapshots
        profiler.take_snapshot().unwrap();
        profiler.take_snapshot().unwrap();
        
        assert_eq!(profiler.get_snapshots().len(), 2);
        
        // Analyze memory usage
        let stats = profiler.analyze_memory_usage().unwrap();
        assert!(stats.avg_memory_bytes > 0);
        assert!(stats.peak_memory_bytes > 0);
    }
}