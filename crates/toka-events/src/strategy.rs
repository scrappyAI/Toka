//! Pluggable strategies for assigning semantic intent to events.

use crate::events::IntentId;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Strategy for assigning intents to events based on embeddings.
#[async_trait]
pub trait IntentStrategy: Send + Sync {
    /// Assign an intent ID to an event based on its embedding.
    async fn assign_intent(&self, embedding: &[f32]) -> Result<(IntentId, bool)>;

    /// Get the total number of discovered intent clusters.
    async fn cluster_count(&self) -> usize;
}

// -----------------------------------------------------------------------------
// Nil Intent Strategy (Default)
// -----------------------------------------------------------------------------

/// Default intent strategy that assigns a nil UUID to all events, effectively
/// disabling intent clustering.
#[derive(Debug, Default, Clone)]
pub struct NilIntentStrategy;

#[async_trait]
impl IntentStrategy for NilIntentStrategy {
    async fn assign_intent(&self, _embedding: &[f32]) -> Result<(IntentId, bool)> {
        Ok((Uuid::nil(), false))
    }

    async fn cluster_count(&self) -> usize {
        0
    }
}

// -----------------------------------------------------------------------------
// Online Clustering Intent Strategy (Feature-gated)
// -----------------------------------------------------------------------------

#[cfg(feature = "intent-cluster")]
pub use online_clustering::*;

#[cfg(feature = "intent-cluster")]
mod online_clustering {
    use super::*;
    use ndarray::Array1;
    use parking_lot::RwLock;

    /// Dimensionality of the embedding vectors.
    pub const EMBEDDING_DIM: usize = 384;

    /// Cosine similarity threshold for clustering.
    const SIMILARITY_THRESHOLD: f32 = 0.82;

    #[derive(Debug, Clone)]
    struct Centroid {
        vector: Array1<f32>,
        count: usize,
        id: IntentId,
    }

    /// An online clustering strategy using cosine similarity.
    #[derive(Debug, Default)]
    pub struct OnlineClusterStrategy {
        centroids: RwLock<Vec<Centroid>>,
    }

    impl OnlineClusterStrategy {
        /// Create a new, empty online clustering strategy.
        pub fn new() -> Self {
            Self::default()
        }
    }

    #[async_trait]
    impl IntentStrategy for OnlineClusterStrategy {
        async fn assign_intent(&self, embedding: &[f32]) -> Result<(IntentId, bool)> {
            if embedding.is_empty() {
                return Ok((Uuid::nil(), false));
            }

            let embed_array = Array1::from_vec(embedding.to_vec());
            let mut centroids = self.centroids.write();

            // Find the best matching centroid above threshold
            if let Some((idx, _)) = centroids
                .iter()
                .enumerate()
                .map(|(i, c)| (i, cosine_similarity(&c.vector, &embed_array)))
                .filter(|(_, sim)| *sim > SIMILARITY_THRESHOLD)
                .max_by(|a, b| a.1.total_cmp(&b.1))
            {
                // Update existing centroid (online mean update)
                let centroid = &mut centroids[idx];
                centroid.vector = (&centroid.vector * centroid.count as f32 + &embed_array)
                    / (centroid.count as f32 + 1.0);
                centroid.count += 1;
                return Ok((centroid.id, false));
            }

            // No match found - create new centroid
            let id = Uuid::new_v4();
            centroids.push(Centroid {
                vector: embed_array,
                count: 1,
                id,
            });

            Ok((id, true))
        }

        async fn cluster_count(&self) -> usize {
            self.centroids.read().len()
        }
    }

    fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        let dot_product = a.dot(b);
        let norm_a = a.dot(a).sqrt();
        let norm_b = b.dot(b).sqrt();
        dot_product / (norm_a * norm_b + 1e-12)
    }
} 