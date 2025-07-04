//! Core configuration management operations.
//!
//! This module provides the main ConfigManager that handles create, read, update,
//! and delete operations for configuration files in YAML, JSON, and TOML formats.
//! All operations include comprehensive validation and error handling.

use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;
use tracing::{info, debug};

use crate::cli::ConfigFormat;
use crate::error::{ConfigError, Result};
use crate::validation::{
    validate_file_path, validate_key_path, validate_json_content,
    validate_directory_path, detect_format_from_path, sanitize_path,
};

/// Main configuration management interface.
///
/// Provides a unified interface for managing configuration files across different
/// formats with comprehensive validation, error handling, and logging.
pub struct ConfigManager {
    // Future: Could add caching, hooks, or other state here
}

impl ConfigManager {
    /// Creates a new ConfigManager instance.
    ///
    /// # Returns
    /// A new ConfigManager ready for configuration operations.
    pub fn new() -> Self {
        debug!("Creating new ConfigManager instance");
        Self {}
    }

    /// Creates a new configuration file with initial content.
    ///
    /// # Arguments
    /// * `file_path` - Path where the configuration file should be created
    /// * `format` - Format for the configuration file
    /// * `initial_content` - Initial content as a JSON string
    ///
    /// # Returns
    /// * `Ok(())` if the file was created successfully
    /// * `Err(ConfigError)` if creation failed
    pub async fn create_config(
        &self,
        file_path: &Path,
        format: ConfigFormat,
        initial_content: String,
    ) -> Result<()> {
        info!("Creating configuration file: {:?} (format: {})", file_path, format);

        // Sanitize and validate the file path
        let safe_path = sanitize_path(file_path)?;
        validate_file_path(&safe_path, false)?;

        // Check if file already exists
        if safe_path.exists() {
            return Err(ConfigError::invalid_format(
                &safe_path,
                "File already exists. Use update command to modify existing files",
            ));
        }

        // Validate and parse initial content
        let content_value = validate_json_content(&initial_content)?;
        debug!("Parsed initial content: {:?}", content_value);

        // Convert content to target format
        let formatted_content = self.serialize_to_format(&content_value, &format)?;

        // Ensure parent directory exists
        if let Some(parent) = safe_path.parent() {
            if !parent.exists() {
                debug!("Creating parent directory: {:?}", parent);
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::io_error(parent, e))?;
            }
        }

        // Write the file
        fs::write(&safe_path, formatted_content)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;

        println!("✅ Configuration file created: {}", safe_path.display());
        info!("Successfully created configuration file: {:?}", safe_path);

        Ok(())
    }

    /// Reads and displays a configuration file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the configuration file to read
    ///
    /// # Returns
    /// * `Ok(())` if the file was read and displayed successfully
    /// * `Err(ConfigError)` if reading failed
    pub async fn read_config(&self, file_path: &Path) -> Result<()> {
        info!("Reading configuration file: {:?}", file_path);

        // Sanitize and validate the file path
        let safe_path = sanitize_path(file_path)?;
        validate_file_path(&safe_path, true)?;

        // Read file content
        let content = fs::read_to_string(&safe_path)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;

        // Detect format and parse content
        let format = detect_format_from_path(&safe_path)?;
        let parsed_value = self.parse_content(&content, &format, &safe_path)?;

        // Display the content in a formatted way
        println!("📄 Configuration file: {}", safe_path.display());
        println!("📋 Format: {}", format.to_uppercase());
        println!("📊 Content:");
        
        // Pretty print as JSON for consistency
        let pretty_json = serde_json::to_string_pretty(&parsed_value)
            .map_err(|e| ConfigError::serialization_error("json", e))?;
        println!("{}", pretty_json);

        debug!("Successfully read and displayed configuration file: {:?}", safe_path);
        Ok(())
    }

    /// Updates a key-value pair in a configuration file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the configuration file to update
    /// * `key_path` - Key path to update (supports dot notation)
    /// * `new_value` - New value as a JSON string
    ///
    /// # Returns
    /// * `Ok(())` if the update was successful
    /// * `Err(ConfigError)` if the update failed
    pub async fn update_config(
        &self,
        file_path: &Path,
        key_path: &str,
        new_value: &str,
    ) -> Result<()> {
        info!("Updating configuration file: {:?}, key: {}", file_path, key_path);

        // Sanitize and validate inputs
        let safe_path = sanitize_path(file_path)?;
        validate_file_path(&safe_path, true)?;
        let key_segments = validate_key_path(key_path)?;
        let value = validate_json_content(new_value)?;

        // Read and parse existing content
        let content = fs::read_to_string(&safe_path)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;
        
        let format = detect_format_from_path(&safe_path)?;
        let mut config_value = self.parse_content(&content, &format, &safe_path)?;

        // Update the value using key path
        self.update_nested_value(&mut config_value, &key_segments, value)?;

        // Convert back to original format and write
        let config_format = match format.as_str() {
            "yaml" => ConfigFormat::Yaml,
            "json" => ConfigFormat::Json,
            "toml" => ConfigFormat::Toml,
            _ => return Err(ConfigError::invalid_format(&safe_path, "Unknown format")),
        };

        let formatted_content = self.serialize_to_format(&config_value, &config_format)?;
        
        fs::write(&safe_path, formatted_content)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;

        println!("✅ Updated key '{}' in {}", key_path, safe_path.display());
        info!("Successfully updated configuration file: {:?}", safe_path);

        Ok(())
    }

    /// Deletes a key from a configuration file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the configuration file
    /// * `key_path` - Key path to delete (supports dot notation)
    ///
    /// # Returns
    /// * `Ok(())` if the deletion was successful
    /// * `Err(ConfigError)` if the deletion failed
    pub async fn delete_key(&self, file_path: &Path, key_path: &str) -> Result<()> {
        info!("Deleting key from configuration file: {:?}, key: {}", file_path, key_path);

        // Sanitize and validate inputs
        let safe_path = sanitize_path(file_path)?;
        validate_file_path(&safe_path, true)?;
        let key_segments = validate_key_path(key_path)?;

        // Read and parse existing content
        let content = fs::read_to_string(&safe_path)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;
        
        let format = detect_format_from_path(&safe_path)?;
        let mut config_value = self.parse_content(&content, &format, &safe_path)?;

        // Delete the key
        self.delete_nested_key(&mut config_value, &key_segments, &safe_path)?;

        // Convert back to original format and write
        let config_format = match format.as_str() {
            "yaml" => ConfigFormat::Yaml,
            "json" => ConfigFormat::Json,
            "toml" => ConfigFormat::Toml,
            _ => return Err(ConfigError::invalid_format(&safe_path, "Unknown format")),
        };

        let formatted_content = self.serialize_to_format(&config_value, &config_format)?;
        
        fs::write(&safe_path, formatted_content)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;

        println!("✅ Deleted key '{}' from {}", key_path, safe_path.display());
        info!("Successfully deleted key from configuration file: {:?}", safe_path);

        Ok(())
    }

    /// Validates the syntax and structure of a configuration file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the configuration file to validate
    ///
    /// # Returns
    /// * `Ok(())` if the file is valid
    /// * `Err(ConfigError)` if validation failed
    pub async fn validate_config(&self, file_path: &Path) -> Result<()> {
        info!("Validating configuration file: {:?}", file_path);

        // Sanitize and validate the file path
        let safe_path = sanitize_path(file_path)?;
        validate_file_path(&safe_path, true)?;

        // Read and parse content
        let content = fs::read_to_string(&safe_path)
            .map_err(|e| ConfigError::io_error(&safe_path, e))?;

        let format = detect_format_from_path(&safe_path)?;
        let parsed_value = self.parse_content(&content, &format, &safe_path)?;

        // Perform additional validation checks
        self.validate_config_structure(&parsed_value, &safe_path)?;

        println!("✅ Configuration file is valid: {}", safe_path.display());
        println!("📋 Format: {}", format.to_uppercase());
        println!("📊 Keys found: {}", self.count_keys(&parsed_value));
        
        info!("Configuration file validation successful: {:?}", safe_path);
        Ok(())
    }

    /// Lists all configuration files in a directory.
    ///
    /// # Arguments
    /// * `directory` - Directory to search for configuration files
    ///
    /// # Returns
    /// * `Ok(())` if the listing was successful
    /// * `Err(ConfigError)` if the listing failed
    pub async fn list_configs(&self, directory: &Path) -> Result<()> {
        info!("Listing configuration files in: {:?}", directory);

        // Validate directory
        validate_directory_path(directory)?;

        // Read directory contents
        let entries = fs::read_dir(directory)
            .map_err(|e| ConfigError::io_error(directory, e))?;

        let mut config_files = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| ConfigError::io_error(directory, e))?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(format) = detect_format_from_path(&path) {
                    // Try to validate the file
                    let status = match self.quick_validate(&path) {
                        Ok(_) => "✅ Valid",
                        Err(_) => "❌ Invalid",
                    };

                    config_files.push((path, format, status));
                }
            }
        }

        // Display results
        println!("📁 Configuration files in: {}", directory.display());
        
        if config_files.is_empty() {
            println!("   No configuration files found");
        } else {
            println!("   Found {} configuration file(s):", config_files.len());
            
            for (path, format, status) in config_files {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                println!("   {} {} ({})", status, file_name, format.to_uppercase());
            }
        }

        info!("Successfully listed configuration files in: {:?}", directory);
        Ok(())
    }

    // Private helper methods

    /// Parses configuration content based on format.
    fn parse_content(&self, content: &str, format: &str, file_path: &Path) -> Result<JsonValue> {
        match format {
            "json" => {
                serde_json::from_str(content)
                    .map_err(|e| ConfigError::parse_error(file_path, e))
            }
            "yaml" => {
                serde_yaml::from_str(content)
                    .map_err(|e| ConfigError::parse_error(file_path, e))
            }
            "toml" => {
                let toml_value: toml::Value = toml::from_str(content)
                    .map_err(|e| ConfigError::parse_error(file_path, e))?;
                
                // Convert TOML value to JSON value
                serde_json::to_value(toml_value)
                    .map_err(|e| ConfigError::parse_error(file_path, e))
            }
            _ => Err(ConfigError::invalid_format(file_path, "Unsupported format")),
        }
    }

    /// Serializes content to the specified format.
    fn serialize_to_format(&self, value: &JsonValue, format: &ConfigFormat) -> Result<String> {
        match format {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(value)
                    .map_err(|e| ConfigError::serialization_error("json", e))
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(value)
                    .map_err(|e| ConfigError::serialization_error("yaml", e))
            }
            ConfigFormat::Toml => {
                // Convert JSON value to TOML value first
                let toml_value: toml::Value = serde_json::from_value(value.clone())
                    .map_err(|e| ConfigError::serialization_error("toml", e))?;
                
                toml::to_string_pretty(&toml_value)
                    .map_err(|e| ConfigError::serialization_error("toml", e))
            }
        }
    }

    /// Updates a nested value using key path segments.
    fn update_nested_value(
        &self,
        config: &mut JsonValue,
        key_segments: &[String],
        new_value: JsonValue,
    ) -> Result<()> {
        if key_segments.is_empty() {
            return Err(ConfigError::invalid_key_path("", "Key path cannot be empty"));
        }

        let mut current = config;
        
        // Navigate to parent of target key
        for segment in &key_segments[..key_segments.len() - 1] {
            current = match current {
                JsonValue::Object(ref mut obj) => {
                    obj.entry(segment.clone()).or_insert(JsonValue::Object(serde_json::Map::new()))
                }
                _ => {
                    return Err(ConfigError::validation_error(
                        "configuration",
                        format!("Cannot navigate through non-object value at key '{}'", segment),
                    ));
                }
            };
        }

        // Set the final value
        match current {
            JsonValue::Object(ref mut obj) => {
                let final_key = &key_segments[key_segments.len() - 1];
                obj.insert(final_key.clone(), new_value);
                Ok(())
            }
            _ => Err(ConfigError::validation_error(
                "configuration",
                "Cannot set key on non-object value",
            )),
        }
    }

    /// Deletes a nested key using key path segments.
    fn delete_nested_key(
        &self,
        config: &mut JsonValue,
        key_segments: &[String],
        file_path: &Path,
    ) -> Result<()> {
        if key_segments.is_empty() {
            return Err(ConfigError::invalid_key_path("", "Key path cannot be empty"));
        }

        let mut current = config;
        
        // Navigate to parent of target key
        for segment in &key_segments[..key_segments.len() - 1] {
            current = match current {
                JsonValue::Object(ref mut obj) => {
                    obj.get_mut(segment).ok_or_else(|| {
                        ConfigError::key_not_found(segment, file_path)
                    })?
                }
                _ => {
                    return Err(ConfigError::validation_error(
                        file_path,
                        format!("Cannot navigate through non-object value at key '{}'", segment),
                    ));
                }
            };
        }

        // Delete the final key
        match current {
            JsonValue::Object(ref mut obj) => {
                let final_key = &key_segments[key_segments.len() - 1];
                if obj.remove(final_key).is_none() {
                    return Err(ConfigError::key_not_found(final_key, file_path));
                }
                Ok(())
            }
            _ => Err(ConfigError::validation_error(
                file_path,
                "Cannot delete key from non-object value",
            )),
        }
    }

    /// Performs quick validation for listing purposes.
    fn quick_validate(&self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| ConfigError::io_error(file_path, e))?;
        
        let format = detect_format_from_path(file_path)?;
        self.parse_content(&content, &format, file_path)?;
        
        Ok(())
    }

    /// Validates configuration structure for additional checks.
    fn validate_config_structure(&self, _value: &JsonValue, _file_path: &Path) -> Result<()> {
        // Future: Add custom validation rules here
        // For now, successful parsing is sufficient validation
        Ok(())
    }

    /// Counts the number of keys in a configuration value.
    fn count_keys(&self, value: &JsonValue) -> usize {
        match value {
            JsonValue::Object(obj) => {
                obj.len() + obj.values().map(|v| self.count_keys(v)).sum::<usize>()
            }
            JsonValue::Array(arr) => {
                arr.iter().map(|v| self.count_keys(v)).sum()
            }
            _ => 0,
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    async fn create_test_manager() -> (ConfigManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_create_json_config() {
        let (manager, temp_dir) = create_test_manager().await;
        let file_path = temp_dir.path().join("test.json");
        
        let result = manager
            .create_config(&file_path, ConfigFormat::Json, r#"{"name": "test"}"#.to_string())
            .await;
        
        assert!(result.is_ok());
        assert!(file_path.exists());
        
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("test"));
    }

    #[tokio::test]
    async fn test_read_config() {
        let (manager, temp_dir) = create_test_manager().await;
        let file_path = temp_dir.path().join("test.json");
        
        // Create a test file
        fs::write(&file_path, r#"{"name": "test"}"#).unwrap();
        
        let result = manager.read_config(&file_path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_config() {
        let (manager, temp_dir) = create_test_manager().await;
        let file_path = temp_dir.path().join("test.json");
        
        // Create initial file
        fs::write(&file_path, r#"{"user": {"name": "old"}}"#).unwrap();
        
        let result = manager
            .update_config(&file_path, "user.name", r#""new""#)
            .await;
        
        assert!(result.is_ok());
        
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("new"));
    }

    #[tokio::test]
    async fn test_validate_config() {
        let (manager, temp_dir) = create_test_manager().await;
        let file_path = temp_dir.path().join("test.json");
        
        fs::write(&file_path, r#"{"valid": "json"}"#).unwrap();
        
        let result = manager.validate_config(&file_path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_configs() {
        let (manager, temp_dir) = create_test_manager().await;
        
        // Create test files
        fs::write(temp_dir.path().join("test1.json"), r#"{"test": 1}"#).unwrap();
        fs::write(temp_dir.path().join("test2.yaml"), "test: 2").unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "not a config").unwrap();
        
        let result = manager.list_configs(temp_dir.path()).await;
        assert!(result.is_ok());
    }
}