//! Configuration management for Toka Agent OS
//!
//! This module provides utilities for loading, validating, and managing
//! configuration across the Toka ecosystem.

use crate::error::{TokaError, TokaResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Base configuration trait that all Toka configurations should implement
pub trait TokaConfig: Serialize + for<'de> Deserialize<'de> + Default {
    /// Validate the configuration
    fn validate(&self) -> TokaResult<()> {
        Ok(())
    }

    /// Load configuration from environment variables
    fn from_env() -> TokaResult<Self> {
        let config = Self::default();
        // Default implementation returns default config
        Ok(config)
    }

    /// Load configuration from a file
    fn from_file<P: AsRef<Path>>(path: P) -> TokaResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TokaError::config(format!("Failed to read config file: {}", e)))?;
        
        // Try JSON first, then YAML
        if let Ok(config) = serde_json::from_str::<Self>(&content) {
            config.validate()?;
            return Ok(config);
        }
        
        // If JSON fails, try YAML (would need serde_yaml crate)
        Err(TokaError::config("Unsupported config format or invalid syntax"))
    }

    /// Merge with another configuration
    fn merge(&mut self, _other: Self) -> TokaResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Environment variable helper
pub struct EnvHelper;

impl EnvHelper {
    /// Get environment variable or return default
    pub fn get_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get environment variable or return error
    pub fn get_required(key: &str) -> TokaResult<String> {
        env::var(key).map_err(|_| {
            TokaError::config_with_key(key, format!("Required environment variable {} not set", key))
        })
    }

    /// Parse environment variable as specific type
    pub fn get_parsed<T>(key: &str) -> TokaResult<Option<T>>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        match env::var(key) {
            Ok(value) => {
                let parsed = value.parse::<T>()
                    .map_err(|e| TokaError::config_with_key(
                        key, 
                        format!("Failed to parse {}: {}", key, e)
                    ))?;
                Ok(Some(parsed))
            }
            Err(_) => Ok(None),
        }
    }

    /// Get boolean from environment variable
    pub fn get_bool(key: &str, default: bool) -> bool {
        match env::var(key) {
            Ok(value) => {
                match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => default,
                }
            }
            Err(_) => default,
        }
    }
}

/// Configuration builder for dynamic configuration construction
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    values: HashMap<String, serde_json::Value>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Set a configuration value
    pub fn set<T: Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.values.insert(key.into(), json_value);
        }
        self
    }

    /// Set a value from environment variable
    pub fn set_from_env(mut self, key: impl Into<String>, env_key: &str) -> Self {
        if let Ok(value) = env::var(env_key) {
            self.values.insert(key.into(), serde_json::Value::String(value));
        }
        self
    }

    /// Set a value from environment variable with default
    pub fn set_from_env_or_default<T: Serialize>(
        mut self, 
        key: impl Into<String>, 
        env_key: &str, 
        default: T
    ) -> Self {
        let value = if let Ok(env_value) = env::var(env_key) {
            serde_json::Value::String(env_value)
        } else {
            serde_json::to_value(default).unwrap_or(serde_json::Value::Null)
        };
        self.values.insert(key.into(), value);
        self
    }

    /// Build the configuration
    pub fn build<T>(self) -> TokaResult<T>
    where
        T: for<'de> Deserialize<'de> + TokaConfig,
    {
        let json_value = serde_json::Value::Object(
            self.values.into_iter()
                .map(|(k, v)| (k, v))
                .collect()
        );
        
        let config: T = serde_json::from_value(json_value)
            .map_err(|e| TokaError::config(format!("Failed to build config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common configuration paths
pub struct ConfigPaths;

impl ConfigPaths {
    /// Get the default configuration directory
    pub fn default_config_dir() -> String {
        EnvHelper::get_or_default("TOKA_CONFIG_DIR", "~/.config/toka")
    }

    /// Get the default data directory
    pub fn default_data_dir() -> String {
        EnvHelper::get_or_default("TOKA_DATA_DIR", "~/.local/share/toka")
    }

    /// Get the default log directory
    pub fn default_log_dir() -> String {
        EnvHelper::get_or_default("TOKA_LOG_DIR", "/var/log/toka")
    }

    /// Get the default runtime directory
    pub fn default_runtime_dir() -> String {
        EnvHelper::get_or_default("TOKA_RUNTIME_DIR", "/var/run/toka")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
    struct TestConfig {
        name: String,
        count: i32,
        enabled: bool,
    }

    impl TokaConfig for TestConfig {
        fn validate(&self) -> TokaResult<()> {
            if self.name.is_empty() {
                return Err(TokaError::validation_with_field("name", "Name cannot be empty"));
            }
            if self.count < 0 {
                return Err(TokaError::validation_with_field("count", "Count must be non-negative"));
            }
            Ok(())
        }
    }

    #[test]
    fn test_config_builder() {
        let config: TestConfig = ConfigBuilder::new()
            .set("name", "test")
            .set("count", 42)
            .set("enabled", true)
            .build()
            .unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.count, 42);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_validation() {
        let config = TestConfig {
            name: "".to_string(),
            count: -1,
            enabled: true,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_helper() {
        env::set_var("TEST_VAR", "test_value");
        assert_eq!(EnvHelper::get_or_default("TEST_VAR", "default"), "test_value");
        assert_eq!(EnvHelper::get_or_default("NONEXISTENT_VAR", "default"), "default");
        
        env::set_var("TEST_BOOL", "true");
        assert!(EnvHelper::get_bool("TEST_BOOL", false));
        
        env::remove_var("TEST_VAR");
        env::remove_var("TEST_BOOL");
    }
}