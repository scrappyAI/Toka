//! Utility functions and helpers for Toka Agent OS
//!
//! This module provides common utility functions, helpers, and convenience
//! methods used across the Toka ecosystem.

use crate::error::{TokaError, TokaResult};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp in seconds
    pub fn now_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get current Unix timestamp in milliseconds
    pub fn now_millis() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    /// Convert duration to human-readable string
    pub fn duration_to_string(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        
        if total_secs < 60 {
            return format!("{}s", total_secs);
        }
        
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        
        if mins < 60 {
            if secs == 0 {
                return format!("{}m", mins);
            }
            return format!("{}m{}s", mins, secs);
        }
        
        let hours = mins / 60;
        let mins = mins % 60;
        
        if hours < 24 {
            if mins == 0 && secs == 0 {
                return format!("{}h", hours);
            } else if secs == 0 {
                return format!("{}h{}m", hours, mins);
            }
            return format!("{}h{}m{}s", hours, mins, secs);
        }
        
        let days = hours / 24;
        let hours = hours % 24;
        
        if hours == 0 && mins == 0 && secs == 0 {
            format!("{}d", days)
        } else {
            format!("{}d{}h{}m{}s", days, hours, mins, secs)
        }
    }

    /// Parse human-readable duration string
    pub fn parse_duration(s: &str) -> TokaResult<Duration> {
        let s = s.trim().to_lowercase();
        
        // Simple parser for common duration formats
        if s.ends_with("ms") {
            let num = s.trim_end_matches("ms").parse::<u64>()
                .map_err(|_| TokaError::validation("Invalid milliseconds format"))?;
            return Ok(Duration::from_millis(num));
        }
        
        if s.ends_with('s') {
            let num = s.trim_end_matches('s').parse::<u64>()
                .map_err(|_| TokaError::validation("Invalid seconds format"))?;
            return Ok(Duration::from_secs(num));
        }
        
        if s.ends_with('m') {
            let num = s.trim_end_matches('m').parse::<u64>()
                .map_err(|_| TokaError::validation("Invalid minutes format"))?;
            return Ok(Duration::from_secs(num * 60));
        }
        
        if s.ends_with('h') {
            let num = s.trim_end_matches('h').parse::<u64>()
                .map_err(|_| TokaError::validation("Invalid hours format"))?;
            return Ok(Duration::from_secs(num * 3600));
        }
        
        if s.ends_with('d') {
            let num = s.trim_end_matches('d').parse::<u64>()
                .map_err(|_| TokaError::validation("Invalid days format"))?;
            return Ok(Duration::from_secs(num * 86400));
        }
        
        // Try to parse as plain number (assume seconds)
        if let Ok(num) = s.parse::<u64>() {
            return Ok(Duration::from_secs(num));
        }
        
        Err(TokaError::validation("Invalid duration format"))
    }
}

/// String utilities
pub struct StringUtils;

impl StringUtils {
    /// Truncate string to specified length with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    /// Convert string to snake_case
    pub fn to_snake_case(s: &str) -> String {
        let mut result = String::new();
        let mut prev_char_was_uppercase = false;
        
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && !prev_char_was_uppercase {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap_or(c));
                prev_char_was_uppercase = true;
            } else {
                result.push(c);
                prev_char_was_uppercase = false;
            }
        }
        
        result
    }

    /// Convert string to kebab-case
    pub fn to_kebab_case(s: &str) -> String {
        Self::to_snake_case(s).replace('_', "-")
    }

    /// Sanitize string for use as identifier
    pub fn sanitize_identifier(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>()
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_string()
    }

    /// Check if string is valid identifier
    pub fn is_valid_identifier(s: &str) -> bool {
        !s.is_empty() 
            && s.chars().next().unwrap_or(' ').is_alphabetic()
            && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
}

/// Collection utilities
pub struct CollectionUtils;

impl CollectionUtils {
    /// Chunk a vector into smaller vectors of specified size
    pub fn chunk_vec<T: Clone>(vec: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
        if chunk_size == 0 {
            return vec![vec];
        }
        
        vec.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Deduplicate vector while preserving order
    pub fn dedup_ordered<T: Clone + PartialEq>(vec: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in vec {
            if !result.contains(&item) {
                result.push(item);
            }
        }
        result
    }
}

/// Path utilities
pub struct PathUtils;

impl PathUtils {
    /// Expand tilde in path
    pub fn expand_tilde(path: &str) -> String {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return path.replacen("~", &home.to_string_lossy(), 1);
            }
        }
        path.to_string()
    }

    /// Ensure directory exists, create if it doesn't
    pub fn ensure_dir_exists(path: &str) -> TokaResult<()> {
        let expanded_path = Self::expand_tilde(path);
        std::fs::create_dir_all(&expanded_path)
            .map_err(|e| TokaError::io_with_path(expanded_path, format!("Failed to create directory: {}", e)))
    }

    /// Check if path is safe (no path traversal)
    pub fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.starts_with('/')
    }
}

/// Retry utilities
pub struct RetryUtils;

impl RetryUtils {
    /// Retry an operation with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(
        mut operation: F,
        max_attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        let mut delay = initial_delay;
        
        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(e);
                    }
                    
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
        
        unreachable!()
    }
}

// Add dirs crate to dependencies (this would need to be added to Cargo.toml)
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_formatting() {
        assert_eq!(TimeUtils::duration_to_string(Duration::from_secs(30)), "30s");
        assert_eq!(TimeUtils::duration_to_string(Duration::from_secs(90)), "1m30s");
        assert_eq!(TimeUtils::duration_to_string(Duration::from_secs(3661)), "1h1m1s");
        assert_eq!(TimeUtils::duration_to_string(Duration::from_secs(86400)), "1d");
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(TimeUtils::parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(TimeUtils::parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(TimeUtils::parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(TimeUtils::parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(TimeUtils::parse_duration("500ms").unwrap(), Duration::from_millis(500));
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::truncate("hello world", 5), "he...");
        assert_eq!(StringUtils::truncate("hi", 5), "hi");
        
        assert_eq!(StringUtils::to_snake_case("CamelCase"), "camel_case");
        assert_eq!(StringUtils::to_kebab_case("CamelCase"), "camel-case");
        
        assert!(StringUtils::is_valid_identifier("valid_name"));
        assert!(!StringUtils::is_valid_identifier("123invalid"));
        assert!(!StringUtils::is_valid_identifier(""));
        
        assert_eq!(StringUtils::sanitize_identifier("test@name!"), "test_name");
    }

    #[test]
    fn test_collection_utils() {
        let vec = vec![1, 2, 3, 4, 5];
        let chunks = CollectionUtils::chunk_vec(vec, 2);
        assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
        
        let vec_with_dups = vec![1, 2, 3, 2, 4, 1, 5];
        let deduped = CollectionUtils::dedup_ordered(vec_with_dups);
        assert_eq!(deduped, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_path_utils() {
        assert!(!PathUtils::is_safe_path("../etc/passwd"));
        assert!(!PathUtils::is_safe_path("/etc/passwd"));
        assert!(PathUtils::is_safe_path("config/app.json"));
        
        // Test tilde expansion (would work in real environment with HOME set)
        let path = PathUtils::expand_tilde("~/test");
        assert!(!path.starts_with("~/")); // Should be expanded if HOME is set
    }
}