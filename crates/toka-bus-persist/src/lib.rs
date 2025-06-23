//! Persistent event bus implementation for the Toka platform.
//!
//! This crate provides a sled-backed persistent event bus with optional
//! intent clustering capabilities.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sled::Tree;
use tokio::sync::broadcast;
use toka_events_core::{causal_hash, CausalDigest, EventHeader, EventId, EventPayload, IntentId};
use uuid::Uuid;

// Re-export core types for convenience
pub use toka_events_core::{create_event_header, DomainEvent};

/// Default broadcast channel size for live event streaming
const DEFAULT_BROADCAST_SIZE: usize = 256;

// -----------------------------------------------------------------------------
// Intent Strategy Trait
// -----------------------------------------------------------------------------

/// Strategy for assigning intents to events based on embeddings.
///
/// This trait allows different intent clustering algorithms to be plugged in,
/// from simple random assignment to sophisticated semantic clustering.
#[async_trait]
pub trait IntentStrategy: Send + Sync {
    /// Assign an intent ID to an event based on its embedding.
    ///
    /// Returns the assigned intent ID and whether this created a new cluster.
    async fn assign_intent(&self, embedding: &[f32]) -> Result<(IntentId, bool)>;
    
    /// Get the total number of discovered intent clusters.
    async fn cluster_count(&self) -> usize;
}

// -----------------------------------------------------------------------------
// No-Op Intent Strategy (Default)
// -----------------------------------------------------------------------------

/// Default intent strategy that assigns a nil UUID to all events.
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
mod clustering {
    use super::*;
    use ndarray::Array1;
    use parking_lot::RwLock;
    
    /// Dimensionality of the embedding vectors
    pub const EMBEDDING_DIM: usize = 768;
    
    /// Cosine similarity threshold for clustering
    const SIMILARITY_THRESHOLD: f32 = 0.82;
    
    #[derive(Debug, Clone)]
    struct Centroid {
        vector: Array1<f32>,
        count: usize,
        id: IntentId,
    }
    
    /// Online clustering strategy using cosine similarity.
    #[derive(Debug, Default)]
    pub struct OnlineClusterStrategy {
        centroids: RwLock<Vec<Centroid>>,
    }
    
    impl OnlineClusterStrategy {
        /// Create a new online clustering strategy.
        pub fn new() -> Self {
            Self::default()
        }
    }
    
    #[async_trait]
    impl IntentStrategy for OnlineClusterStrategy {
        async fn assign_intent(&self, embedding: &[f32]) -> Result<(IntentId, bool)> {
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

#[cfg(feature = "intent-cluster")]
pub use clustering::OnlineClusterStrategy;

// -----------------------------------------------------------------------------
// Persistent Event Bus
// -----------------------------------------------------------------------------

/// Persistent event bus with sled storage and optional intent clustering.
#[derive(Debug)]
pub struct PersistentEventBus<S: IntentStrategy = NilIntentStrategy> {
    db_payloads: Tree,
    db_headers: Tree,
    broadcast_tx: broadcast::Sender<EventHeader>,
    intent_strategy: S,
}

impl PersistentEventBus<NilIntentStrategy> {
    /// Create a new persistent event bus at the given path without intent clustering.
    pub fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers = db.open_tree("headers")?;
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BROADCAST_SIZE);
        
        Ok(Self {
            db_payloads,
            db_headers,
            broadcast_tx,
            intent_strategy: NilIntentStrategy,
        })
    }
}

impl<S: IntentStrategy> PersistentEventBus<S> {
    /// Create a new persistent event bus with a custom intent strategy.
    pub fn with_intent_strategy(path: &str, strategy: S) -> Result<Self> {
        let db = sled::open(path)?;
        let db_payloads = db.open_tree("payloads")?;
        let db_headers = db.open_tree("headers")?;
        let (broadcast_tx, _) = broadcast::channel(DEFAULT_BROADCAST_SIZE);
        
        Ok(Self {
            db_payloads,
            db_headers,
            broadcast_tx,
            intent_strategy: strategy,
        })
    }
    
    /// Commit an event payload to persistent storage.
    ///
    /// This method:
    /// 1. Serializes the payload using MessagePack
    /// 2. Computes a causal hash from payload and parent digests
    /// 3. Deduplicates payload storage (content-addressed)
    /// 4. Assigns an intent using the configured strategy
    /// 5. Creates and persists the event header
    /// 6. Broadcasts the header to live subscribers
    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind: &str,
        embedding: &[f32],
    ) -> Result<EventHeader> {
        // 1. Serialize payload
        let payload_bytes = rmp_serde::to_vec_named(payload)?;
        
        // 2. Compute causal hash
        let parent_digests: Vec<CausalDigest> = parents.iter().map(|h| h.digest).collect();
        let digest = causal_hash(&payload_bytes, &parent_digests);
        
        // 3. Deduplicate payload storage
        if self.db_payloads.get(&digest)?.is_none() {
            self.db_payloads.insert(&digest, payload_bytes)?;
        }
        
        // 4. Assign intent
        let (intent, _is_new_cluster) = self.intent_strategy.assign_intent(embedding).await?;
        
        // 5. Create event header
        let header = EventHeader {
            id: Uuid::new_v4(),
            parents: parents.iter().map(|h| h.id).collect(),
            timestamp: Utc::now(),
            digest,
            intent,
            kind: kind.to_string(),
        };
        
        // 6. Persist header
        let header_bytes = rmp_serde::to_vec_named(&header)?;
        self.db_headers.insert(header.id.as_bytes(), header_bytes)?;
        
        // 7. Broadcast to live subscribers
        let _ = self.broadcast_tx.send(header.clone());
        
        Ok(header)
    }
    
    /// Subscribe to live event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.broadcast_tx.subscribe()
    }
    
    /// Retrieve an event header by ID.
    pub fn get_header(&self, event_id: &EventId) -> Result<Option<EventHeader>> {
        if let Some(bytes) = self.db_headers.get(event_id.as_bytes())? {
            let header: EventHeader = rmp_serde::from_slice(&bytes)?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }
    
    /// Retrieve an event payload by digest.
    pub fn get_payload<P: EventPayload>(&self, digest: &CausalDigest) -> Result<Option<P>> {
        if let Some(bytes) = self.db_payloads.get(digest)? {
            let payload: P = rmp_serde::from_slice(&bytes)?;
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
    
    /// Get the number of intent clusters discovered.
    pub async fn intent_cluster_count(&self) -> usize {
        self.intent_strategy.cluster_count().await
    }
    
    /// Get the total number of events stored.
    pub fn event_count(&self) -> usize {
        self.db_headers.len()
    }
    
    /// Get the total number of unique payloads stored.
    pub fn payload_count(&self) -> usize {
        self.db_payloads.len()
    }
}

// Convenience type alias for the most common usage
pub type VaultBus = PersistentEventBus<NilIntentStrategy>;

#[cfg(feature = "intent-cluster")]
pub type AgentBus = PersistentEventBus<OnlineClusterStrategy>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use toka_events_core::DomainEvent;
    
    #[tokio::test]
    async fn test_basic_commit_and_retrieve() -> Result<()> {
        let temp_dir = tempdir()?;
        let bus = VaultBus::open(temp_dir.path().to_str().unwrap())?;
        
        let payload = DomainEvent::System {
            component: "test".to_string(),
            payload: serde_json::json!({"msg": "hello"}),
        };
        
        let header = bus.commit(&payload, &[], "system.test", &[]).await?;
        
        // Verify header was created
        assert_eq!(header.kind, "system.test");
        assert_eq!(header.parents.len(), 0);
        assert_eq!(header.intent, Uuid::nil()); // Using NilIntentStrategy
        
        // Verify we can retrieve the header
        let retrieved_header = bus.get_header(&header.id)?;
        assert!(retrieved_header.is_some());
        assert_eq!(retrieved_header.unwrap().id, header.id);
        
        // Verify we can retrieve the payload
        let retrieved_payload: Option<DomainEvent> = bus.get_payload(&header.digest)?;
        assert!(retrieved_payload.is_some());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_subscription() -> Result<()> {
        let temp_dir = tempdir()?;
        let bus = VaultBus::open(temp_dir.path().to_str().unwrap())?;
        
        let mut subscriber = bus.subscribe();
        
        let payload = DomainEvent::System {
            component: "test".to_string(),
            payload: serde_json::json!({"msg": "broadcast"}),
        };
        
        let header = bus.commit(&payload, &[], "system.broadcast", &[]).await?;
        
        // Should receive the event via broadcast
        let received_header = subscriber.recv().await?;
        assert_eq!(received_header.id, header.id);
        assert_eq!(received_header.kind, "system.broadcast");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_causal_relationships() -> Result<()> {
        let temp_dir = tempdir()?;
        let bus = VaultBus::open(temp_dir.path().to_str().unwrap())?;
        
        // Create parent event
        let parent_payload = DomainEvent::System {
            component: "parent".to_string(),
            payload: serde_json::json!({"action": "start"}),
        };
        let parent_header = bus.commit(&parent_payload, &[], "system.start", &[]).await?;
        
        // Create child event
        let child_payload = DomainEvent::System {
            component: "child".to_string(),
            payload: serde_json::json!({"action": "continue"}),
        };
        let child_header = bus.commit(&child_payload, &[parent_header.clone()], "system.continue", &[]).await?;
        
        // Verify causal relationship
        assert_eq!(child_header.parents.len(), 1);
        assert_eq!(child_header.parents[0], parent_header.id);
        
        // Child should have different digest due to different payload and parent
        assert_ne!(child_header.digest, parent_header.digest);
        
        Ok(())
    }
    
    #[cfg(feature = "intent-cluster")]
    #[tokio::test]
    async fn test_intent_clustering() -> Result<()> {
        let temp_dir = tempdir()?;
        let strategy = OnlineClusterStrategy::new();
        let bus = PersistentEventBus::with_intent_strategy(temp_dir.path().to_str().unwrap(), strategy)?;
        
        let payload1 = DomainEvent::Agent {
            agent_id: Uuid::new_v4(),
            payload: serde_json::json!({"action": "think"}),
        };
        
        let payload2 = DomainEvent::Agent {
            agent_id: Uuid::new_v4(),
            payload: serde_json::json!({"action": "reason"}),
        };
        
        // Similar embeddings (should cluster together with high similarity)
        let embedding1 = vec![1.0; 768];
        let embedding2 = vec![0.9; 768]; // Very similar
        
        let header1 = bus.commit(&payload1, &[], "agent.think", &embedding1).await?;
        let header2 = bus.commit(&payload2, &[], "agent.reason", &embedding2).await?;
        
        // Should have some intent clusters
        assert!(bus.intent_cluster_count().await > 0);
        
        Ok(())
    }
}