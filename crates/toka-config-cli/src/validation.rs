//! Input validation functions for configuration management operations.
//!
//! This module provides comprehensive validation for file paths, configuration values,
//! and key paths to ensure safe and reliable operations with meaningful error feedback.

use std::path::{Path, PathBuf};
use crate::error::{ConfigError, Result};

/// Validates that a file path is safe and accessible for operations.
///
/// Performs comprehensive validation including:
/// - Path safety checks (no traversal attacks)
/// - Extension validation for supported formats
/// - Parent directory accessibility
/// - Basic security validation
///
/// # Arguments
/// * `path` - The file path to validate
/// * `require_exists` - Whether the file must already exist
///
/// # Returns
/// * `Ok(())` if the path is valid and safe
/// * `Err(ConfigError)` with specific validation failure details
pub fn validate_file_path(path: &Path, require_exists: bool) -> Result<()> {
    // Check for path traversal attempts
    if path.components().any(|component| {
        matches!(component, std::path::Component::ParentDir)
    }) {
        return Err(ConfigError::invalid_format(
            path,
            "Path traversal (../) is not allowed for security reasons",
        ));
    }

    // Validate file extension for supported formats
    if let Some(extension) = path.extension() {
        let ext_str = extension.to_string_lossy().to_lowercase();
        if !matches!(ext_str.as_str(), "yaml" | "yml" | "json" | "toml") {
            return Err(ConfigError::invalid_format(
                path,
                format!("Unsupported file extension: {}", ext_str),
            ));
        }
    } else {
        return Err(ConfigError::invalid_format(
            path,
            "File must have a supported extension (.yaml, .yml, .json, .toml)",
        ));
    }

    // Check if file exists when required
    if require_exists && !path.exists() {
        return Err(ConfigError::file_not_found(path.to_path_buf()));
    }

    // Validate parent directory exists and is accessible
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(ConfigError::directory_error(
                parent,
                "Parent directory does not exist",
            ));
        }

        // Check if we can read the parent directory
        if let Err(e) = std::fs::read_dir(parent) {
            return Err(ConfigError::io_error(parent, e));
        }
    }

    Ok(())
}

/// Validates a key path for configuration operations.
///
/// Supports dot notation for nested keys and validates:
/// - Key path format and characters
/// - Depth limitations for safety
/// - Reserved key name conflicts
///
/// # Arguments
/// * `key_path` - The key path to validate (supports dot notation)
///
/// # Returns
/// * `Ok(Vec<String>)` containing the parsed key segments
/// * `Err(ConfigError)` with validation failure details
pub fn validate_key_path(key_path: &str) -> Result<Vec<String>> {
    if key_path.is_empty() {
        return Err(ConfigError::invalid_key_path(
            key_path,
            "Key path cannot be empty",
        ));
    }

    // Split by dots and validate each segment
    let segments: Vec<String> = key_path
        .split('.')
        .map(|s| s.trim().to_string())
        .collect();

    // Check depth limitation (prevent excessive nesting)
    if segments.len() > 10 {
        return Err(ConfigError::invalid_key_path(
            key_path,
            "Key path depth cannot exceed 10 levels",
        ));
    }

    // Validate each segment
    for (index, segment) in segments.iter().enumerate() {
        if segment.is_empty() {
            return Err(ConfigError::invalid_key_path(
                key_path,
                format!("Empty key segment at position {}", index + 1),
            ));
        }

        // Check for valid characters (alphanumeric, underscore, hyphen)
        if !segment.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ConfigError::invalid_key_path(
                key_path,
                format!(
                    "Invalid characters in key segment '{}'. Only alphanumeric, underscore, and hyphen are allowed",
                    segment
                ),
            ));
        }

        // Check for reserved names
        if matches!(segment.as_str(), "null" | "true" | "false" | "undefined") {
            return Err(ConfigError::invalid_key_path(
                key_path,
                format!("'{}' is a reserved keyword and cannot be used as a key", segment),
            ));
        }

        // Keys should not start with numbers for better compatibility
        if segment.chars().next().map_or(false, |c| c.is_numeric()) {
            return Err(ConfigError::invalid_key_path(
                key_path,
                format!("Key segment '{}' cannot start with a number", segment),
            ));
        }
    }

    Ok(segments)
}

/// Validates configuration content in JSON format.
///
/// Performs syntax validation and basic structure checks to ensure
/// the content can be safely parsed and processed.
///
/// # Arguments
/// * `content` - The JSON content string to validate
///
/// # Returns
/// * `Ok(serde_json::Value)` containing the parsed JSON value
/// * `Err(ConfigError)` with parsing failure details
pub fn validate_json_content(content: &str) -> Result<serde_json::Value> {
    serde_json::from_str(content).map_err(|e| {
        ConfigError::validation_error(
            "JSON content",
            format!("Invalid JSON syntax: {}", e),
        )
    })
}

/// Validates a directory path for listing operations.
///
/// Ensures the directory exists, is accessible, and safe to read.
///
/// # Arguments
/// * `directory` - The directory path to validate
///
/// # Returns
/// * `Ok(())` if the directory is valid and accessible
/// * `Err(ConfigError)` with validation failure details
pub fn validate_directory_path(directory: &Path) -> Result<()> {
    // Check for path traversal attempts
    if directory.components().any(|component| {
        matches!(component, std::path::Component::ParentDir)
    }) {
        return Err(ConfigError::directory_error(
            directory,
            "Path traversal (../) is not allowed for security reasons",
        ));
    }

    // Check if directory exists
    if !directory.exists() {
        return Err(ConfigError::directory_error(
            directory,
            "Directory does not exist",
        ));
    }

    // Check if it's actually a directory
    if !directory.is_dir() {
        return Err(ConfigError::directory_error(
            directory,
            "Path exists but is not a directory",
        ));
    }

    // Check if we can read the directory
    if let Err(e) = std::fs::read_dir(directory) {
        return Err(ConfigError::io_error(directory, e));
    }

    Ok(())
}

/// Determines the configuration format from a file extension.
///
/// # Arguments
/// * `path` - The file path to analyze
///
/// # Returns
/// * `Ok(String)` containing the detected format ("yaml", "json", or "toml")
/// * `Err(ConfigError)` if the format cannot be determined
pub fn detect_format_from_path(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .ok_or_else(|| {
            ConfigError::invalid_format(
                path,
                "File must have an extension to determine format",
            )
        })?
        .to_string_lossy()
        .to_lowercase();

    match extension.as_str() {
        "yaml" | "yml" => Ok("yaml".to_string()),
        "json" => Ok("json".to_string()),
        "toml" => Ok("toml".to_string()),
        _ => Err(ConfigError::invalid_format(
            path,
            format!("Unsupported file extension: {}", extension),
        )),
    }
}

/// Sanitizes a file path by resolving it and checking for safety.
///
/// # Arguments
/// * `path` - The raw file path to sanitize
///
/// # Returns
/// * `Ok(PathBuf)` containing the sanitized canonical path
/// * `Err(ConfigError)` if the path cannot be safely resolved
pub fn sanitize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    
    // Convert to absolute path if relative
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| ConfigError::io_error("current directory", e))?
            .join(path)
    };

    // Basic validation
    validate_file_path(&absolute_path, false)?;
    
    Ok(absolute_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_key_path_valid() {
        let result = validate_key_path("user.profile.name");
        assert!(result.is_ok());
        
        let segments = result.unwrap();
        assert_eq!(segments, vec!["user", "profile", "name"]);
    }

    #[test]
    fn test_validate_key_path_empty() {
        let result = validate_key_path("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_key_path_reserved() {
        let result = validate_key_path("user.null.name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reserved"));
    }

    #[test]
    fn test_validate_key_path_invalid_chars() {
        let result = validate_key_path("user.profile@domain.name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid characters"));
    }

    #[test]
    fn test_validate_key_path_starts_with_number() {
        let result = validate_key_path("user.1profile.name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot start with a number"));
    }

    #[test]
    fn test_validate_json_content_valid() {
        let result = validate_json_content(r#"{"key": "value"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_content_invalid() {
        let result = validate_json_content(r#"{"key": "value""#);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_detect_format_from_path() {
        assert_eq!(detect_format_from_path(&PathBuf::from("config.yaml")).unwrap(), "yaml");
        assert_eq!(detect_format_from_path(&PathBuf::from("config.yml")).unwrap(), "yaml");
        assert_eq!(detect_format_from_path(&PathBuf::from("config.json")).unwrap(), "json");
        assert_eq!(detect_format_from_path(&PathBuf::from("config.toml")).unwrap(), "toml");
    }

    #[test]
    fn test_detect_format_unsupported() {
        let result = detect_format_from_path(&PathBuf::from("config.xml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn test_validate_file_path_traversal() {
        let path = PathBuf::from("../../../etc/passwd");
        let result = validate_file_path(&path, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("traversal"));
    }
}