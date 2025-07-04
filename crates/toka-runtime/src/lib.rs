#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-runtime** – Runtime adapter for Toka OS.
//!
//! This crate provides the configuration and runtime management layer that
//! bridges the deterministic kernel with storage backends and other fuzzy
//! components. It handles the lifecycle of kernel instances, storage
//! persistence, and provides convenient APIs for building Toka applications.
//!
//! The runtime sits above the deterministic core and coordinates between:
//! - The kernel (deterministic state machine)
//! - Storage backends (event persistence)
//! - Event bus (real-time notifications)
//! - Authentication (capability validation)

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{info, debug, error as log_error};

use toka_types::Message;
use toka_auth::TokenValidator;
use toka_bus_core::{EventBus, InMemoryBus, KernelEvent};
use toka_kernel::{Kernel, WorldState};
use toka_store_core::StorageBackend;

#[cfg(feature = "memory-storage")]
use toka_store_memory::MemoryBackend;

#[cfg(feature = "sled-storage")]
use toka_store_sled::SledBackend;

#[cfg(feature = "sqlite-storage")]
use toka_store_sqlite::SqliteBackend;

//─────────────────────────────
//  Configuration
//─────────────────────────────

/// Configuration for the Toka runtime.
///
/// This struct contains all the settings needed to configure and start
/// a Toka runtime instance, including storage backend selection,
/// bus settings, and operational parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Event bus ring buffer capacity
    pub bus_capacity: usize,
    /// Storage backend configuration
    pub storage: StorageConfig,
    /// Whether to spawn the kernel in a separate task
    pub spawn_kernel: bool,
    /// Maximum number of events to buffer for persistence
    pub persistence_buffer_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            bus_capacity: 1024,
            storage: StorageConfig::Memory,
            spawn_kernel: false,
            persistence_buffer_size: 256,
        }
    }
}

/// Storage backend configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageConfig {
    /// In-memory storage (non-persistent)
    Memory,
    /// Sled-based persistent storage
    #[cfg(feature = "sled-storage")]
    Sled { 
        /// Database file path
        path: String 
    },
    /// SQLite-based persistent storage
    #[cfg(feature = "sqlite-storage")]
    Sqlite {
        /// Database file path
        path: String
    },
}

//─────────────────────────────
//  Runtime context
//─────────────────────────────

/// A complete Toka runtime instance.
///
/// This struct encapsulates a configured Toka system including the kernel,
/// storage backend, event bus, and authentication. It provides the main
/// entry point for interacting with the Toka system.
pub struct Runtime {
    kernel: Arc<Kernel>,
    storage: Arc<dyn StorageBackend>,
    bus: Arc<dyn EventBus>,
    _persistence_task: Option<tokio::task::JoinHandle<()>>,
}

impl Runtime {
    /// Create a new runtime with the given configuration and auth validator.
    ///
    /// This will initialize all components according to the configuration
    /// and optionally start background tasks for event persistence.
    pub async fn new(
        config: RuntimeConfig,
        auth: Arc<dyn TokenValidator>,
    ) -> Result<Self> {
        info!("Initializing Toka runtime with config: {:?}", config);

        // Create event bus
        let bus: Arc<dyn EventBus> = Arc::new(InMemoryBus::new(config.bus_capacity));
        debug!("Created event bus with capacity {}", config.bus_capacity);

        // Create storage backend
        let storage = Self::create_storage_backend(&config.storage).await?;
        debug!("Created storage backend: {:?}", config.storage);

        // Create kernel
        let kernel = Arc::new(Kernel::new(
            WorldState::default(),
            auth,
            bus.clone(),
        ));
        debug!("Created kernel with default world state");

        // Start persistence task if configured
        let persistence_task = if config.persistence_buffer_size > 0 {
            Some(Self::spawn_persistence_task(
                bus.clone(),
                storage.clone(),
                config.persistence_buffer_size,
            ).await?)
        } else {
            None
        };

        info!("Toka runtime initialized successfully");

        Ok(Self {
            kernel,
            storage,
            bus,
            _persistence_task: persistence_task,
        })
    }

    /// Create a storage backend based on configuration.
    async fn create_storage_backend(config: &StorageConfig) -> Result<Arc<dyn StorageBackend>> {
        match config {
            #[cfg(feature = "memory-storage")]
            StorageConfig::Memory => {
                debug!("Creating in-memory storage backend");
                Ok(Arc::new(MemoryBackend::new()))
            }
            #[cfg(feature = "sled-storage")]
            StorageConfig::Sled { path } => {
                debug!("Creating sled storage backend at path: {}", path);
                let backend = SledBackend::open(path)?;
                Ok(Arc::new(backend))
            }
            #[cfg(feature = "sqlite-storage")]
            StorageConfig::Sqlite { path } => {
                debug!("Creating SQLite storage backend at path: {}", path);
                let backend = SqliteBackend::open(path).await?;
                Ok(Arc::new(backend))
            }
            #[cfg(not(feature = "memory-storage"))]
            StorageConfig::Memory => {
                Err(anyhow::anyhow!("Memory storage feature not enabled"))
            }
        }
    }

    /// Spawn a background task that persists events from the bus to storage.
    async fn spawn_persistence_task(
        bus: Arc<dyn EventBus>,
        storage: Arc<dyn StorageBackend>,
        buffer_size: usize,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let mut rx = bus.subscribe();
        
        let task = tokio::spawn(async move {
            debug!("Starting persistence task with buffer size {}", buffer_size);
            
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = Self::persist_event(&*storage, &event).await {
                            log_error!("Failed to persist event {:?}: {}", event, e);
                        } else {
                            debug!("Persisted event: {:?}", event);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        log_error!("Persistence task lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Event bus closed, stopping persistence task");
                        break;
                    }
                }
            }
        });

        Ok(task)
    }

    /// Persist a single kernel event to storage.
    async fn persist_event(_storage: &dyn StorageBackend, _event: &KernelEvent) -> Result<()> {
        // For now, we don't persist kernel events directly since they don't have
        // the full event header structure. In a full implementation, this would
        // convert kernel events to storage events and persist them.
        // This is intentionally simplified for the refactor.
        debug!("Event persistence not yet implemented");
        Ok(())
    }

    /// Submit a message to the kernel for processing.
    ///
    /// This is the main entry point for interacting with the Toka system.
    /// The message will be validated, processed by the kernel, and result
    /// in events being emitted on the bus.
    pub async fn submit(&self, message: Message) -> Result<KernelEvent> {
        debug!("Submitting message from entity {:?}", message.origin);
        self.kernel.submit(message).await
    }

    /// Get a reference to the kernel's world state.
    ///
    /// This provides read-only access to the current system state for
    /// queries and introspection.
    pub fn world_state(&self) -> Arc<tokio::sync::RwLock<WorldState>> {
        self.kernel.state_ptr()
    }

    /// Subscribe to the live event stream.
    ///
    /// Returns a receiver that will receive all events emitted by the kernel
    /// as they happen. Useful for real-time monitoring and reactive systems.
    pub fn subscribe(&self) -> broadcast::Receiver<KernelEvent> {
        self.bus.subscribe()
    }

    /// Get a reference to the storage backend.
    ///
    /// This allows direct access to the storage layer for advanced use cases
    /// that need to query or manipulate stored events directly.
    pub fn storage(&self) -> Arc<dyn StorageBackend> {
        self.storage.clone()
    }

    /// Get a reference to the event bus.
    ///
    /// This provides access to the event bus for publishing custom events
    /// or creating additional subscriptions.
    pub fn bus(&self) -> Arc<dyn EventBus> {
        self.bus.clone()
    }

    /// Shutdown the runtime gracefully.
    ///
    /// This will stop background tasks and ensure all pending operations
    /// are completed before returning.
    pub async fn shutdown(self) -> Result<()> {
        info!("Shutting down Toka runtime");

        if let Some(task) = self._persistence_task {
            task.abort();
            let _ = task.await;
        }

        info!("Toka runtime shutdown complete");
        Ok(())
    }
}

//─────────────────────────────
//  Convenience functions
//─────────────────────────────

/// Run a Toka runtime with the given configuration and authentication.
///
/// This is a high-level convenience function that creates and runs a runtime
/// instance. It's suitable for simple applications that don't need fine-grained
/// control over the runtime lifecycle.
pub async fn run(
    config: RuntimeConfig,
    auth: Arc<dyn TokenValidator>,
) -> Result<Runtime> {
    Runtime::new(config, auth).await
}

/// Create a runtime configuration for testing purposes.
///
/// This returns a configuration suitable for unit tests and integration tests,
/// using in-memory storage and small buffer sizes.
pub fn test_config() -> RuntimeConfig {
    RuntimeConfig {
        bus_capacity: 16,
        storage: StorageConfig::Memory,
        spawn_kernel: false,
        persistence_buffer_size: 0, // Disable persistence for tests
    }
}

//─────────────────────────────
//  Error types
//─────────────────────────────

/// Errors that can occur during runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// Configuration error
    #[error("runtime configuration error: {0}")]
    Configuration(String),
    /// Storage backend error
    #[error("storage backend error: {0}")]
    Storage(String),
    /// Kernel operation error
    #[error("kernel operation error: {0}")]
    Kernel(String),
    /// Authentication error
    #[error("authentication error: {0}")]
    Authentication(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use toka_auth::{Claims, TokenValidator};
    use toka_types::{Operation, TaskSpec};
    // Removed unused import

    #[derive(Clone, Debug)]
    struct TestValidator;

    #[async_trait]
    impl TokenValidator for TestValidator {
        async fn validate(&self, raw: &str) -> toka_auth::Result<Claims> {
            Ok(Claims {
                sub: raw.to_string(),  // Use the token as the subject for testing
                vault: "test".into(),
                permissions: vec![],
                iat: 0,
                exp: u64::MAX,
                jti: "test".into(),
            })
        }
    }

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = test_config();
        let auth = Arc::new(TestValidator);

        let runtime = Runtime::new(config, auth).await.unwrap();
        // Just verify we can get references to storage and bus
        let _storage = runtime.storage();
        let _bus = runtime.bus();
    }

    #[tokio::test]
    async fn test_message_submission() {
        let config = test_config();
        let auth = Arc::new(TestValidator);
        let runtime = Runtime::new(config, auth).await.unwrap();

        let entity = toka_types::EntityId(42);
        let message = Message {
            origin: entity,
            capability: "42".to_string(),  // Token that matches EntityId(42)
            op: Operation::ScheduleAgentTask {
                agent: entity,
                task: TaskSpec {
                    description: "test task".to_string(),
                },
            },
        };

        let event = runtime.submit(message).await.unwrap();
        
        // Verify the event is correct
        match event {
            KernelEvent::TaskScheduled { agent, .. } => {
                assert_eq!(agent, entity);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = test_config();
        let auth = Arc::new(TestValidator);
        let runtime = Runtime::new(config, auth).await.unwrap();

        let mut rx = runtime.subscribe();

        let entity = toka_types::EntityId(123);
        let message = Message {
            origin: entity,
            capability: "123".to_string(),  // Token that matches EntityId(123)
            op: Operation::ScheduleAgentTask {
                agent: entity,
                task: TaskSpec {
                    description: "subscription test".to_string(),
                },
            },
        };

        // Submit message
        let _event = runtime.submit(message).await.unwrap();

        // Should receive event via subscription
        let received_event = rx.recv().await.unwrap();
        match received_event {
            KernelEvent::TaskScheduled { agent, .. } => {
                assert_eq!(agent, entity);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_world_state_access() {
        let config = test_config();
        let auth = Arc::new(TestValidator);
        let runtime = Runtime::new(config, auth).await.unwrap();

        let entity = toka_types::EntityId(999);
        let task = TaskSpec {
            description: "state test".to_string(),
        };

        let message = Message {
            origin: entity,
            capability: "999".to_string(),  // Token that matches EntityId(999)
            op: Operation::ScheduleAgentTask {
                agent: entity,
                task: task.clone(),
            },
        };

        // Submit message to modify state
        let _event = runtime.submit(message).await.unwrap();

        // Check world state was updated
        let state = runtime.world_state();
        let state_guard = state.read().await;
        let agent_tasks = state_guard.agent_tasks.get(&entity).unwrap();
        assert_eq!(agent_tasks.len(), 1);
        assert_eq!(agent_tasks[0], task);
    }

    #[tokio::test]
    async fn test_runtime_shutdown() {
        let config = test_config();
        let auth = Arc::new(TestValidator);
        let runtime = Runtime::new(config, auth).await.unwrap();

        // Should shutdown without errors
        runtime.shutdown().await.unwrap();
    }
}