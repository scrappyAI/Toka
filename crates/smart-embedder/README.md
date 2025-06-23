# Smart Embedder

Smart embedding generation utilities for the Toka platform.

## Overview

This crate provides utilities for generating and managing embeddings for text and other data types. It's designed to support semantic search, similarity matching, and other AI-powered features within the Toka ecosystem.

## Features

- Text embedding generation
- Embedding similarity calculations
- Vector storage and retrieval
- Batch processing support
- Multiple embedding model support
- Semantic search capabilities

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
smart-embedder = "0.1.0"
```

### Example

```rust
use smart_embedder::{Embedder, EmbeddingModel};

let embedder = Embedder::new(EmbeddingModel::Default);

// Generate embeddings
let text = "Hello, world!";
let embedding = embedder.embed_text(text).await?;

// Batch embedding generation
let texts = vec!["text1", "text2", "text3"];
let embeddings = embedder.embed_batch(texts).await?;

// Calculate similarity
let similarity = embedder.cosine_similarity(&embedding1, &embedding2)?;

// Semantic search
let results = embedder.semantic_search(
    query_embedding,
    &candidate_embeddings,
    5
).await?;
```

## Dependencies

- Machine learning libraries for embedding generation
- Vector math libraries for similarity calculations
- Async runtime support

## Design Philosophy

- **Performance**: Optimized for high-throughput embedding generation
- **Flexibility**: Support for multiple embedding models and algorithms
- **Accuracy**: High-quality embeddings for semantic understanding
- **Scalability**: Batch processing and efficient storage

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 