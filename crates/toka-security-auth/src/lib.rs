//! Toka Auth Core
//!
//! This crate provides *no-std*-friendly primitives for capability tokens that
//! the runtime and other services can build upon.  It purposefully avoids any
//! heavy crypto or async dependencies.

pub mod token;

pub mod prelude;

pub use prelude::*;

pub use token::CapabilityToken; 