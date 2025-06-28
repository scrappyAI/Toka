#![allow(deprecated)]
#![allow(dead_code)]
// Runtime module

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use toka_storage::{LocalFsAdapter, StorageAdapter};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;
use toka_bus::{MemoryBus, EventBus};
#[cfg(feature = "auth")]
use std::time::Duration;
#[cfg(feature = "auth")]
use crate::security::{self, Envelope};

use crate::agents::Agent;
use crate::agents::SymbolicAgent;
use crate::tools::ToolRegistry;

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
    event_bus: Arc<Mutex<MemoryBus>>,
    event_tx: broadcast::Sender<(String, String)>,
    #[allow(dead_code)]
    tool_registry: Arc<ToolRegistry>,
    storage_adapters: Arc<RwLock<HashMap<String, Arc<dyn StorageAdapter>>>>,
    is_running: Arc<Mutex<bool>>,
    #[cfg(feature = "auth")]
    security: Envelope,
}

impl Runtime {
    /// Create a new runtime instance with the given configuration
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        let event_bus = MemoryBus::new(config.event_buffer_size);
        let (event_tx, _) = broadcast::channel(config.event_buffer_size);

        let tool_registry = ToolRegistry::new();

        // ── Storage adapters ───────────────────────────────────────────────
        let mut adapters: HashMap<String, Arc<dyn StorageAdapter>> = HashMap::new();
        // For now we re-use the local filesystem adapter for the `vault` scheme.
        // Future slices will introduce a dedicated blob adapter living alongside
        // the `toka-vault` crate.
        let vault_fs = LocalFsAdapter::new(&config.vault_path)
            .with_context(|| format!("Failed to init vault storage at {}", config.vault_path))?;
        adapters.insert("vault".into(), Arc::new(vault_fs));

        // Legacy local filesystem adapter for backwards-compat
        let local_adapter = LocalFsAdapter::new(&config.storage_root)
            .with_context(|| format!("Failed to init local storage at {}", config.storage_root))?;
        adapters.insert("local".into(), Arc::new(local_adapter));

        #[cfg(feature = "auth")]
        let security_env = Envelope::initialise(
            security::random_secret(),
            Duration::from_secs(300),
        );

        let runtime = Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Arc::new(Mutex::new(event_bus)),
            event_tx,
            tool_registry: Arc::new(tool_registry),
            storage_adapters: Arc::new(RwLock::new(adapters)),
            is_running: Arc::new(Mutex::new(false)),
            #[cfg(feature = "auth")]
            security: security_env.clone(),
        };

        #[cfg(feature = "auth")]
        {
            // Initialise tracing once; callers may choose to override later.
            crate::security::install_redacted_tracing(security_env);
        }

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
        // 1. Broadcast to local agents via channel.
        // If no runtime task is currently listening (e.g. after `stop()` or during shutdown),
        // the broadcast channel may be in a *closed* state.  Treat this as a **non-fatal**
        // condition – upstream callers emit events primarily for agent consumption and can
        // safely ignore the absence of listeners.

        if let Err(tokio::sync::broadcast::error::SendError(_)) =
            self.event_tx.send((event_type.clone(), data.clone()))
        {
            // Channel closed – swallow the error to preserve best-effort semantics.
            // Downstream persistence (Vault) continues unimpeded.
        }

        // 2. Publish on the intra-process bus (legacy behaviour).
        let bus = self.event_bus.lock().await;
        let _bus_header = bus.publish(&data, &event_type).await?;

        // Persistence skipped in slim build.

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

        // Persist to a simple JSON file under the vault storage root.  This is
        // an interim solution until the new projection API lands.
        let json = serde_json::to_vec(&agent_ids)?;
        let key = "runtime_state.json";
        if let Some(vault_fs) = self.storage("vault").await {
            vault_fs.put(key, &json).await?;
        } else {
            // Fallback to local file system at runtime_path
            let path = std::path::Path::new(&self.config.vault_path).join(key);
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
            tokio::fs::write(path, &json).await?;
        }

        // Agent-specific state persistence skipped in slim build.

        Ok(())
    }

    /// Load runtime state from vault
    pub async fn load_state(&self) -> Result<()> {
        // The minimal build restores *only* the list of agent IDs (without their full state).
        // This is sufficient for basic CLI inspection commands.
        let key = "runtime_state.json";
        let maybe_bytes = if let Some(vault_fs) = self.storage("vault").await {
            vault_fs.get(key).await?
        } else {
            let path = std::path::Path::new(&self.config.vault_path).join(key);
            match tokio::fs::read(path).await {
                Ok(b) => Some(b),
                Err(_) => None,
            }
        };

        if let Some(bytes) = maybe_bytes {
            let agent_ids: Vec<String> = serde_json::from_slice(&bytes)
                .context("Failed to deserialize runtime agent summary")?;

            let mut agent_map = self.agents.write().await;
            for id in agent_ids {
                agent_map.entry(id).or_insert_with(|| {
                    Box::new(SymbolicAgent::new("restored")) as Box<dyn Agent + Send + Sync>
                });
            }

            info!("Loaded {} agent placeholders from vault", agent_map.len());

            // Per-agent state restore omitted in slim build.
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

    // ── Security helpers (auth feature) ─────────────────────────────────
    #[cfg(feature = "auth")]
    pub fn validator(&self) -> crate::security::MultiValidator {
        self.security.validator()
    }

    #[cfg(feature = "auth")]
    pub fn rotate_secrets(&self) {
        self.security.rotate();
    }

    #[cfg(feature = "auth")]
    pub fn redact(&self, input: &str) -> String {
        self.security.redact(input)
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
            ..RuntimeConfig::default()
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
            ..RuntimeConfig::default()
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
            ..RuntimeConfig::default()
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
