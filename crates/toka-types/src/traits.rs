#![allow(missing_docs)]
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Common parameter map type used by tools (key-value string pairs).
pub type Params = HashMap<String, String>;

/// Structured parameter wrapper used by tools and registries. Keeps the
/// original `ToolParams` naming used across the codebase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolParams {
    /// Name of the tool for which the parameters are intended (useful for
    /// logging/audit).
    pub name: String,
    /// Arbitrary key-value argument map.
    #[serde(default)]
    pub args: Params,
}

impl ToolParams {
    /// Borrow the inner map for APIs that expect `Params`.
    pub fn as_map(&self) -> &Params {
        &self.args
    }
}

/// Execution metadata returned by every tool run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolMetadata {
    /// Wall-clock execution duration in milliseconds.
    pub execution_time_ms: u64,
    /// Semantic version of the tool implementation.
    pub tool_version: String,
    /// Unix timestamp when the tool finished execution.
    pub timestamp: u64,
}

/// Standard result wrapper for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    /// Whether execution was successful.
    pub success: bool,
    /// Tool output payload (format defined by tool implementation).
    pub output: String,
    /// Execution metadata including timing, version etc.
    pub metadata: ToolMetadata,
}

/// Core abstraction for executable tools in the Toka ecosystem.
///
/// The trait is intentionally minimal and lives in `toka-types` so it can be
/// shared without creating cyclic dependencies.  All higher-level crates
/// (`toka-tools`, `toka-runtime`, etc.) should depend on *this* definition.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Canonical registry name (snake_case).
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Semantic version string (e.g. "1.2.0").
    fn version(&self) -> &str;

    /// Validate input parameters.
    fn validate_params(&self, params: &ToolParams) -> Result<()>;

    /// Execute the tool.
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;
}

/// Minimal interface for an agent instance that can receive kernel messages.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Called by the runtime when the agent receives a `Message`.
    async fn handle_message(&self, msg: &crate::Message) -> Result<()>;
}

/// Marker trait for resources that can be granted to agents/tools (e.g. file
/// handles, network sockets, GPU contexts).
pub trait Resource: Send + Sync {
    /// Stable identifier of the resource owner / handle.
    fn id(&self) -> crate::EntityId;
}

//─────────────────────────────
//  Capability primitives
//─────────────────────────────

/// Canonical claim set embedded in every capability token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject – usually the user or agent identifier.
    pub sub: String,
    /// Vault / workspace identifier the subject wishes to access.
    pub vault: String,
    /// Ordered list of permission strings (e.g. "transfer").
    pub permissions: Vec<String>,
    /// Issued-at timestamp (seconds since Unix epoch).
    pub iat: u64,
    /// Expiry timestamp (seconds since Unix epoch, must be > `iat`).
    pub exp: u64,
    /// Unique token identifier for replay protection.
    pub jti: String,
}

/// Maximum allowed token lifetime in seconds (24h).
pub const MAX_TOKEN_LIFETIME_SECS: u64 = 86_400;
/// Maximum permission entries per token.
pub const MAX_PERMISSIONS_COUNT: usize = 100;

/// Simple error type used by capability validation logic.
#[derive(Debug)]
pub struct CapabilityError(pub String);

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CapabilityError {}

/// Convenience result alias for capability‐related functions.
pub type CapResult<T> = std::result::Result<T, CapabilityError>;

/// Validate common semantic rules for claims.
impl Claims {
    /// Perform semantic validation of the claim set.
    ///
    /// Ensures lengths, lifetimes and counts are within safe bounds.
    pub fn validate(&self) -> CapResult<()> {
        if self.sub.trim().is_empty() || self.sub.len() > 256 {
            return Err(CapabilityError("Invalid subject identifier".into()));
        }
        if self.vault.trim().is_empty() || self.vault.len() > 256 {
            return Err(CapabilityError("Invalid vault identifier".into()));
        }
        if self.permissions.len() > MAX_PERMISSIONS_COUNT { return Err(CapabilityError("Too many permissions".into())); }
        if self.exp <= self.iat || self.exp - self.iat > MAX_TOKEN_LIFETIME_SECS {
            return Err(CapabilityError("Invalid token lifetime".into()));
        }
        Ok(())
    }
}

/// Capability token trait implemented by concrete formats (JWT, Biscuit…).
#[async_trait]
pub trait CapabilityToken: Sized + Send + Sync {
    async fn mint(claims: &Claims, key: &[u8]) -> CapResult<Self>;
    fn as_str(&self) -> &str;
}

/// Validator trait used by kernel/auth middleware.
#[async_trait]
pub trait TokenValidator: Send + Sync {
    async fn validate(&self, raw: &str) -> CapResult<Claims>;
}