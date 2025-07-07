#![forbid(unsafe_code)]

//! Semantic analysis plugin interface for event content analysis.
//!
//! This module provides traits and types for implementing pluggable semantic analysis
//! of event data. The plugin system enables custom analysis without modifying core
//! storage logic, supporting extensions for content classification, relationship
//! extraction, and anomaly detection.

use crate::{EventHeader, EventId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a semantic analysis plugin.
pub type PluginId = Uuid;

/// Result type for semantic analysis operations.
pub type SemanticResult<T> = Result<T, SemanticError>;

/// Errors that can occur during semantic analysis.
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    /// Plugin not found
    #[error("plugin not found: {0}")]
    PluginNotFound(PluginId),
    /// Analysis failed
    #[error("semantic analysis failed: {0}")]
    AnalysisFailed(String),
    /// Invalid plugin configuration
    #[error("invalid plugin configuration: {0}")]
    InvalidConfiguration(String),
    /// Plugin registration failed
    #[error("plugin registration failed: {0}")]
    RegistrationFailed(String),
}

/// Metadata about a semantic analysis plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: PluginId,
    /// Human-readable name
    pub name: String,
    /// Brief description of the plugin's purpose
    pub description: String,
    /// Version string
    pub version: String,
    /// Plugin author information
    pub author: String,
    /// Plugin configuration schema (JSON Schema)
    pub config_schema: Option<String>,
}

/// Configuration parameters for a semantic analysis plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin identifier
    pub plugin_id: PluginId,
    /// Configuration parameters as JSON
    pub parameters: serde_json::Value,
    /// Whether the plugin is enabled
    pub enabled: bool,
}

/// Classification result for event content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// The classified category
    pub category: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Additional metadata about the classification
    pub metadata: HashMap<String, String>,
    /// Sub-categories or tags
    pub tags: Vec<String>,
}

/// Relationship between two events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRelationship {
    /// Source event ID
    pub source_id: EventId,
    /// Target event ID
    pub target_id: EventId,
    /// Type of relationship
    pub relationship_type: String,
    /// Strength of the relationship (0.0 to 1.0)
    pub strength: f64,
    /// Additional relationship metadata
    pub metadata: HashMap<String, String>,
}

/// Graph of relationships between events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipGraph {
    /// All relationships in the graph
    pub relationships: Vec<EventRelationship>,
    /// Graph-level metadata
    pub metadata: HashMap<String, String>,
}

/// Anomaly detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    /// Event ID that contains the anomaly
    pub event_id: EventId,
    /// Type of anomaly detected
    pub anomaly_type: String,
    /// Severity level (0.0 to 1.0)
    pub severity: f64,
    /// Human-readable description
    pub description: String,
    /// Additional context about the anomaly
    pub context: HashMap<String, String>,
    /// Suggested actions to take
    pub suggested_actions: Vec<String>,
}

/// Content classifier plugin interface.
///
/// Analyzes event content to classify it into semantic categories.
#[async_trait]
pub trait ContentClassifier: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Configure the plugin
    async fn configure(&mut self, config: &PluginConfig) -> SemanticResult<()>;

    /// Analyze event content and return classification
    async fn analyze(&self, header: &EventHeader, payload: &[u8]) -> SemanticResult<ClassificationResult>;

    /// Batch analyze multiple events
    async fn batch_analyze(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<ClassificationResult>> {
        let mut results = Vec::new();
        for (header, payload) in events {
            results.push(self.analyze(header, payload).await?);
        }
        Ok(results)
    }
}

/// Relationship extractor plugin interface.
///
/// Extracts relationships between events based on content and metadata.
#[async_trait]
pub trait RelationshipExtractor: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Configure the plugin
    async fn configure(&mut self, config: &PluginConfig) -> SemanticResult<()>;

    /// Extract relationships from a set of events
    async fn extract_relationships(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph>;

    /// Update relationships with new events
    async fn update_relationships(&self, existing: &RelationshipGraph, new_events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph>;
}

/// Anomaly detector plugin interface.
///
/// Detects unusual patterns in event sequences and content.
#[async_trait]
pub trait AnomalyDetector: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Configure the plugin
    async fn configure(&mut self, config: &PluginConfig) -> SemanticResult<()>;

    /// Detect anomalies in a stream of events
    async fn detect_anomalies(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<AnomalyReport>>;

    /// Update anomaly detection model with new events
    async fn update_model(&mut self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<()>;
}

/// Plugin types supported by the semantic analysis framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// Content classification plugin
    ContentClassifier,
    /// Relationship extraction plugin
    RelationshipExtractor,
    /// Anomaly detection plugin
    AnomalyDetector,
}

/// Trait for plugin registry management.
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a content classifier plugin
    async fn register_classifier(&mut self, plugin: Box<dyn ContentClassifier>) -> SemanticResult<()>;

    /// Register a relationship extractor plugin
    async fn register_extractor(&mut self, plugin: Box<dyn RelationshipExtractor>) -> SemanticResult<()>;

    /// Register an anomaly detector plugin
    async fn register_detector(&mut self, plugin: Box<dyn AnomalyDetector>) -> SemanticResult<()>;

    /// Get registered plugin metadata
    async fn list_plugins(&self) -> SemanticResult<Vec<PluginMetadata>>;

    /// Remove a plugin from the registry
    async fn unregister_plugin(&mut self, plugin_id: PluginId) -> SemanticResult<()>;

    /// Get plugin configuration
    async fn get_config(&self, plugin_id: PluginId) -> SemanticResult<Option<PluginConfig>>;

    /// Update plugin configuration
    async fn update_config(&mut self, config: PluginConfig) -> SemanticResult<()>;
}

/// Semantic analysis engine that coordinates plugins.
#[async_trait]
pub trait SemanticEngine: Send + Sync {
    /// Run content classification on events
    async fn classify_content(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<ClassificationResult>>;

    /// Extract relationships between events
    async fn extract_relationships(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph>;

    /// Detect anomalies in event stream
    async fn detect_anomalies(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<AnomalyReport>>;

    /// Run full semantic analysis pipeline
    async fn analyze(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<SemanticAnalysisResult>;
}

/// Complete semantic analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisResult {
    /// Classification results
    pub classifications: Vec<ClassificationResult>,
    /// Relationship graph
    pub relationships: RelationshipGraph,
    /// Anomaly reports
    pub anomalies: Vec<AnomalyReport>,
    /// Analysis metadata
    pub metadata: HashMap<String, String>,
}

/// Builder for creating semantic analysis configurations.
#[derive(Debug, Clone)]
pub struct SemanticConfigBuilder {
    configs: Vec<PluginConfig>,
}

impl SemanticConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Add a plugin configuration
    pub fn add_plugin(mut self, plugin_id: PluginId, parameters: serde_json::Value, enabled: bool) -> Self {
        self.configs.push(PluginConfig {
            plugin_id,
            parameters,
            enabled,
        });
        self
    }

    /// Build the configuration list
    pub fn build(self) -> Vec<PluginConfig> {
        self.configs
    }
}

impl Default for SemanticConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic analysis utilities.
pub mod utils {
    use super::*;

    /// Create plugin metadata for a content classifier
    pub fn classifier_metadata(name: &str, description: &str, version: &str, author: &str) -> PluginMetadata {
        PluginMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            config_schema: None,
        }
    }

    /// Create plugin metadata for a relationship extractor
    pub fn extractor_metadata(name: &str, description: &str, version: &str, author: &str) -> PluginMetadata {
        PluginMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            config_schema: None,
        }
    }

    /// Create plugin metadata for an anomaly detector
    pub fn detector_metadata(name: &str, description: &str, version: &str, author: &str) -> PluginMetadata {
        PluginMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            config_schema: None,
        }
    }

    /// Validate plugin configuration against schema
    pub fn validate_config(config: &PluginConfig, metadata: &PluginMetadata) -> Result<(), String> {
        if config.plugin_id != metadata.id {
            return Err("Plugin ID mismatch".to_string());
        }

        // TODO: Implement JSON schema validation if schema is provided
        if let Some(_schema) = &metadata.config_schema {
            // Schema validation would go here
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_plugin_metadata_creation() {
        let metadata = utils::classifier_metadata(
            "Test Classifier",
            "A test content classifier",
            "1.0.0",
            "Test Author",
        );

        assert_eq!(metadata.name, "Test Classifier");
        assert_eq!(metadata.description, "A test content classifier");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "Test Author");
    }

    #[test]
    fn test_config_builder() {
        let configs = SemanticConfigBuilder::new()
            .add_plugin(Uuid::new_v4(), json!({"param": "value"}), true)
            .add_plugin(Uuid::new_v4(), json!({"param2": "value2"}), false)
            .build();

        assert_eq!(configs.len(), 2);
        assert!(configs[0].enabled);
        assert!(!configs[1].enabled);
    }

    #[test]
    fn test_classification_result() {
        let result = ClassificationResult {
            category: "test_category".to_string(),
            confidence: 0.85,
            metadata: HashMap::from([("key".to_string(), "value".to_string())]),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert_eq!(result.category, "test_category");
        assert_eq!(result.confidence, 0.85);
        assert_eq!(result.tags.len(), 2);
    }

    #[test]
    fn test_relationship_graph() {
        let relationship = EventRelationship {
            source_id: Uuid::new_v4(),
            target_id: Uuid::new_v4(),
            relationship_type: "follows".to_string(),
            strength: 0.9,
            metadata: HashMap::new(),
        };

        let graph = RelationshipGraph {
            relationships: vec![relationship],
            metadata: HashMap::new(),
        };

        assert_eq!(graph.relationships.len(), 1);
        assert_eq!(graph.relationships[0].relationship_type, "follows");
    }

    #[test]
    fn test_anomaly_report() {
        let report = AnomalyReport {
            event_id: Uuid::new_v4(),
            anomaly_type: "unusual_pattern".to_string(),
            severity: 0.7,
            description: "Detected unusual pattern in event sequence".to_string(),
            context: HashMap::from([("pattern".to_string(), "burst".to_string())]),
            suggested_actions: vec!["investigate".to_string()],
        };

        assert_eq!(report.anomaly_type, "unusual_pattern");
        assert_eq!(report.severity, 0.7);
        assert_eq!(report.suggested_actions.len(), 1);
    }
} 