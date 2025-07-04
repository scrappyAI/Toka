//! Response validation to ensure safe outputs from LLM providers.
//!
//! This module provides comprehensive response validation to ensure that
//! LLM outputs are safe, appropriate, and meet security requirements.

use anyhow::{Context, Result};
use regex::Regex;
use tracing::{debug, warn};

use crate::LlmResponse;

/// Response validator that ensures safe outputs from LLM providers.
pub struct ResponseValidator {
    /// Patterns that indicate potentially harmful content
    harmful_patterns: Vec<HarmfulPattern>,
    /// Maximum allowed response length
    max_response_length: usize,
    /// Whether to enable code validation
    code_validation_enabled: bool,
}

/// A pattern that indicates potentially harmful content in responses.
#[derive(Debug, Clone)]
struct HarmfulPattern {
    /// The regex pattern to match
    pattern: Regex,
    /// Human-readable description of what this pattern detects
    description: String,
    /// Whether to block the response entirely or just sanitize
    block_response: bool,
}

impl ResponseValidator {
    /// Create a new response validator with default security patterns.
    pub fn new() -> Self {
        let harmful_patterns = vec![
            // Executable code patterns
            HarmfulPattern {
                pattern: Regex::new(r"(?i)(system|exec|eval|subprocess|shell|cmd|powershell|bash|sh)\s*\(").unwrap(),
                description: "Executable code in response".to_string(),
                block_response: true,
            },
            // File system operations
            HarmfulPattern {
                pattern: Regex::new(r"(?i)(open|read|write|delete|rm|mv|cp|mkdir|rmdir)\s*\(").unwrap(),
                description: "File system operations in response".to_string(),
                block_response: true,
            },
        ];
        
        Self {
            harmful_patterns,
            max_response_length: 1_048_576, // 1MB max
            code_validation_enabled: true,
        }
    }
    
    /// Validate a response to ensure it's safe and appropriate.
    pub fn validate(&self, response: LlmResponse) -> Result<LlmResponse> {
        debug!("Validating LLM response");
        
        let original_content = response.content().to_string();
        let validated_content = original_content.clone();
        let mut blocked_patterns = Vec::new();
        
        // Check for harmful patterns
        for pattern in &self.harmful_patterns {
            if pattern.pattern.is_match(&validated_content) {
                if pattern.block_response {
                    blocked_patterns.push(pattern.description.clone());
                }
            }
        }
        
        // Block response if harmful patterns found
        if !blocked_patterns.is_empty() {
            warn!("Blocking response due to harmful patterns: {:?}", blocked_patterns);
            anyhow::bail!("Response blocked due to security concerns: {}", blocked_patterns.join(", "));
        }
        
        // Validate response length
        if validated_content.len() > self.max_response_length {
            anyhow::bail!("Response too long: {} characters", validated_content.len());
        }
        
        Ok(response)
    }
    
    /// Check if a response would be blocked (for testing).
    pub fn would_block(&self, content: &str) -> bool {
        self.harmful_patterns
            .iter()
            .any(|pattern| pattern.block_response && pattern.pattern.is_match(content))
    }
}

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}
