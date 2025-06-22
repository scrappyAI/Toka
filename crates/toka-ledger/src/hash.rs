//! Causal hashing utilities for content-addressed event storage.

use blake3::Hasher;
use crate::core::CausalDigest;

/// Compute a causal Blake3 digest of the given payload and parent digests.
///
/// The algorithm is: `digest = Blake3(payload_bytes || parent_digest_1 || parent_digest_2 …)`
///
/// This enables:
/// - **Content addressability**: identical payload + parent set → identical digest
/// - **Deduplication**: payload bytes stored once per digest
/// - **Causal conflict detection**: digest mismatch implies divergent ancestry
pub fn causal_hash(payload: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = Hasher::new();
    hasher.update(payload);
    for d in parent_digests {
        hasher.update(d);
    }
    *hasher.finalize().as_bytes()
} 