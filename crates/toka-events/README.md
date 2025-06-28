# toka-events

## Toka Events – Canonical Event Store (renamed from `toka-vault`)

This crate provides the canonical, secure, and persistent event store for the
Toka platform. It is the single source of truth for "what happened" across
the entire system.

### Core Features

*   **Unified API**: A single `Vault` entry point for different backends.
*   **Persistent Storage**: A `sled`-backed, content-addressed store for events.
*   **In-Memory Mode**: A non-persistent, in-memory bus for testing and lightweight scenarios.
*   **Causal Hashing**: Events are linked via a Blake3 causal hash chain, ensuring integrity.
*   **Intent Clustering**: (Optional) Automatically groups events by semantic intent using embeddings.
*   **Live Streaming**: Broadcasts committed event headers to live subscribers.

### Usage

```rust
use toka_events::Vault;
use toka_events::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyEvent {
    id: u32,
    message: String,
}
impl EventPayload for MyEvent {}

// Open a persistent store
let vault = Vault::open_persistent("./my-events-data")?;

// Or use an in-memory vault
let memory_vault = Vault::new_memory();

// Commit an event
let my_payload = MyEvent { id: 1, message: "Hello, Events!".to_string() };
let header = vault.commit(&my_payload, &[], "my.event.kind", &[]).await?;

println!("Committed event with ID: {}", header.id);

// Subscribe to live events
let mut subscriber = vault.subscribe();
while let Ok(header) = subscriber.recv().await {
    println!("Received live event: {:?}", header);
}
```

License: MIT OR Apache-2.0
