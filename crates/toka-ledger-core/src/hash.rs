//! Causal hashing utilities for content-addressed event storage.

use blake3::Hasher;
use crate::core::CausalDigest;

/// Compute a causal Blake3 digest of the given payload and parent digests.
///
/// The algorithm is: `digest = Blake3(payload_bytes || parent_digest_1 || …)`
///
/// Guarantees:
/// * identical payload + identical parent set ⇒ identical digest
/// * any divergence in ancestry ⇒ digest mismatch
pub fn causal_hash(payload: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = Hasher::new();
    hasher.update(payload);
    for d in parent_digests {
        hasher.update(d);
    }
    *hasher.finalize().as_bytes()
} 