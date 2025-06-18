//! Toka Meta-Crate
//!
//! This crate is a convenience layer that re-exports the most frequently used
//! items across the Toka platform.  Think of it as the *standard library* for
//! building on Toka.  If you prefer granular deps, depend on the individual
//! crates instead.

// -----------------------------------------------------------------------------
// Prelude – glob-import to grab common symbols in one line
// -----------------------------------------------------------------------------
#[allow(unused_imports)]
pub mod prelude {
    #[cfg(feature = "auth")]  pub use toka_security_auth::prelude::*;
    #[cfg(feature = "vault")] pub use toka_security_vault::prelude::*;
    #[cfg(feature = "agents")] pub use toka_agents::prelude::*;
    // Future: add toolkit prelude when available.
}

// -----------------------------------------------------------------------------
// Re-export entire crates behind feature flags so users can access full APIs.
// -----------------------------------------------------------------------------
#[cfg(feature = "auth")]  pub use toka_security_auth  as auth;
#[cfg(feature = "vault")] pub use toka_security_vault as vault;
#[cfg(feature = "agents")] pub use toka_agents         as agents;
#[cfg(feature = "toolkit")] pub use toka_toolkit       as toolkit; 