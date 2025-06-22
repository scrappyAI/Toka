//! Vault Hash – causal hashing utilities.
//!
//! The hashing algorithm is deliberately simple and deterministic:
//! `digest = Blake3(payload_bytes || parent_digest_1 || parent_digest_2 …)`.
//!
//! This property enables:
//! * **Content addressability** – identical payload + parent set → identical digest.
//! * **Deduplication** – payload bytes are stored once per digest.
//! * **Causal conflict detection** – mismatch implies divergent ancestry.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use blake3::Hasher;
use vault_core::CausalDigest;

/// Compute a causal Blake3 digest of the given payload and parent digests.
pub fn causal_hash(payload: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = Hasher::new();
    hasher.update(payload);
    for d in parent_digests {
        hasher.update(d);
    }
    *hasher.finalize().as_bytes()
} 