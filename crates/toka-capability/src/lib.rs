#![forbid(unsafe_code)]
//! Toka Capability – v0.2 primitives
//!
//! This crate provides *no-std*-friendly primitives for **capability tokens**
//! that internal Toka services use for authorisation.  The public interface is
//! deliberately small and stable so that downstream crates can swap the
//! underlying crypto or transport without major refactors.
//!
//! ## Versioning
//! This is the **0.2-alpha** rewrite that supersedes the former
//! `toka-security-auth` crate (v0.1).  It aligns with the draft
//! specification in `docs/40_capability_tokens_spec_v0.1.md` and introduces
//! cleaner boundaries ready for future extensions like EdDSA, Biscuit and
//! opaque-token revocation.
//!
//! Differences to v0.1:
//! * Crate renamed to `toka-capability`.
//! * Moved revocation & CVM concerns into sibling crates so this one stays
//!   focused on **creation** and **validation** only.
//! * Internal validation API tightened – no more default clock leeway.
//!
//! ---
//! This crate is memory safe (no `unsafe`) and has **zero required runtime
//! allocations** on the happy path.

pub mod token;
pub mod validator;

pub mod prelude;

pub use prelude::*;

pub use token::CapabilityToken;