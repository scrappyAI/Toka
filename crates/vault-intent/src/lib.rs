//! Vault Intent – online clustering of event embeddings to discover high-level intents.
//!
//! This is a lightweight, lock-free (parking_lot) implementation that keeps
//! centroids in-memory.  It is **not** persisted across runs; callers are
//! expected to checkpoint or derive intent IDs as needed.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use ndarray::Array1;
use parking_lot::RwLock;
use uuid::Uuid;

/// Dimensionality of the embedding vectors. 768 is common for sentence transformers.
pub const D: usize = 768;
/// Cosine similarity threshold at which two embeddings are considered the same intent.
const THRESH: f32 = 0.82;

#[derive(Clone)]
struct Centroid {
    vec:   Array1<f32>,
    count: usize,
    id:    Uuid,
}

/// Thread-safe store of centroids representing discovered intents.
#[derive(Default)]
pub struct IntentStore {
    centroids: RwLock<Vec<Centroid>>,
}

impl IntentStore {
    /// Create a fresh store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assign an embedding to the closest centroid above `THRESH`.
    /// Returns the assigned `IntentId` and a flag indicating whether a new
    /// centroid was created.
    pub fn assign(&self, embed: &Array1<f32>) -> (Uuid, bool) {
        let mut lock = self.centroids.write();

        // Find best matching centroid.
        if let Some((idx, _)) = lock
            .iter()
            .enumerate()
            .map(|(i, c)| (i, cosine(&c.vec, embed)))
            .filter(|(_, sim)| *sim > THRESH)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        {
            // Online centroid update (simple mean update).
            let c = &mut lock[idx];
            c.vec = (&c.vec * c.count as f32 + embed) / (c.count as f32 + 1.0);
            c.count += 1;
            return (c.id, false);
        }

        // No match – create new centroid.
        let id = Uuid::new_v4();
        lock.push(Centroid {
            vec: embed.clone(),
            count: 1,
            id,
        });
        (id, true)
    }
}

fn cosine(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    let dot = a.dot(b);
    dot / (a.norm_l2() * b.norm_l2() + 1e-12)
} 