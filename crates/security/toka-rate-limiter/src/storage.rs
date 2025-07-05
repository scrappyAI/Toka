//! Storage backends for rate limiting data

use crate::{RateLimitStorage, RateLimitUsage, RateLimitKey};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// In-memory storage backend for rate limiting
/// 
/// This implementation stores rate limiting data in memory using a HashMap.
/// It's suitable for single-instance deployments or testing, but doesn't
/// persist across restarts and doesn't support distributed rate limiting.
#[derive(Debug)]
pub struct MemoryRateLimitStorage {
    data: Arc<RwLock<HashMap<String, RateLimitUsage>>>,
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<DateTime<Utc>>>,
}

impl MemoryRateLimitStorage {
    /// Create a new memory storage instance
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::minutes(5),
            last_cleanup: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Create a new memory storage instance with custom cleanup interval
    pub fn with_cleanup_interval(cleanup_interval: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval,
            last_cleanup: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Get current storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        let data = self.data.read().await;
        let total_entries = data.len();
        let last_cleanup = *self.last_cleanup.read().await;
        
        // Calculate memory usage estimation
        let estimated_memory = total_entries * std::mem::size_of::<RateLimitUsage>();
        
        StorageStats {
            total_entries,
            estimated_memory_bytes: estimated_memory,
            last_cleanup,
        }
    }

    /// Check if cleanup should be performed
    async fn should_cleanup(&self) -> bool {
        let last_cleanup = *self.last_cleanup.read().await;
        let now = Utc::now();
        now - last_cleanup >= self.cleanup_interval
    }

    /// Perform cleanup if needed
    async fn maybe_cleanup(&self) -> Result<()> {
        if self.should_cleanup().await {
            self.cleanup_expired().await?;
        }
        Ok(())
    }

    /// Remove entries that are older than their typical window duration
    async fn remove_old_entries(&self, max_age: Duration) -> Result<usize> {
        let mut data = self.data.write().await;
        let now = Utc::now();
        let initial_count = data.len();
        
        data.retain(|_, usage| {
            now - usage.last_request <= max_age
        });
        
        let removed_count = initial_count - data.len();
        
        if removed_count > 0 {
            debug!("Removed {} old rate limit entries", removed_count);
        }
        
        Ok(removed_count)
    }
}

impl Default for MemoryRateLimitStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitStorage for MemoryRateLimitStorage {
    async fn get_usage(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>> {
        // Perform cleanup if needed
        self.maybe_cleanup().await?;
        
        let data = self.data.read().await;
        let storage_key = key.to_storage_key();
        Ok(data.get(&storage_key).cloned())
    }

    async fn update_usage(&self, key: &RateLimitKey, usage: &RateLimitUsage) -> Result<()> {
        let mut data = self.data.write().await;
        let storage_key = key.to_storage_key();
        data.insert(storage_key, usage.clone());
        Ok(())
    }

    async fn increment_usage(&self, key: &RateLimitKey, amount: u64) -> Result<RateLimitUsage> {
        let mut data = self.data.write().await;
        let storage_key = key.to_storage_key();
        
        let usage = data.entry(storage_key)
            .or_insert_with(|| RateLimitUsage::new(0, Utc::now()));
        
        usage.increment(amount);
        Ok(usage.clone())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        // Remove entries older than 1 hour (conservative cleanup)
        let removed_count = self.remove_old_entries(Duration::hours(1)).await?;
        
        // Update last cleanup time
        {
            let mut last_cleanup = self.last_cleanup.write().await;
            *last_cleanup = Utc::now();
        }
        
        debug!("Rate limit storage cleanup completed, removed {} entries", removed_count);
        Ok(removed_count)
    }

    async fn reset_usage(&self, key: &RateLimitKey) -> Result<()> {
        let mut data = self.data.write().await;
        let storage_key = key.to_storage_key();
        data.remove(&storage_key);
        debug!("Reset rate limit usage for key: {}", storage_key);
        Ok(())
    }
}

/// Statistics about storage performance and usage
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total number of stored entries
    pub total_entries: usize,
    /// Estimated memory usage in bytes
    pub estimated_memory_bytes: usize,
    /// Last time cleanup was performed
    pub last_cleanup: DateTime<Utc>,
}

/// Distributed storage backend for rate limiting (placeholder)
/// 
/// This would be implemented for production distributed deployments
/// using Redis, PostgreSQL, or other shared storage systems.
#[allow(dead_code)]
pub struct DistributedRateLimitStorage {
    // Implementation would depend on the backend
    // Examples: Redis client, database connection pool, etc.
}

impl DistributedRateLimitStorage {
    /// Create a new distributed storage instance
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

// Note: DistributedRateLimitStorage implementation would require
// specific backend dependencies (redis, sqlx, etc.) so it's left
// as a placeholder for now.

/// Storage backend that combines multiple storage implementations
/// for layered rate limiting (e.g., memory + distributed)
pub struct LayeredRateLimitStorage {
    primary: Arc<dyn RateLimitStorage>,
    fallback: Option<Arc<dyn RateLimitStorage>>,
}

impl LayeredRateLimitStorage {
    /// Create a new layered storage with primary and optional fallback
    pub fn new(
        primary: Arc<dyn RateLimitStorage>,
        fallback: Option<Arc<dyn RateLimitStorage>>,
    ) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl RateLimitStorage for LayeredRateLimitStorage {
    async fn get_usage(&self, key: &RateLimitKey) -> Result<Option<RateLimitUsage>> {
        // Try primary storage first
        match self.primary.get_usage(key).await {
            Ok(result) => Ok(result),
            Err(_) => {
                // Fall back to secondary storage if primary fails
                if let Some(ref fallback) = self.fallback {
                    warn!("Primary storage failed, using fallback for get_usage");
                    fallback.get_usage(key).await
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn update_usage(&self, key: &RateLimitKey, usage: &RateLimitUsage) -> Result<()> {
        // Update both primary and fallback if available
        let primary_result = self.primary.update_usage(key, usage).await;
        
        if let Some(ref fallback) = self.fallback {
            if let Err(e) = fallback.update_usage(key, usage).await {
                warn!("Fallback storage update failed: {}", e);
            }
        }
        
        primary_result
    }

    async fn increment_usage(&self, key: &RateLimitKey, amount: u64) -> Result<RateLimitUsage> {
        // Use primary storage for increment
        match self.primary.increment_usage(key, amount).await {
            Ok(result) => {
                // Update fallback asynchronously if available
                if let Some(ref fallback) = self.fallback {
                    let key_clone = key.clone();
                    let result_clone = result.clone();
                    let fallback_clone = Arc::clone(fallback);
                    
                    tokio::spawn(async move {
                        if let Err(e) = fallback_clone.update_usage(&key_clone, &result_clone).await {
                            warn!("Fallback storage update failed: {}", e);
                        }
                    });
                }
                Ok(result)
            }
            Err(e) => {
                // Try fallback if primary fails
                if let Some(ref fallback) = self.fallback {
                    warn!("Primary storage failed, using fallback for increment_usage");
                    fallback.increment_usage(key, amount).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut total_cleaned = 0;
        
        // Cleanup primary storage
        match self.primary.cleanup_expired().await {
            Ok(count) => total_cleaned += count,
            Err(e) => warn!("Primary storage cleanup failed: {}", e),
        }
        
        // Cleanup fallback storage
        if let Some(ref fallback) = self.fallback {
            match fallback.cleanup_expired().await {
                Ok(count) => total_cleaned += count,
                Err(e) => warn!("Fallback storage cleanup failed: {}", e),
            }
        }
        
        Ok(total_cleaned)
    }

    async fn reset_usage(&self, key: &RateLimitKey) -> Result<()> {
        // Reset in both storages
        let primary_result = self.primary.reset_usage(key).await;
        
        if let Some(ref fallback) = self.fallback {
            if let Err(e) = fallback.reset_usage(key).await {
                warn!("Fallback storage reset failed: {}", e);
            }
        }
        
        primary_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let storage = MemoryRateLimitStorage::new();
        let key = RateLimitKey::IpAddress("192.168.1.1".parse::<IpAddr>().unwrap());
        
        // Initially no usage
        let usage = storage.get_usage(&key).await.unwrap();
        assert!(usage.is_none());
        
        // Increment usage
        let new_usage = storage.increment_usage(&key, 1).await.unwrap();
        assert_eq!(new_usage.count, 1);
        
        // Get usage should return the incremented value
        let retrieved_usage = storage.get_usage(&key).await.unwrap();
        assert!(retrieved_usage.is_some());
        assert_eq!(retrieved_usage.unwrap().count, 1);
        
        // Reset usage
        storage.reset_usage(&key).await.unwrap();
        let usage_after_reset = storage.get_usage(&key).await.unwrap();
        assert!(usage_after_reset.is_none());
    }

    #[tokio::test]
    async fn test_memory_storage_cleanup() {
        let storage = MemoryRateLimitStorage::with_cleanup_interval(Duration::milliseconds(100));
        let key = RateLimitKey::IpAddress("192.168.1.1".parse::<IpAddr>().unwrap());
        
        // Add some usage
        storage.increment_usage(&key, 1).await.unwrap();
        
        let stats_before = storage.get_stats().await;
        assert_eq!(stats_before.total_entries, 1);
        
        // Force cleanup
        let cleaned = storage.cleanup_expired().await.unwrap();
        debug!("Cleaned {} entries", cleaned);
        
        // Note: cleanup uses conservative 1-hour threshold,
        // so recent entries won't be cleaned
        let stats_after = storage.get_stats().await;
        assert_eq!(stats_after.total_entries, 1);
    }

    #[tokio::test]
    async fn test_layered_storage() {
        let primary = Arc::new(MemoryRateLimitStorage::new());
        let fallback = Arc::new(MemoryRateLimitStorage::new());
        let layered = LayeredRateLimitStorage::new(
            primary.clone(),
            Some(fallback.clone()),
        );
        
        let key = RateLimitKey::IpAddress("192.168.1.1".parse::<IpAddr>().unwrap());
        
        // Increment through layered storage
        let usage = layered.increment_usage(&key, 1).await.unwrap();
        assert_eq!(usage.count, 1);
        
        // Both primary and fallback should have the data
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Allow async update
        
        let primary_usage = primary.get_usage(&key).await.unwrap();
        assert!(primary_usage.is_some());
        assert_eq!(primary_usage.unwrap().count, 1);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let storage = MemoryRateLimitStorage::new();
        let key1 = RateLimitKey::IpAddress("192.168.1.1".parse::<IpAddr>().unwrap());
        let key2 = RateLimitKey::IpAddress("192.168.1.2".parse::<IpAddr>().unwrap());
        
        // Add some entries
        storage.increment_usage(&key1, 1).await.unwrap();
        storage.increment_usage(&key2, 2).await.unwrap();
        
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert!(stats.estimated_memory_bytes > 0);
    }
} 