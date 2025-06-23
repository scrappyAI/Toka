//! Toka Events — ultra-light, in-memory event bus.
//!
//! This crate provides modern event handling capabilities with feature-gated
//! backward compatibility for legacy code.
//!
//! ## Features
//! - `rich` (default): Modern EventBus API with rich event types
//! - `legacy`: Deprecated EventDispatcher trait and InMemoryDispatcher
//!
//! Future crates (e.g. `toka-events-redis`, `toka-events-grpc`) can depend on
//! this crate and provide additional implementations without heavy dependencies.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Legacy components (feature-gated)
#[cfg(feature = "legacy")]
mod dispatcher;
#[cfg(feature = "legacy")]
mod in_memory;

// Core models and modern rich API
mod models;
#[cfg(feature = "rich")]
pub mod rich;

// Conditional exports based on features
#[cfg(feature = "legacy")]
pub use dispatcher::{EventDispatcher, EventError, Subscriber};
#[cfg(feature = "legacy")]
pub use in_memory::InMemoryDispatcher;

pub use models::{Event, EventKind};

#[cfg(feature = "rich")]
pub use rich::*;

// ## Migration Notice
// The `EventDispatcher` trait and `InMemoryDispatcher` are now feature-gated
// behind the "legacy" feature. New code should use the modern `EventBus` API.
