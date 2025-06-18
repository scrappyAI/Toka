//! # Vaults
//!
//! Defines the `Vault` data structure, a secure container for user and agent data.
//!
//! Vaults are the primary mechanism for storing contextual information that agents
//! can operate on. They are owned by a user and can have specific access controls
//! for different agents or services.

use crate::ids::{AgentID, UserID, VaultID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a secure storage container for data.
///
/// A Vault holds arbitrary data in a key-value store (`data`) and maintains
/// metadata about its creation, ownership, and access controls.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    /// A unique identifier for the vault.
    pub id: VaultID,

    /// The ID of the user who owns this vault.
    pub owner_id: UserID,

    /// A user-friendly name for the vault.
    pub name: String,

    /// A description of the vault's purpose or contents.
    #[serde(default)]
    pub description: Option<String>,

    /// The main data store for the vault, holding arbitrary contextual data.
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,

    /// A list of agents who have been granted access to this vault.
    #[serde(default)]
    pub authorized_agents: Vec<AgentID>,

    /// The timestamp when the vault was created.
    pub created_at: DateTime<Utc>,

    /// The timestamp of the last update to the vault.
    pub updated_at: DateTime<Utc>,
}
