# Toka Bus Memory

In-memory event bus implementation for the Toka platform.

## Overview

This crate provides an in-memory event bus implementation that allows components within the Toka platform to communicate asynchronously. It's designed for high-performance, low-latency event processing within a single process.

## Features

- In-memory event publishing and subscription
- Async/await support with Tokio
- Event filtering and routing
- Thread-safe concurrent access
- Lightweight and fast

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-bus-memory = "0.1.0"
```

### Example

```rust
use toka_bus_memory::EventBus;
use toka_events_core::Event;

let bus = EventBus::new();

// Subscribe to events
let mut subscriber = bus.subscribe("agent.*").await?;

// Publish an event
bus.publish(Event::new("agent.created", payload)).await?;

// Receive events
while let Some(event) = subscriber.recv().await {
    println!("Received event: {:?}", event);
}
```

## Design Philosophy

- **Performance First**: Optimized for high-throughput event processing
- **Memory Efficient**: Minimal memory overhead for event storage
- **Thread Safe**: Concurrent access from multiple threads
- **Async Native**: Built on Tokio for modern async Rust applications

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 