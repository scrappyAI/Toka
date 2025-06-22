# Toka Vaults

This crate implements the "Vault OS" layer for Toka – a causal, content-addressed event store with semantic intent clustering. The vault serves as the canonical source of truth for all agent interactions and system events.

## Core Concepts

### Event-Driven Architecture

Instead of agents maintaining their own private state, all system interactions flow through the vault:

- **Agents** emit events to the vault and subscribe to relevant event streams
- **Events** are immutable, causally-linked records with semantic embeddings
- **Intents** are automatically discovered clusters of semantically similar events

### Causal Hashing

Every event is content-addressed using Blake3 hashing that includes:
- The event payload (serialized as MessagePack)
- The causal digests of all parent events

This enables:
- **Deduplication**: Identical events are stored only once
- **Integrity**: Tampering with history is detectable
- **Replay**: Events can be deterministically reconstructed

### Intent Clustering

Events are automatically grouped into semantic clusters using:
- Dense vector embeddings (typically 768-dimensional)
- Online cosine similarity clustering
- Adaptive centroid updates

## Usage

### Basic Event Storage

```rust
use toka_ledger::{VaultBus, EventPayload};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserLogin {
    user_id: String,
    timestamp: u64,
}

async fn example() -> anyhow::Result<()> {
    let vault = VaultBus::open("./vault-data")?;
    
    // Simple embedding (in practice, use smart-embedder crate)
    let embedding = ndarray::arr1(&[0.1, 0.5, -0.2]);
    
    let header = vault.commit(
        &UserLogin {
            user_id: "alice".to_string(),
            timestamp: 1234567890,
        },
        &[], // No parent events
        "auth.login",
        embedding
    ).await?;
    
    println!("Event committed: {}", header.id);
    Ok(())
}
```

### Event Streaming

```rust
use tokio_stream::StreamExt;

async fn watch_events(vault: &VaultBus) -> anyhow::Result<()> {
    let mut stream = vault.subscribe();
    
    while let Ok(header) = stream.recv().await {
        println!("New event: {} (kind: {})", header.id, header.kind);
    }
    
    Ok(())
}
```

### Causal Dependencies

```rust
// Events can reference parent events to maintain causality
let parent_event = vault.commit(&initial_state, &[], "init", embedding1).await?;

let dependent_event = vault.commit(
    &state_update,
    &[parent_event], // This event depends on the parent
    "update",
    embedding2
).await?;
```

## Architecture

The crate is organized into several modules:

- **`core`**: Fundamental types (`EventHeader`, `EventPayload`, etc.)
- **`hash`**: Causal hashing utilities using Blake3
- **`intent`**: Online clustering for semantic event grouping
- **`bus`**: Main `VaultBus` with RocksDB storage and live streaming

## Storage

Events are stored in two RocksDB databases:
- **Payloads**: `digest → serialized_payload` (content-addressed)
- **Headers**: `event_id → event_header` (for fast lookups)

This design enables efficient deduplication while maintaining fast access patterns.

## Integration

The vault integrates seamlessly with:
- **`smart-embedder`**: For generating semantic embeddings from structured events
- **`toka-agents`**: Agents can commit events and subscribe to streams
- **`toka-runtime`**: The runtime can replay event history for debugging

## Performance Characteristics

- **Write throughput**: Limited by RocksDB write performance (~10K-100K events/sec)
- **Storage efficiency**: Deduplication reduces storage for repeated events
- **Intent clustering**: O(k) where k is the number of existing clusters
- **Memory usage**: Intent centroids kept in-memory (typically <1MB for 1000 clusters) 