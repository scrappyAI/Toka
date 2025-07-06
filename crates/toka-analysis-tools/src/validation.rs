//! Input and output validation for analysis tools

use anyhow::Result;
use toka_tools::ToolParams;
use crate::AnalysisError;

/// Input validator
pub struct InputValidator;

impl InputValidator {
    /// Create a new input validator
    pub fn new() -> Self {
        Self
    }
    
    /// Validate tool parameters
    pub fn validate(&self, params: &ToolParams) -> Result<()> {
        // Basic validation - in practice would be more comprehensive
        for (key, value) in &params.args {
            if value.is_empty() {
                return Err(AnalysisError::InvalidInput(
                    format!("Parameter {} cannot be empty", key)
                ).into());
            }
        }
        Ok(())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Output validator
pub struct OutputValidator;

impl OutputValidator {
    /// Create a new output validator
    pub fn new() -> Self {
        Self
    }
    
    /// Validate output data
    pub fn validate(&self, output: &str) -> Result<()> {
        // Basic validation - in practice would validate JSON/format structure
        if output.is_empty() {
            return Err(AnalysisError::OutputProcessingFailed(
                "Output is empty".to_string()
            ).into());
        }
        Ok(())
    }
}

impl Default for OutputValidator {
    fn default() -> Self {
        Self::new()
    }
}