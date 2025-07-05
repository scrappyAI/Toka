#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-store-semantic** – Semantic analysis implementation for Toka OS event store.
//!
//! This crate provides concrete implementations of the semantic analysis plugin interface
//! defined in `toka-store-core`. It includes a plugin registry, semantic engine, and
//! example plugin implementations for content classification, relationship extraction,
//! and anomaly detection.

use toka_store_core::semantic::*;
use toka_store_core::prelude::*;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Concrete implementation of the plugin registry.
///
/// This registry manages all registered semantic analysis plugins and their configurations.
/// It uses async-safe data structures to support multi-threaded access.
pub struct DefaultPluginRegistry {
    /// Content classifier plugins
    classifiers: RwLock<HashMap<PluginId, Box<dyn ContentClassifier>>>,
    /// Relationship extractor plugins
    extractors: RwLock<HashMap<PluginId, Box<dyn RelationshipExtractor>>>,
    /// Anomaly detector plugins
    detectors: RwLock<HashMap<PluginId, Box<dyn AnomalyDetector>>>,
    /// Plugin configurations
    configs: RwLock<HashMap<PluginId, PluginConfig>>,
}

impl DefaultPluginRegistry {
    /// Create a new plugin registry.
    pub fn new() -> Self {
        Self {
            classifiers: RwLock::new(HashMap::new()),
            extractors: RwLock::new(HashMap::new()),
            detectors: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Get the number of registered plugins.
    pub async fn plugin_count(&self) -> usize {
        let classifiers = self.classifiers.read().await;
        let extractors = self.extractors.read().await;
        let detectors = self.detectors.read().await;
        classifiers.len() + extractors.len() + detectors.len()
    }

    /// Check if a plugin is registered.
    pub async fn is_registered(&self, plugin_id: PluginId) -> bool {
        let classifiers = self.classifiers.read().await;
        let extractors = self.extractors.read().await;
        let detectors = self.detectors.read().await;
        classifiers.contains_key(&plugin_id) 
            || extractors.contains_key(&plugin_id)
            || detectors.contains_key(&plugin_id)
    }
}

impl Default for DefaultPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginRegistry {
    async fn register_classifier(&mut self, plugin: Box<dyn ContentClassifier>) -> SemanticResult<()> {
        let plugin_id = plugin.metadata().id;
        
        if self.is_registered(plugin_id).await {
            return Err(SemanticError::RegistrationFailed(
                format!("Plugin {} is already registered", plugin_id)
            ));
        }
        
        let mut classifiers = self.classifiers.write().await;
        classifiers.insert(plugin_id, plugin);
        Ok(())
    }

    async fn register_extractor(&mut self, plugin: Box<dyn RelationshipExtractor>) -> SemanticResult<()> {
        let plugin_id = plugin.metadata().id;
        
        if self.is_registered(plugin_id).await {
            return Err(SemanticError::RegistrationFailed(
                format!("Plugin {} is already registered", plugin_id)
            ));
        }
        
        let mut extractors = self.extractors.write().await;
        extractors.insert(plugin_id, plugin);
        Ok(())
    }

    async fn register_detector(&mut self, plugin: Box<dyn AnomalyDetector>) -> SemanticResult<()> {
        let plugin_id = plugin.metadata().id;
        
        if self.is_registered(plugin_id).await {
            return Err(SemanticError::RegistrationFailed(
                format!("Plugin {} is already registered", plugin_id)
            ));
        }
        
        let mut detectors = self.detectors.write().await;
        detectors.insert(plugin_id, plugin);
        Ok(())
    }

    async fn list_plugins(&self) -> SemanticResult<Vec<PluginMetadata>> {
        let mut plugins = Vec::new();
        
        let classifiers = self.classifiers.read().await;
        for classifier in classifiers.values() {
            plugins.push(classifier.metadata().clone());
        }
        
        let extractors = self.extractors.read().await;
        for extractor in extractors.values() {
            plugins.push(extractor.metadata().clone());
        }
        
        let detectors = self.detectors.read().await;
        for detector in detectors.values() {
            plugins.push(detector.metadata().clone());
        }
        
        Ok(plugins)
    }

    async fn unregister_plugin(&mut self, plugin_id: PluginId) -> SemanticResult<()> {
        let mut removed = false;
        
        {
            let mut classifiers = self.classifiers.write().await;
            if classifiers.remove(&plugin_id).is_some() {
                removed = true;
            }
        }
        
        {
            let mut extractors = self.extractors.write().await;
            if extractors.remove(&plugin_id).is_some() {
                removed = true;
            }
        }
        
        {
            let mut detectors = self.detectors.write().await;
            if detectors.remove(&plugin_id).is_some() {
                removed = true;
            }
        }
        
        if removed {
            let mut configs = self.configs.write().await;
            configs.remove(&plugin_id);
            Ok(())
        } else {
            Err(SemanticError::PluginNotFound(plugin_id))
        }
    }

    async fn get_config(&self, plugin_id: PluginId) -> SemanticResult<Option<PluginConfig>> {
        let configs = self.configs.read().await;
        Ok(configs.get(&plugin_id).cloned())
    }

    async fn update_config(&mut self, config: PluginConfig) -> SemanticResult<()> {
        if !self.is_registered(config.plugin_id).await {
            return Err(SemanticError::PluginNotFound(config.plugin_id));
        }
        
        let mut configs = self.configs.write().await;
        configs.insert(config.plugin_id, config);
        Ok(())
    }
}

/// Concrete implementation of the semantic analysis engine.
///
/// This engine coordinates all registered plugins to provide comprehensive
/// semantic analysis of event data.
pub struct DefaultSemanticEngine {
    /// Plugin registry
    registry: Arc<DefaultPluginRegistry>,
}

impl DefaultSemanticEngine {
    /// Create a new semantic engine with the given plugin registry.
    pub fn new(registry: Arc<DefaultPluginRegistry>) -> Self {
        Self { registry }
    }

    /// Create a new semantic engine with an empty registry.
    pub fn with_empty_registry() -> Self {
        Self::new(Arc::new(DefaultPluginRegistry::new()))
    }

    /// Get a reference to the plugin registry.
    pub fn registry(&self) -> &Arc<DefaultPluginRegistry> {
        &self.registry
    }
}

#[async_trait]
impl SemanticEngine for DefaultSemanticEngine {
    async fn classify_content(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<ClassificationResult>> {
        let mut all_results = Vec::new();
        
        // Run all enabled classifier plugins
        let classifiers = self.registry.classifiers.read().await;
        let configs = self.registry.configs.read().await;
        
        for classifier in classifiers.values() {
            let plugin_id = classifier.metadata().id;
            
            // Check if plugin is enabled
            if let Some(config) = configs.get(&plugin_id) {
                if !config.enabled {
                    continue;
                }
            }
            
            // Run classification
            match classifier.batch_analyze(events).await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    // Log error but continue with other plugins
                    eprintln!("Classification error in plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        Ok(all_results)
    }

    async fn extract_relationships(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph> {
        let mut combined_graph = RelationshipGraph {
            relationships: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Run all enabled extractor plugins
        let extractors = self.registry.extractors.read().await;
        let configs = self.registry.configs.read().await;
        
        for extractor in extractors.values() {
            let plugin_id = extractor.metadata().id;
            
            // Check if plugin is enabled
            if let Some(config) = configs.get(&plugin_id) {
                if !config.enabled {
                    continue;
                }
            }
            
            // Extract relationships
            match extractor.extract_relationships(events).await {
                Ok(graph) => {
                    combined_graph.relationships.extend(graph.relationships);
                    combined_graph.metadata.extend(graph.metadata);
                }
                Err(e) => {
                    // Log error but continue with other plugins
                    eprintln!("Relationship extraction error in plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        Ok(combined_graph)
    }

    async fn detect_anomalies(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<AnomalyReport>> {
        let mut all_reports = Vec::new();
        
        // Run all enabled detector plugins
        let detectors = self.registry.detectors.read().await;
        let configs = self.registry.configs.read().await;
        
        for detector in detectors.values() {
            let plugin_id = detector.metadata().id;
            
            // Check if plugin is enabled
            if let Some(config) = configs.get(&plugin_id) {
                if !config.enabled {
                    continue;
                }
            }
            
            // Detect anomalies
            match detector.detect_anomalies(events).await {
                Ok(reports) => all_reports.extend(reports),
                Err(e) => {
                    // Log error but continue with other plugins
                    eprintln!("Anomaly detection error in plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        Ok(all_reports)
    }

    async fn analyze(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<SemanticAnalysisResult> {
        // Run all analysis types in parallel
        let (classifications, relationships, anomalies) = tokio::join!(
            self.classify_content(events),
            self.extract_relationships(events),
            self.detect_anomalies(events)
        );
        
        Ok(SemanticAnalysisResult {
            classifications: classifications?,
            relationships: relationships?,
            anomalies: anomalies?,
            metadata: HashMap::from([
                ("engine_version".to_string(), env!("CARGO_PKG_VERSION").to_string()),
                ("analysis_timestamp".to_string(), chrono::Utc::now().to_rfc3339()),
            ]),
        })
    }
}

/// Example plugins module containing basic implementations.
pub mod examples {
    use super::*;
    
    /// Simple content classifier that classifies based on event kind.
    pub struct KindBasedClassifier {
        metadata: PluginMetadata,
    }
    
    impl KindBasedClassifier {
        /// Create a new kind-based classifier.
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "Kind-Based Classifier".to_string(),
                    description: "Classifies events based on their kind field".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Toka OS".to_string(),
                    config_schema: None,
                },
            }
        }
    }
    
    impl Default for KindBasedClassifier {
        fn default() -> Self {
            Self::new()
        }
    }
    
    #[async_trait]
    impl ContentClassifier for KindBasedClassifier {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn configure(&mut self, _config: &PluginConfig) -> SemanticResult<()> {
            // No configuration needed for this simple classifier
            Ok(())
        }
        
        async fn analyze(&self, header: &EventHeader, _payload: &[u8]) -> SemanticResult<ClassificationResult> {
            let category = if header.kind.contains('.') {
                header.kind.split('.').next().unwrap_or("unknown")
            } else {
                "simple"
            };
            
            Ok(ClassificationResult {
                category: category.to_string(),
                confidence: 0.8, // Fixed confidence for this simple classifier
                metadata: HashMap::from([
                    ("classifier".to_string(), "kind-based".to_string()),
                    ("original_kind".to_string(), header.kind.clone()),
                ]),
                tags: vec![header.kind.clone()],
            })
        }
    }
    
    /// Simple relationship extractor that creates relationships based on parent-child links.
    pub struct ParentChildExtractor {
        metadata: PluginMetadata,
    }
    
    impl ParentChildExtractor {
        /// Create a new parent-child extractor.
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "Parent-Child Extractor".to_string(),
                    description: "Extracts parent-child relationships from event headers".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Toka OS".to_string(),
                    config_schema: None,
                },
            }
        }
    }
    
    impl Default for ParentChildExtractor {
        fn default() -> Self {
            Self::new()
        }
    }
    
    #[async_trait]
    impl RelationshipExtractor for ParentChildExtractor {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn configure(&mut self, _config: &PluginConfig) -> SemanticResult<()> {
            // No configuration needed for this simple extractor
            Ok(())
        }
        
        async fn extract_relationships(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph> {
            let mut relationships = Vec::new();
            
            for (header, _payload) in events {
                for parent_id in &header.parents {
                    relationships.push(EventRelationship {
                        source_id: *parent_id,
                        target_id: header.id,
                        relationship_type: "parent_child".to_string(),
                        strength: 1.0, // Perfect strength for direct parent-child relationships
                        metadata: HashMap::from([
                            ("extractor".to_string(), "parent-child".to_string()),
                            ("child_kind".to_string(), header.kind.clone()),
                        ]),
                    });
                }
            }
            
            Ok(RelationshipGraph {
                relationships,
                metadata: HashMap::from([
                    ("extractor".to_string(), "parent-child".to_string()),
                    ("total_events".to_string(), events.len().to_string()),
                ]),
            })
        }
        
        async fn update_relationships(&self, existing: &RelationshipGraph, new_events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<RelationshipGraph> {
            let new_graph = self.extract_relationships(new_events).await?;
            
            let mut combined = existing.clone();
            combined.relationships.extend(new_graph.relationships);
            combined.metadata.extend(new_graph.metadata);
            
            Ok(combined)
        }
    }
    
    /// Simple anomaly detector that detects events with unusual timestamps.
    pub struct TimestampAnomalyDetector {
        metadata: PluginMetadata,
    }
    
    impl TimestampAnomalyDetector {
        /// Create a new timestamp anomaly detector.
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: Uuid::new_v4(),
                    name: "Timestamp Anomaly Detector".to_string(),
                    description: "Detects events with unusual timestamps".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Toka OS".to_string(),
                    config_schema: None,
                },
            }
        }
    }
    
    impl Default for TimestampAnomalyDetector {
        fn default() -> Self {
            Self::new()
        }
    }
    
    #[async_trait]
    impl AnomalyDetector for TimestampAnomalyDetector {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn configure(&mut self, _config: &PluginConfig) -> SemanticResult<()> {
            // No configuration needed for this simple detector
            Ok(())
        }
        
        async fn detect_anomalies(&self, events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<Vec<AnomalyReport>> {
            let mut reports = Vec::new();
            
            if events.is_empty() {
                return Ok(reports);
            }
            
            // Calculate mean timestamp
            let total_timestamp: i64 = events.iter()
                .map(|(header, _)| header.timestamp.timestamp())
                .sum();
            let mean_timestamp = total_timestamp / events.len() as i64;
            
            // Find events with timestamps significantly different from mean
            for (header, _payload) in events {
                let timestamp_diff = (header.timestamp.timestamp() - mean_timestamp).abs();
                
                // Flag as anomaly if timestamp is more than 1 hour from mean
                if timestamp_diff > 3600 {
                    reports.push(AnomalyReport {
                        event_id: header.id,
                        anomaly_type: "timestamp_outlier".to_string(),
                        severity: (timestamp_diff as f64 / 3600.0).min(1.0),
                        description: format!("Event timestamp is {} seconds from mean", timestamp_diff),
                        context: HashMap::from([
                            ("detector".to_string(), "timestamp".to_string()),
                            ("timestamp_diff".to_string(), timestamp_diff.to_string()),
                            ("mean_timestamp".to_string(), mean_timestamp.to_string()),
                        ]),
                        suggested_actions: vec![
                            "Verify event timestamp accuracy".to_string(),
                            "Check for clock synchronization issues".to_string(),
                        ],
                    });
                }
            }
            
            Ok(reports)
        }
        
        async fn update_model(&mut self, _events: &[(EventHeader, Vec<u8>)]) -> SemanticResult<()> {
            // This simple detector doesn't have a model to update
            Ok(())
        }
    }
}

/// Convenient prelude for importing common types.
pub mod prelude {
    pub use super::{
        DefaultPluginRegistry, DefaultSemanticEngine,
        examples::{KindBasedClassifier, ParentChildExtractor, TimestampAnomalyDetector},
    };
    pub use toka_store_core::semantic::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_store_core::prelude::*;
    
    #[tokio::test]
    async fn test_plugin_registry_basic_operations() {
        let mut registry = DefaultPluginRegistry::new();
        
        // Register a classifier
        let classifier = Box::new(examples::KindBasedClassifier::new());
        let classifier_id = classifier.metadata().id;
        
        registry.register_classifier(classifier).await.unwrap();
        
        // Check it's registered
        assert!(registry.is_registered(classifier_id).await);
        assert_eq!(registry.plugin_count().await, 1);
        
        // List plugins
        let plugins = registry.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, classifier_id);
        
        // Unregister
        registry.unregister_plugin(classifier_id).await.unwrap();
        assert!(!registry.is_registered(classifier_id).await);
        assert_eq!(registry.plugin_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_semantic_engine_full_analysis() {
        let mut registry = DefaultPluginRegistry::new();
        
        // Register example plugins
        registry.register_classifier(Box::new(examples::KindBasedClassifier::new())).await.unwrap();
        registry.register_extractor(Box::new(examples::ParentChildExtractor::new())).await.unwrap();
        registry.register_detector(Box::new(examples::TimestampAnomalyDetector::new())).await.unwrap();
        
        let engine = DefaultSemanticEngine::new(Arc::new(registry));
        
        // Create test events
        let parent_header = EventHeader {
            id: Uuid::new_v4(),
            parents: smallvec::SmallVec::new(),
            timestamp: chrono::Utc::now(),
            digest: [0u8; 32],
            intent: Uuid::new_v4(),
            kind: "test.parent".to_string(),
        };
        
        let child_header = EventHeader {
            id: Uuid::new_v4(),
            parents: smallvec::SmallVec::from_vec(vec![parent_header.id]),
            timestamp: chrono::Utc::now(),
            digest: [1u8; 32],
            intent: Uuid::new_v4(),
            kind: "test.child".to_string(),
        };
        
        let events = vec![
            (parent_header.clone(), vec![1, 2, 3]),
            (child_header.clone(), vec![4, 5, 6]),
        ];
        
        // Run full analysis
        let result = engine.analyze(&events).await.unwrap();
        
        // Check results
        assert_eq!(result.classifications.len(), 2); // One for each event
        assert_eq!(result.relationships.relationships.len(), 1); // One parent-child relationship
        assert_eq!(result.anomalies.len(), 0); // No timestamp anomalies with recent events
        
        // Verify classification results
        assert!(result.classifications.iter().any(|c| c.category == "test"));
        
        // Verify relationship
        let relationship = &result.relationships.relationships[0];
        assert_eq!(relationship.source_id, parent_header.id);
        assert_eq!(relationship.target_id, child_header.id);
        assert_eq!(relationship.relationship_type, "parent_child");
    }
    
    #[tokio::test]
    async fn test_example_classifier() {
        let classifier = examples::KindBasedClassifier::new();
        
        let header = EventHeader {
            id: Uuid::new_v4(),
            parents: smallvec::SmallVec::new(),
            timestamp: chrono::Utc::now(),
            digest: [0u8; 32],
            intent: Uuid::new_v4(),
            kind: "user.login".to_string(),
        };
        
        let result = classifier.analyze(&header, &[]).await.unwrap();
        
        assert_eq!(result.category, "user");
        assert_eq!(result.confidence, 0.8);
        assert!(result.tags.contains(&"user.login".to_string()));
    }
} 