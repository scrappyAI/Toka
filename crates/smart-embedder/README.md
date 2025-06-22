# Smart Embedder

This crate provides an agentic approach to generating semantic embeddings from structured events. Instead of hand-writing templates for every event type, it uses Large Language Models (LLMs) to automatically summarize events into natural language, then embeds those summaries into dense vectors.

## Core Concept

The smart embedder follows a simple pipeline:

1. **Structured Event** → **JSON serialization**
2. **JSON** → **LLM summarization** → **Natural language description**  
3. **Description** → **Sentence embedding** → **Dense vector**

This approach is:
- **Template-free**: No need to write custom summarizers for each event type
- **Semantic**: Captures the intent and meaning, not just structure
- **Clustering-friendly**: Similar events naturally cluster together in embedding space

## Architecture

The crate is built around two key traits:

- **`Llm`**: Abstracts any completion backend (OpenAI, local models, etc.)
- **`SentenceEncoder`**: Abstracts sentence → vector encoding

This design allows you to mix and match different LLM providers and embedding models.

## Usage

### Basic Setup

```rust
use smart_embedder::{SmartEmbedder, Llm, SentenceEncoder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserRegistration {
    username: String,
    email: String,
    timestamp: u64,
}

async fn example() -> anyhow::Result<()> {
    // Set up your LLM and encoder (implementations depend on features)
    let llm = MyLlmClient::new();
    let encoder = MyEncoder::new();
    let embedder = SmartEmbedder::new(llm, encoder);
    
    let event = UserRegistration {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        timestamp: 1234567890,
    };
    
    // Generate embedding
    let vector = embedder.embed_event("user.registration", &event).await?;
    
    // Use with vault
    vault.commit(&event, &[], "user.registration", vector).await?;
    
    Ok(())
}
```

### With OpenAI (Feature: `openai`)

```toml
[dependencies]
smart-embedder = { version = "0.1", features = ["openai"] }
```

```rust
use smart_embedder::openai_client::OpenAiClient;

let llm = OpenAiClient::new(
    std::env::var("OPENAI_API_KEY")?,
    "https://api.openai.com".to_string()
);
```

### With Local Transformers (Feature: `transformers`)

```toml
[dependencies]
smart-embedder = { version = "0.1", features = ["transformers"] }
```

```rust
use smart_embedder::transformer_encoder::BertEncoder;

let encoder = BertEncoder::new()?; // Loads all-MiniLM-L6-v2
```

## Features

- **`openai`**: Enables OpenAI-compatible HTTP client
- **`transformers`**: Enables local BERT-based sentence embeddings via `rust-bert`

By default, no features are enabled, allowing you to provide your own implementations.

## Custom Implementations

### Custom LLM Client

```rust
use smart_embedder::{Llm, async_trait};

struct MyLocalLlm {
    // Your local model setup
}

#[async_trait]
impl Llm for MyLocalLlm {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        // Call your local model (ollama, llama.cpp, etc.)
        todo!()
    }
}
```

### Custom Encoder

```rust
use smart_embedder::{SentenceEncoder, async_trait};
use ndarray::Array1;

struct MyEncoder {
    // Your embedding model
}

#[async_trait]
impl SentenceEncoder for MyEncoder {
    async fn encode(&self, sentence: &str) -> anyhow::Result<Array1<f32>> {
        // Encode sentence to vector
        todo!()
    }
}
```

## Integration with Toka Vaults

The smart embedder is designed to work seamlessly with the Toka vault system:

```rust
use toka_ledger::VaultBus;
use smart_embedder::SmartEmbedder;

async fn commit_with_smart_embedding<T: Serialize>(
    vault: &VaultBus,
    embedder: &SmartEmbedder<impl Llm, impl SentenceEncoder>,
    event: &T,
    kind: &str,
    parents: &[EventHeader],
) -> anyhow::Result<EventHeader> {
    let embedding = embedder.embed_event(kind, event).await?;
    vault.commit(event, parents, kind, embedding).await
}
```

## Example: Event Summarization

Given this event:

```json
{
  "user_id": "alice123",
  "action": "purchase",
  "item_id": "book_456",
  "amount": 29.99,
  "currency": "USD"
}
```

The LLM might generate: *"User alice123 purchased item book_456 for $29.99"*

This natural language summary is then embedded into a dense vector that captures the semantic meaning of the purchase action.

## Performance Considerations

- **LLM calls**: Can be slow (100ms-2s) and expensive. Consider caching by event content hash.
- **Local vs Remote**: Local models are faster but may be less capable than cloud models.
- **Batch processing**: For high throughput, consider batching multiple events in a single LLM call.

## Roadmap

- **Caching**: Built-in digest-based caching to avoid re-processing identical events
- **Batch processing**: Support for embedding multiple events in a single LLM call
- **Fine-tuning**: Tools for fine-tuning embedders on domain-specific event logs
- **Streaming**: Support for streaming large event sequences 