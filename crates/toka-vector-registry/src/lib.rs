//! Toka Vector Registry - Semantic Tool Discovery and Capability Matching
//!
//! This crate provides a vector-based registry system for dynamic tool discovery
//! and capability matching. It enables agents to find tools based on semantic
//! similarity, capability requirements, and contextual needs using embeddings
//! and vector similarity search.
//!
//! # Architecture
//!
//! The vector registry consists of:
//!
//! - **Embedding Engine**: Converts tool descriptions and queries to vector embeddings
//! - **Vector Store**: High-performance vector similarity search and storage
//! - **Capability Matcher**: Matches tool capabilities with agent requirements
//! - **Discovery API**: Semantic search interface for tool discovery
//! - **Registration System**: Dynamic tool registration with metadata extraction
//!
//! # Usage
//!
//! ```rust
//! use toka_vector_registry::{VectorRegistry, ToolQuery, SimilarityThreshold};
//! use toka_kernel::{ToolKernel, CapabilitySet};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize registry with kernel enforcement
//!     let kernel = ToolKernel::new().await?;
//!     let registry = VectorRegistry::new(kernel).await?;
//!     
//!     // Register tools with automatic metadata extraction
//!     registry.register_tool_from_description(
//!         "file_reader",
//!         "Reads and processes text files with encoding support"
//!     ).await?;
//!     
//!     // Discover tools based on natural language query
//!     let query = ToolQuery::new("I need to read a CSV file")
//!         .with_capabilities(&CapabilitySet::workspace_files())
//!         .with_similarity_threshold(0.7);
//!         
//!     let tools = registry.discover_tools(query).await?;
//!     println!("Found {} matching tools", tools.len());
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// Re-export kernel types
pub use toka_kernel::{
    ToolKernel, SecurityLevel, KernelError,
    Capability, CapabilitySet,
};

pub mod embeddings;
pub mod vectorstore;
pub mod matching;
pub mod discovery;

/// Vector-based tool registry for semantic discovery
pub struct VectorRegistry {
    kernel: Arc<ToolKernel>,
    embedding_engine: Arc<dyn EmbeddingEngine + Send + Sync>,
    vector_store: Arc<dyn VectorStore + Send + Sync>,
    tool_metadata: RwLock<HashMap<String, ToolRegistration>>,
    capability_index: RwLock<CapabilityIndex>,
}

/// Tool registration with vector embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistration {
    /// Unique tool identifier
    pub tool_id: String,
    /// Tool metadata
    pub metadata: ToolMetadata,
    /// Description embedding vector
    pub description_embedding: Vec<f32>,
    /// Capability embeddings
    pub capability_embeddings: HashMap<String, Vec<f32>>,
    /// Registration timestamp
    pub registered_at: std::time::SystemTime,
    /// Usage statistics
    pub usage_stats: UsageStatistics,
}

/// Enhanced tool metadata for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub capabilities: CapabilitySet,
    pub input_schema: JsonValue,
    pub output_schema: JsonValue,
    pub examples: Vec<ToolExample>,
    pub version: String,
    pub author: String,
}

/// Tool usage example for better discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub input: JsonValue,
    pub expected_output: String,
    pub use_case: String,
}

/// Tool usage statistics for ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub execution_count: u64,
    pub success_rate: f32,
    pub average_rating: f32,
    pub last_used: Option<std::time::SystemTime>,
}

/// Query for tool discovery
#[derive(Debug, Clone)]
pub struct ToolQuery {
    /// Natural language description of what's needed
    pub query_text: String,
    /// Required capabilities
    pub required_capabilities: Option<CapabilitySet>,
    /// Optional capabilities (nice to have)
    pub optional_capabilities: Option<CapabilitySet>,
    /// Category filter
    pub category_filter: Option<String>,
    /// Tag filters
    pub tag_filters: Option<Vec<String>>,
    /// Minimum similarity threshold
    pub similarity_threshold: f32,
    /// Maximum number of results
    pub max_results: usize,
    /// Security level constraint
    pub security_level: Option<SecurityLevel>,
}

/// Tool discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDiscoveryResult {
    /// Tool registration information
    pub tool: ToolRegistration,
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f32,
    /// Capability match score
    pub capability_match_score: f32,
    /// Combined relevance score
    pub relevance_score: f32,
    /// Explanation of why this tool was matched
    pub match_explanation: String,
}

/// Capability indexing for efficient matching
#[derive(Debug, Clone)]
struct CapabilityIndex {
    capability_to_tools: HashMap<String, Vec<String>>,
    tool_to_capabilities: HashMap<String, Vec<String>>,
}

/// Trait for embedding engines
#[async_trait::async_trait]
pub trait EmbeddingEngine {
    /// Generate embedding for text
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for multiple texts (batch operation)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    
    /// Get embedding dimension
    fn embedding_dimension(&self) -> usize;
    
    /// Get engine information
    fn engine_info(&self) -> EmbeddingEngineInfo;
}

/// Embedding engine information
#[derive(Debug, Clone)]
pub struct EmbeddingEngineInfo {
    pub name: String,
    pub version: String,
    pub model: String,
    pub dimension: usize,
    pub max_tokens: usize,
}

/// Trait for vector stores
#[async_trait::async_trait]
pub trait VectorStore {
    /// Store vector with metadata
    async fn store_vector(
        &self,
        id: &str,
        vector: &[f32],
        metadata: &JsonValue,
    ) -> Result<()>;
    
    /// Search for similar vectors
    async fn search_similar(
        &self,
        query_vector: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<VectorSearchResult>>;
    
    /// Remove vector by ID
    async fn remove_vector(&self, id: &str) -> Result<()>;
    
    /// Get vector by ID
    async fn get_vector(&self, id: &str) -> Result<Option<StoredVector>>;
    
    /// Get store statistics
    async fn get_stats(&self) -> Result<VectorStoreStats>;
}

/// Vector search result
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: JsonValue,
}

/// Stored vector with metadata
#[derive(Debug, Clone)]
pub struct StoredVector {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: JsonValue,
}

/// Vector store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_vectors: usize,
    pub index_size_bytes: usize,
    pub avg_search_time_ms: f32,
}

impl VectorRegistry {
    /// Create new vector registry with kernel enforcement
    pub async fn new(kernel: ToolKernel) -> Result<Self> {
        let embedding_engine = Arc::new(embeddings::OpenAIEmbeddingEngine::new().await?);
        let vector_store = Arc::new(vectorstore::InMemoryVectorStore::new(
            embedding_engine.embedding_dimension()
        ));
        
        Ok(Self {
            kernel: Arc::new(kernel),
            embedding_engine,
            vector_store,
            tool_metadata: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(CapabilityIndex::new()),
        })
    }
    
    /// Register a tool with automatic metadata extraction
    pub async fn register_tool_from_description(
        &self,
        tool_id: &str,
        description: &str,
    ) -> Result<()> {
        // Generate embedding for description
        let description_embedding = self.embedding_engine.embed_text(description).await?;
        
        // Extract capabilities from description (simplified)
        let capabilities = self.extract_capabilities_from_description(description).await?;
        
        // Create tool metadata
        let metadata = ToolMetadata {
            id: tool_id.to_string(),
            name: tool_id.to_string(), // Could be enhanced with NLP
            description: description.to_string(),
            category: "general".to_string(), // Could be classified automatically
            tags: vec![], // Could be extracted from description
            capabilities,
            input_schema: JsonValue::Null,
            output_schema: JsonValue::Null,
            examples: vec![],
            version: "1.0.0".to_string(),
            author: "system".to_string(),
        };
        
        self.register_tool_with_metadata(tool_id, metadata, description_embedding).await
    }
    
    /// Register a tool with full metadata
    pub async fn register_tool_with_metadata(
        &self,
        tool_id: &str,
        metadata: ToolMetadata,
        description_embedding: Vec<f32>,
    ) -> Result<()> {
        // Generate capability embeddings
        let mut capability_embeddings = HashMap::new();
        for capability in metadata.capabilities.iter() {
            let cap_text = format!("{}", capability);
            let cap_embedding = self.embedding_engine.embed_text(&cap_text).await?;
            capability_embeddings.insert(cap_text, cap_embedding);
        }
        
        // Create tool registration
        let registration = ToolRegistration {
            tool_id: tool_id.to_string(),
            metadata: metadata.clone(),
            description_embedding: description_embedding.clone(),
            capability_embeddings,
            registered_at: std::time::SystemTime::now(),
            usage_stats: UsageStatistics {
                execution_count: 0,
                success_rate: 0.0,
                average_rating: 0.0,
                last_used: None,
            },
        };
        
        // Store in vector store
        let vector_metadata = serde_json::to_value(&metadata)?;
        self.vector_store.store_vector(
            tool_id,
            &description_embedding,
            &vector_metadata,
        ).await?;
        
        // Update registrations
        let mut tool_metadata = self.tool_metadata.write().await;
        tool_metadata.insert(tool_id.to_string(), registration);
        
        // Update capability index
        self.update_capability_index(tool_id, &metadata.capabilities).await;
        
        tracing::info!("Registered tool in vector registry: {}", tool_id);
        Ok(())
    }
    
    /// Discover tools based on semantic query
    pub async fn discover_tools(&self, query: ToolQuery) -> Result<Vec<ToolDiscoveryResult>> {
        // Generate embedding for query
        let query_embedding = self.embedding_engine.embed_text(&query.query_text).await?;
        
        // Search for similar tools
        let similar_tools = self.vector_store.search_similar(
            &query_embedding,
            query.max_results * 2, // Get more for filtering
            query.similarity_threshold,
        ).await?;
        
        let mut results = Vec::new();
        let tool_metadata = self.tool_metadata.read().await;
        
        for search_result in similar_tools {
            if let Some(registration) = tool_metadata.get(&search_result.id) {
                // Calculate capability match score
                let capability_match_score = self.calculate_capability_match(
                    &registration.metadata.capabilities,
                    &query.required_capabilities,
                    &query.optional_capabilities,
                );
                
                // Apply filters
                if !self.passes_filters(&registration.metadata, &query) {
                    continue;
                }
                
                // Calculate combined relevance score
                let relevance_score = self.calculate_relevance_score(
                    search_result.score,
                    capability_match_score,
                    &registration.usage_stats,
                );
                
                // Generate match explanation
                let match_explanation = self.generate_match_explanation(
                    &registration.metadata,
                    search_result.score,
                    capability_match_score,
                );
                
                results.push(ToolDiscoveryResult {
                    tool: registration.clone(),
                    similarity_score: search_result.score,
                    capability_match_score,
                    relevance_score,
                    match_explanation,
                });
            }
        }
        
        // Sort by relevance score and limit results
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(query.max_results);
        
        Ok(results)
    }
    
    /// Update tool usage statistics
    pub async fn update_tool_usage(
        &self,
        tool_id: &str,
        success: bool,
        rating: Option<f32>,
    ) -> Result<()> {
        let mut tool_metadata = self.tool_metadata.write().await;
        
        if let Some(registration) = tool_metadata.get_mut(tool_id) {
            let stats = &mut registration.usage_stats;
            stats.execution_count += 1;
            
            // Update success rate
            let total_successes = (stats.success_rate * (stats.execution_count - 1) as f32) 
                + if success { 1.0 } else { 0.0 };
            stats.success_rate = total_successes / stats.execution_count as f32;
            
            // Update rating if provided
            if let Some(new_rating) = rating {
                if stats.average_rating == 0.0 {
                    stats.average_rating = new_rating;
                } else {
                    stats.average_rating = (stats.average_rating + new_rating) / 2.0;
                }
            }
            
            stats.last_used = Some(std::time::SystemTime::now());
        }
        
        Ok(())
    }
    
    /// Get tool recommendations based on usage patterns
    pub async fn get_recommendations(
        &self,
        session_id: &str,
        context: &str,
    ) -> Result<Vec<ToolDiscoveryResult>> {
        // Simple implementation - could be enhanced with ML models
        let query = ToolQuery::new(context)
            .with_similarity_threshold(0.6)
            .with_max_results(5);
            
        self.discover_tools(query).await
    }
    
    /// Extract capabilities from natural language description
    async fn extract_capabilities_from_description(&self, description: &str) -> Result<CapabilitySet> {
        // Simplified capability extraction - could use NLP models
        let mut capabilities = Vec::new();
        
        let description_lower = description.to_lowercase();
        
        if description_lower.contains("file") || description_lower.contains("read") {
            capabilities.push(Capability::FileRead(toka_kernel::FileAccess::Global));
        }
        
        if description_lower.contains("write") || description_lower.contains("save") {
            capabilities.push(Capability::FileWrite(toka_kernel::FileAccess::Global));
        }
        
        if description_lower.contains("network") || description_lower.contains("http") {
            capabilities.push(Capability::NetworkConnect(toka_kernel::NetworkAccess::Global));
        }
        
        if description_lower.contains("process") || description_lower.contains("command") {
            capabilities.push(Capability::ProcessSpawn);
        }
        
        Ok(CapabilitySet::with_capabilities(capabilities))
    }
    
    /// Update capability index
    async fn update_capability_index(&self, tool_id: &str, capabilities: &CapabilitySet) {
        let mut index = self.capability_index.write().await;
        
        for capability in capabilities.iter() {
            let cap_str = format!("{}", capability);
            index.capability_to_tools.entry(cap_str.clone())
                .or_insert_with(Vec::new)
                .push(tool_id.to_string());
        }
        
        let tool_caps: Vec<String> = capabilities.iter()
            .map(|cap| format!("{}", cap))
            .collect();
        index.tool_to_capabilities.insert(tool_id.to_string(), tool_caps);
    }
    
    /// Calculate capability match score
    fn calculate_capability_match(
        &self,
        tool_capabilities: &CapabilitySet,
        required_capabilities: &Option<CapabilitySet>,
        optional_capabilities: &Option<CapabilitySet>,
    ) -> f32 {
        let mut score = 1.0; // Start with perfect match
        
        // Check required capabilities
        if let Some(required) = required_capabilities {
            let required_count = required.len() as f32;
            let mut matched_count = 0.0;
            
            for cap in required.iter() {
                if tool_capabilities.contains(cap) {
                    matched_count += 1.0;
                }
            }
            
            if required_count > 0.0 {
                score *= matched_count / required_count;
            }
        }
        
        // Bonus for optional capabilities
        if let Some(optional) = optional_capabilities {
            let optional_count = optional.len() as f32;
            let mut optional_matched = 0.0;
            
            for cap in optional.iter() {
                if tool_capabilities.contains(cap) {
                    optional_matched += 1.0;
                }
            }
            
            if optional_count > 0.0 {
                score *= 1.0 + (optional_matched / optional_count) * 0.2; // 20% bonus
            }
        }
        
        score.min(1.0)
    }
    
    /// Check if tool passes query filters
    fn passes_filters(&self, metadata: &ToolMetadata, query: &ToolQuery) -> bool {
        // Category filter
        if let Some(category) = &query.category_filter {
            if metadata.category != *category {
                return false;
            }
        }
        
        // Tag filters
        if let Some(tag_filters) = &query.tag_filters {
            for required_tag in tag_filters {
                if !metadata.tags.contains(required_tag) {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Calculate combined relevance score
    fn calculate_relevance_score(
        &self,
        similarity_score: f32,
        capability_match_score: f32,
        usage_stats: &UsageStatistics,
    ) -> f32 {
        let popularity_score = if usage_stats.execution_count > 0 {
            (usage_stats.execution_count as f32).log10() / 4.0 // Normalize log scale
        } else {
            0.0
        };
        
        let quality_score = (usage_stats.success_rate + usage_stats.average_rating / 5.0) / 2.0;
        
        // Weighted combination
        similarity_score * 0.4 +
        capability_match_score * 0.3 +
        popularity_score.min(1.0) * 0.2 +
        quality_score * 0.1
    }
    
    /// Generate human-readable match explanation
    fn generate_match_explanation(
        &self,
        metadata: &ToolMetadata,
        similarity_score: f32,
        capability_match_score: f32,
    ) -> String {
        let mut explanation = Vec::new();
        
        if similarity_score > 0.8 {
            explanation.push("High semantic similarity to your query".to_string());
        } else if similarity_score > 0.6 {
            explanation.push("Moderate semantic similarity to your query".to_string());
        } else {
            explanation.push("Some semantic similarity to your query".to_string());
        }
        
        if capability_match_score > 0.8 {
            explanation.push("Strong capability match".to_string());
        } else if capability_match_score > 0.5 {
            explanation.push("Partial capability match".to_string());
        }
        
        if !metadata.tags.is_empty() {
            explanation.push(format!("Tagged as: {}", metadata.tags.join(", ")));
        }
        
        explanation.join("; ")
    }
}

impl ToolQuery {
    /// Create new tool query
    pub fn new(query_text: &str) -> Self {
        Self {
            query_text: query_text.to_string(),
            required_capabilities: None,
            optional_capabilities: None,
            category_filter: None,
            tag_filters: None,
            similarity_threshold: 0.5,
            max_results: 10,
            security_level: None,
        }
    }
    
    /// Add required capabilities
    pub fn with_capabilities(mut self, capabilities: &CapabilitySet) -> Self {
        self.required_capabilities = Some(capabilities.clone());
        self
    }
    
    /// Add optional capabilities
    pub fn with_optional_capabilities(mut self, capabilities: &CapabilitySet) -> Self {
        self.optional_capabilities = Some(capabilities.clone());
        self
    }
    
    /// Set similarity threshold
    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }
    
    /// Set maximum results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
    
    /// Add category filter
    pub fn with_category(mut self, category: &str) -> Self {
        self.category_filter = Some(category.to_string());
        self
    }
    
    /// Add tag filters
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tag_filters = Some(tags);
        self
    }
}

impl CapabilityIndex {
    fn new() -> Self {
        Self {
            capability_to_tools: HashMap::new(),
            tool_to_capabilities: HashMap::new(),
        }
    }
}

impl Default for UsageStatistics {
    fn default() -> Self {
        Self {
            execution_count: 0,
            success_rate: 0.0,
            average_rating: 0.0,
            last_used: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_kernel::presets;

    #[tokio::test]
    async fn test_vector_registry_creation() {
        let kernel = presets::testing_kernel().await.unwrap();
        let registry = VectorRegistry::new(kernel).await.unwrap();
        
        // Should be able to create registry
        assert!(registry.tool_metadata.read().await.is_empty());
    }
    
    #[tokio::test] 
    async fn test_tool_query_builder() {
        let query = ToolQuery::new("Read a CSV file")
            .with_similarity_threshold(0.8)
            .with_max_results(5)
            .with_category("file_operations");
            
        assert_eq!(query.query_text, "Read a CSV file");
        assert_eq!(query.similarity_threshold, 0.8);
        assert_eq!(query.max_results, 5);
        assert_eq!(query.category_filter, Some("file_operations".to_string()));
    }
    
    #[test]
    fn test_usage_statistics_default() {
        let stats = UsageStatistics::default();
        assert_eq!(stats.execution_count, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.average_rating, 0.0);
        assert!(stats.last_used.is_none());
    }
} 