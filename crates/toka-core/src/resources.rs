use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::ids::ResourceID;
// Model-specific info has been moved to models.rs

/// Represents the type of a resource managed or tracked by the platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    LLMModel, // This variant indicates that more specific info can be found via ModelID
    Tool,
    AgentCapability,
    Dataset,
    APIEndpoint,
    Other(String),
}

/// A generic descriptor for any resource on the platform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceDescriptor {
    pub resource_id: ResourceID, // Unique ID for this specific resource instance
    pub resource_type: ResourceType,
    pub name: String, // User-friendly name of the resource
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} 