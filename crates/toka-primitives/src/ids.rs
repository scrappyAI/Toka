//! # Toka Core Identifiers
//!
//! This module provides a unified and type-safe system for handling various
//! identifiers used throughout the Toka platform. It is designed to prevent
//! accidental misuse of one ID type for another (e.g., using a `UserID` where
//! a `VaultID` is expected).
//!
//! The implementation is copied from the original `toka-core` crate but lives
//! here to keep `toka-primitives` agnostic to higher-level business logic.

use serde::{de::Visitor, Deserializer, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use uuid::Uuid;

// --- Core ID Structure ----------------------------------------------------

/// A generic, type-safe identifier.
///
/// `Id<T>` is a wrapper around a `Uuid` that is tagged with a zero-sized
/// marker type `T` to ensure type safety at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id<T> {
    uuid: Uuid,
    _marker: PhantomData<T>,
}

/// A trait that defines the prefix for a specific kind of ID.
pub trait IdKind {
    /// The prefix for the ID type (e.g., "user", "vault").
    const PREFIX: &'static str;
}

impl<T: IdKind> Id<T> {
    /// Creates a new ID with a random UUID.
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    /// Creates an ID from an existing `Uuid` (used by tests and fixtures).
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying `Uuid` reference.
    pub fn as_uuid(&self) -> &Uuid {
        &self.uuid
    }
}

// --- Marker Structs -------------------------------------------------------

/// Marker for User IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserMarker;
/// Marker for Agent IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentMarker;
/// Marker for Model IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelMarker;
/// Marker for Transaction IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionMarker;
/// Marker for Resource IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceMarker;
/// Marker for Product IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProductMarker;
/// Marker for Vault IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VaultMarker;

// --- ID Kind Implementations ---------------------------------------------

impl IdKind for UserMarker {
    const PREFIX: &'static str = "user";
}
impl IdKind for AgentMarker {
    const PREFIX: &'static str = "agnt";
}
impl IdKind for ModelMarker {
    const PREFIX: &'static str = "mdl";
}
impl IdKind for TransactionMarker {
    const PREFIX: &'static str = "txn";
}
impl IdKind for ResourceMarker {
    const PREFIX: &'static str = "rsrc";
}
impl IdKind for ProductMarker {
    const PREFIX: &'static str = "prod";
}
impl IdKind for VaultMarker {
    const PREFIX: &'static str = "vlt";
}

// --- Type Aliases ---------------------------------------------------------

/// Unique identifier for users in the system
pub type UserID = Id<UserMarker>;
/// Unique identifier for agents in the system
pub type AgentID = Id<AgentMarker>;
/// Unique identifier for models in the system
pub type ModelID = Id<ModelMarker>;
/// Unique identifier for transactions in the system
pub type TransactionID = Id<TransactionMarker>;
/// Unique identifier for resources in the system
pub type ResourceID = Id<ResourceMarker>;
/// Unique identifier for products in the system
pub type ProductID = Id<ProductMarker>;
/// Unique identifier for vaults in the system
pub type VaultID = Id<VaultMarker>;

// --- Boilerplate Conversions & Serialization -----------------------------

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }
}

impl<T> From<Uuid> for Id<T> {
    fn from(uuid: Uuid) -> Self {
        Self {
            uuid,
            _marker: PhantomData,
        }
    }
}

impl<T> From<Id<T>> for Uuid {
    fn from(id: Id<T>) -> Self {
        id.uuid
    }
}

impl<T: IdKind> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", T::PREFIX, self.uuid)
    }
}

impl<T: IdKind> FromStr for Id<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Support both `prefix_uuid` and bare UUID strings for compatibility
        if let Some((prefix, uuid_part)) = s.split_once('_') {
            if prefix != T::PREFIX {
                anyhow::bail!("Invalid prefix: expected '{}', got '{}'", T::PREFIX, prefix);
            }
            let uuid = Uuid::from_str(uuid_part)
                .map_err(|e| anyhow::anyhow!("Invalid UUID part in ID: {}", e))?;
            return Ok(Self {
                uuid,
                _marker: PhantomData,
            });
        }
        // Fallback: treat entire string as UUID without prefix
        let uuid = Uuid::from_str(s).map_err(|e| anyhow::anyhow!("Invalid UUID string: {}", e))?;
        Ok(Self {
            uuid,
            _marker: PhantomData,
        })
    }
}

impl<T: IdKind> serde::Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T: IdKind> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor<V>(PhantomData<V>);

        impl<'de, V: IdKind> Visitor<'de> for IdVisitor<V> {
            type Value = Id<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format 'prefix_uuid'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Id::from_str(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(IdVisitor(PhantomData))
    }
}
