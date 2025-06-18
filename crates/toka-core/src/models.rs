use crate::currency::MicroUSD;
use crate::ids::ModelID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about an LLM provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelProviderInfo {
    pub name: String, // e.g., "OpenAI", "Anthropic", "MistralAI", "Local"
    pub api_base_url: Option<String>,
}

/// Defines the cost structure for using a specific LLM model.
/// This represents the cost to *your platform*.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPricingInfo {
    pub model_id: ModelID, // Unique identifier for the model (e.g., "gpt-4-turbo-2024-04-09")
    pub friendly_name: String, // e.g., "GPT-4 Turbo (2024-04-09)"
    pub provider_info: ModelProviderInfo,
    pub input_cost_per_1k_tokens_micro_usd: MicroUSD, // Cost for 1000 input tokens
    pub output_cost_per_1k_tokens_micro_usd: MicroUSD, // Cost for 1000 output tokens
    pub request_fixed_cost_micro_usd: Option<MicroUSD>,
    pub unit_of_measure_is_tokens: bool, // True if costs are per token, false if per character or other unit.
    pub max_context_window_tokens: Option<u32>,
    pub supports_streaming: bool,
    pub last_updated: DateTime<Utc>, // When this pricing information was last verified/updated.
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>, // e.g., ["text-generation", "summarization", "experimental"]
}
