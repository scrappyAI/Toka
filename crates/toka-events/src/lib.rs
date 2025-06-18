//! Toka Events — ultra-light, in-memory event bus.
//!
//! This crate purposefully avoids any network or external storage dependencies.
//! It exposes a small [`EventDispatcher`] trait plus a default
//! [`InMemoryDispatcher`] implementation built on `tokio::sync::broadcast`.
//!
//! Future crates (e.g. `toka-events-redis`, `toka-events-grpc`) can depend on
//! this crate and provide additional dispatcher implementations without
//! burdening `toka-events` with their heavy dependencies.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod dispatcher;
mod in_memory;
mod models;

pub use dispatcher::{EventDispatcher, EventError, Subscriber};
pub use in_memory::InMemoryDispatcher;
pub use models::{Event, EventKind};
