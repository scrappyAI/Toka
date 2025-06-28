# toka-bus-api

`toka-bus-api` — minimal, **no_std-friendly** trait contracts and data types
for the Toka **in-process event bus**.

This crate purposefully ships **zero** heavy dependencies; consumers can
enable the following *opt-in* features:

| Feature | Enables | Notes |
|---------|---------|-------|
| `serde-support` | `serde::{Serialize, Deserialize}` impls on all public
                   types | Keeps default build lean if you only work with
                   opaque headers. |
| `async` | • `async_trait` for the runtime traits
          | • `anyhow` for ergonomic `Result` types | Activates the async
          APIs; implies you are in a `tokio` (or compatible) env. |
| `tokio` | `tokio::sync::broadcast::Receiver` in the `EventBus` API | Pulled
           in automatically by `async` but can be enabled standalone. |

### Quick Example
```rust
use toka_bus_api::{prelude::*, AgentEvent};
use anyhow::Result;

// Dummy implementation which just logs.
struct LoggerBus;
#[async_trait::async_trait]
impl EventBus for LoggerBus {
    async fn publish<P: EventPayload + ?Sized>(&self,
        _payload: &P,
        kind: &str) -> Result<BusEventHeader> {
        println!("published {kind}");
        Ok(BusEventHeader::new(kind))
    }
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BusEventHeader> {
        let (tx, rx) = tokio::sync::broadcast::channel(8);
        let _ = tx; // tx would be stored and used by publish
        rx
    }
}
```

See `toka-bus` for the default in-memory implementation.

License: MIT OR Apache-2.0
