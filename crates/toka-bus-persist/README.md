# Toka Bus Persist

Persistent event bus implementation for the Toka platform.

## Overview

This crate provides a persistent event bus implementation that stores events in a database for durability and replay capabilities. It's designed for production systems that require event persistence and historical event access.

## Features

- Persistent event storage with database backend
- Event replay and historical access
- Transactional event publishing
- Event filtering and querying
- Async/await support with Tokio
- Database migration support

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-bus-persist = "0.1.0"
```

### Example

```rust
use toka_bus_persist::PersistentEventBus;
use toka_events_core::Event;

// Initialize with database connection
let bus = PersistentEventBus::new(database_url).await?;

// Publish an event (automatically persisted)
bus.publish(Event::new("agent.created", payload)).await?;

// Subscribe to events (includes historical events)
let mut subscriber = bus.subscribe("agent.*").await?;

// Replay events from a specific point in time
let events = bus.replay_events("agent.*", since_timestamp).await?;
```

## Dependencies

- SQLx for database operations
- PostgreSQL as the primary database backend
- Tokio for async runtime support

## Design Philosophy

- **Durability**: All events are persisted to ensure no data loss
- **Replayability**: Historical events can be replayed for analysis
- **Scalability**: Designed to handle high-volume event streams
- **Reliability**: Transactional guarantees for event publishing

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 