//! Result caching for analysis tools

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::AnalysisResult;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Maximum cache size
    pub max_size: usize,
    /// Time to live for cache entries
    pub ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
            ttl: Duration::from_secs(3600),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cached entries
    pub entries: usize,
}

/// Result cache
pub struct ResultCache {
    config: CacheConfig,
    cache: HashMap<String, CacheEntry>,
    stats: CacheStats,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    result: AnalysisResult,
    timestamp: SystemTime,
}

impl ResultCache {
    /// Create a new result cache
    pub fn new(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            config,
            cache: HashMap::new(),
            stats: CacheStats {
                hits: 0,
                misses: 0,
                entries: 0,
            },
        })
    }
    
    /// Get cached result
    pub fn get(&mut self, key: &str) -> Option<AnalysisResult> {
        if !self.config.enabled {
            return None;
        }
        
        if let Some(entry) = self.cache.get(key) {
            if entry.timestamp.elapsed().unwrap_or(Duration::MAX) < self.config.ttl {
                self.stats.hits += 1;
                return Some(entry.result.clone());
            } else {
                self.cache.remove(key);
            }
        }
        
        self.stats.misses += 1;
        None
    }
    
    /// Store result in cache
    pub fn put(&mut self, key: String, result: AnalysisResult) {
        if !self.config.enabled {
            return;
        }
        
        // Evict old entries if cache is full
        if self.cache.len() >= self.config.max_size {
            self.evict_oldest();
        }
        
        let entry = CacheEntry {
            result,
            timestamp: SystemTime::now(),
        };
        
        self.cache.insert(key, entry);
        self.stats.entries = self.cache.len();
    }
    
    /// Clear cache
    pub async fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        self.stats.entries = 0;
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
    
    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.cache.remove(&oldest_key);
        }
    }
}