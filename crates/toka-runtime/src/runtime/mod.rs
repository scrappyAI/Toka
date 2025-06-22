// Runtime module

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use toka_security_vault::blob_adapter::VaultBlobAdapter;
use toka_storage::{LocalFsAdapter, StorageAdapter};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::agents::Agent;
use crate::agents::SymbolicAgent;
use crate::events::{EventBus, EventType};
use crate::tools::ToolRegistry;
use crate::vault::{Vault, VaultEntry};

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub vault_path: String,
    pub max_agents: usize,
    pub event_buffer_size: usize,
    pub storage_root: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            vault_path: "runtime_data".to_string(),
            max_agents: 100,
            event_buffer_size: 1000,
            storage_root: dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".toka/storage")
                .to_string_lossy()
                .into_owned(),
        }
    }
}

/// Core runtime structure that manages agents and system state
pub struct Runtime {
    config: RuntimeConfig,
    agents: Arc<RwLock<HashMap<String, Box<dyn Agent + Send + Sync>>>>,
    event_bus: Arc<Mutex<EventBus>>,
    event_tx: broadcast::Sender<(String, String)>,
    vault: Arc<Vault>,
    #[allow(dead_code)]
    tool_registry: Arc<ToolRegistry>,
    storage_adapters: Arc<RwLock<HashMap<String, Arc<dyn StorageAdapter>>>>,
    is_running: Arc<Mutex<bool>>,
}

impl Runtime {
    /// Create a new runtime instance with the given configuration
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        let vault = Vault::new(&config.vault_path)
            .with_context(|| format!("Failed to initialize vault at {}", config.vault_path))?;
        let vault = Arc::new(vault);

        let event_bus = EventBus::new(config.event_buffer_size);
        let (event_tx, _) = broadcast::channel(config.event_buffer_size);

        let tool_registry = ToolRegistry::new();

        // ── Storage adapters ───────────────────────────────────────────────
        let mut adapters: HashMap<String, Arc<dyn StorageAdapter>> = HashMap::new();
        let blob_adapter = VaultBlobAdapter::new(vault.clone());
        adapters.insert("vault".into(), Arc::new(blob_adapter));

        // Legacy local filesystem adapter for backwards-compat
        let local_adapter = LocalFsAdapter::new(&config.storage_root)
            .with_context(|| format!("Failed to init local storage at {}", config.storage_root))?;
        adapters.insert("local".into(), Arc::new(local_adapter));

        let runtime = Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Arc::new(Mutex::new(event_bus)),
            event_tx,
            vault: vault.clone(),
            tool_registry: Arc::new(tool_registry),
            storage_adapters: Arc::new(RwLock::new(adapters)),
            is_running: Arc::new(Mutex::new(false)),
        };

        runtime.load_state().await?;

        Ok(runtime)
    }

    /// Start the runtime
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }

        *is_running = true;
        info!("Runtime started");

        // Start event processing loop
        let event_tx = self.event_tx.clone();
        let _event_bus = self.event_bus.clone(); // Keep for future use
        let agents = self.agents.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut event_rx = event_tx.subscribe();

            while *is_running.lock().await {
                match event_rx.recv().await {
                    Ok((event_type, event_data)) => {
                        let mut agents = agents.write().await;
                        for agent in agents.values_mut() {
                            if let Err(e) = agent.process_event(&event_type, &event_data).await {
                                error!(
                                    "Agent {} failed to process event {}: {}",
                                    agent.name(),
                                    event_type,
                                    e
                                );
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Event processing lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the runtime
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if !*is_running {
            return Ok(());
        }

        *is_running = false;
        info!("Runtime stopped");
        Ok(())
    }

    /// Register a new agent with the runtime
    pub async fn register_agent(&self, agent: Box<dyn Agent + Send + Sync>) -> Result<String> {
        let agent_id = Uuid::new_v4().to_string();
        let mut agents = self.agents.write().await;

        if agents.len() >= self.config.max_agents {
            return Err(anyhow::anyhow!("Maximum number of agents reached"));
        }

        agents.insert(agent_id.clone(), agent);
        info!("Registered new agent: {}", agent_id);

        Ok(agent_id)
    }

    /// Remove an agent from the runtime
    pub async fn remove_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        if agents.remove(agent_id).is_some() {
            info!("Removed agent: {}", agent_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }

    /// Get a list of all registered agent IDs
    pub async fn list_agents(&self) -> Vec<String> {
        self.agents.read().await.keys().cloned().collect()
    }

    /// Get access to the tool registry
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    /// Emit an event to all agents
    pub async fn emit_event(&self, event_type: String, data: String) -> Result<()> {
        // Send to broadcast channel for agent processing
        self.event_tx.send((event_type.clone(), data.clone()))?;

        // Also send to event bus for external listeners
        let event_bus = self.event_bus.lock().await;
        let generic_event = EventType::Generic { event_type, data };
        event_bus.emit(generic_event, "runtime-cli").await?;

        Ok(())
    }

    /// Save runtime state to vault
    pub async fn save_state(&self) -> Result<()> {
        // NOTE: Serialising trait objects requires additional machinery (`typetag`) which we
        // deliberately avoid in the minimal runtime build.  Instead we persist a *summary* of
        // current agent IDs.  This leaves room for richer persistence strategies behind a
        // future feature flag without breaking object-safety.

        let agent_ids: Vec<String> = {
            let agents = self.agents.read().await;
            agents.keys().cloned().collect()
        };

        let entry = VaultEntry {
            key: "runtime_state".to_string(),
            data: serde_json::to_string(&agent_ids)?,
            metadata: crate::vault::VaultMetadata {
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                version: 1,
            },
        };

        self.vault.insert(&entry).await?;

        // Persist each agent full state.
        let agents = self.agents.read().await;
        for agent in agents.values() {
            let _ = agent.save_state(&*self.vault).await;
        }
        Ok(())
    }

    /// Load runtime state from vault
    pub async fn load_state(&self) -> Result<()> {
        // The minimal build restores *only* the list of agent IDs (without their full state).
        // This is sufficient for basic CLI inspection commands.
        if let Some(entry) = self.vault.get("runtime_state").await? {
            let agent_ids: Vec<String> = serde_json::from_str(&entry.data)
                .context("Failed to deserialize runtime agent summary")?;

            let mut agent_map = self.agents.write().await;
            for id in agent_ids {
                agent_map.entry(id).or_insert_with(|| {
                    Box::new(SymbolicAgent::new("restored")) as Box<dyn Agent + Send + Sync>
                });
            }

            info!("Loaded {} agent placeholders from vault", agent_map.len());

            // attempt to load full state for each
            for (id, agent) in agent_map.iter_mut() {
                let _ = agent.load_state(&*self.vault).await;
            }
        }
        Ok(())
    }

    /// Returns whether the runtime is currently running.
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// Access a storage adapter by scheme (`local`, `s3`, …).
    pub async fn storage(&self, scheme: &str) -> Option<Arc<dyn StorageAdapter>> {
        self.storage_adapters.read().await.get(scheme).cloned()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Ensure runtime is stopped
        if let Err(e) = futures::executor::block_on(self.stop()) {
            error!("Failed to stop runtime during drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::SymbolicAgent;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_runtime_lifecycle() -> Result<()> {
        let temp_dir = tempdir()?;
        let config = RuntimeConfig {
            vault_path: temp_dir.path().to_str().unwrap().to_string(),
            max_agents: 10,
            event_buffer_size: 100,
            storage_root: dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".toka/storage")
                .to_string_lossy()
                .into_owned(),
        };

        let runtime = Runtime::new(config).await?;

        // Test start/stop
        runtime.start().await?;
        assert!(*runtime.is_running.lock().await);

        runtime.stop().await?;
        assert!(!*runtime.is_running.lock().await);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_management_and_persistence() -> Result<()> {
        // Create a temporary directory for the vault
        let temp_dir = tempdir()?;
        let vault_path = temp_dir.path().to_str().unwrap().to_string();

        // Create first runtime instance
        let config1 = RuntimeConfig {
            vault_path: vault_path.clone(),
            max_agents: 2,
            event_buffer_size: 100,
            storage_root: dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".toka/storage")
                .to_string_lossy()
                .into_owned(),
        };

        let runtime1 = Arc::new(Runtime::new(config1).await?);

        // Test agent registration
        let agent = Box::new(SymbolicAgent::new("test_agent"));
        let agent_id = runtime1.register_agent(agent).await?;
        assert_eq!(runtime1.list_agents().await.len(), 1);

        // Save state
        runtime1.save_state().await?;

        // Stop the first runtime and drop it
        runtime1.stop().await?;
        drop(runtime1); // Explicitly drop to ensure vault is closed

        // Small delay to ensure resources are released
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create second runtime instance with the same vault path
        let config2 = RuntimeConfig {
            vault_path,
            max_agents: 2,
            event_buffer_size: 100,
            storage_root: dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".toka/storage")
                .to_string_lossy()
                .into_owned(),
        };

        let runtime2 = Runtime::new(config2).await?;

        // Verify the agent was loaded from the vault
        let agents = runtime2.list_agents().await;
        assert_eq!(agents.len(), 1);
        assert!(agents.contains(&agent_id));

        // Test agent removal
        runtime2.remove_agent(&agent_id).await?;
        assert_eq!(runtime2.list_agents().await.len(), 0);

        Ok(())
    }
}
