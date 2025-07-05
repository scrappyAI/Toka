//! Key rotation manager with automatic background rotation

use crate::{KeyVersion, RotationConfig, KeyStore, RotationEventHandler};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Manages automatic JWT key rotation with configurable intervals and event handling
pub struct KeyRotationManager<S, E> 
where
    S: KeyStore,
    E: RotationEventHandler,
{
    store: Arc<S>,
    event_handler: Arc<E>,
    config: RotationConfig,
    current_generation: Arc<RwLock<u64>>,
    rotation_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl<S, E> KeyRotationManager<S, E>
where
    S: KeyStore + 'static,
    E: RotationEventHandler + 'static,
{
    /// Create a new key rotation manager
    pub fn new(store: Arc<S>, event_handler: Arc<E>, config: RotationConfig) -> Self {
        Self {
            store,
            event_handler,
            config,
            current_generation: Arc::new(RwLock::new(1)),
            rotation_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the manager and start automatic rotation if enabled
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing key rotation manager");
        
        // Check if we have any existing keys
        let active_key = self.store.get_active_key().await?;
        
        if active_key.is_none() {
            info!("No active key found, generating initial key");
            let initial_key = self.generate_new_key().await?;
            self.store.store_key(&initial_key).await?;
            self.event_handler.on_key_generated(&initial_key).await?;
        } else {
            info!("Found existing active key, determining current generation");
            let all_keys = self.store.get_all_keys().await?;
            let max_generation = all_keys.iter().map(|k| k.generation).max().unwrap_or(0);
            *self.current_generation.write().await = max_generation;
            debug!("Current generation set to {}", max_generation);
        }

        // Start automatic rotation if enabled
        if self.config.auto_rotation_enabled {
            self.start_automatic_rotation().await?;
        }

        info!("Key rotation manager initialized successfully");
        Ok(())
    }

    /// Start the automatic rotation background task
    pub async fn start_automatic_rotation(&self) -> Result<()> {
        let mut task_guard = self.rotation_task.write().await;
        
        if task_guard.is_some() {
            warn!("Automatic rotation already started");
            return Ok(());
        }

        info!("Starting automatic key rotation with interval: {:?}", self.config.rotation_interval);
        
        let store = Arc::clone(&self.store);
        let event_handler = Arc::clone(&self.event_handler);
        let config = self.config.clone();
        let current_generation = Arc::clone(&self.current_generation);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(config.rotation_interval.to_std().unwrap());
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                debug!("Checking if key rotation is needed");
                match Self::perform_rotation_check(&store, &event_handler, &config, &current_generation).await {
                    Ok(rotated) => {
                        if rotated {
                            info!("Key rotation completed successfully");
                        }
                    }
                    Err(e) => {
                        error!("Key rotation failed: {}", e);
                        if let Err(handler_err) = event_handler.on_rotation_failed(&e).await {
                            error!("Event handler failed to process rotation error: {}", handler_err);
                        }
                    }
                }
                
                // Cleanup expired keys
                match store.cleanup_expired_keys().await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Cleaned up {} expired keys", count);
                            if let Err(e) = event_handler.on_keys_cleaned_up(count).await {
                                error!("Event handler failed to process cleanup: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to cleanup expired keys: {}", e);
                    }
                }
            }
        });
        
        *task_guard = Some(handle);
        Ok(())
    }

    /// Stop the automatic rotation background task
    pub async fn stop_automatic_rotation(&self) -> Result<()> {
        let mut task_guard = self.rotation_task.write().await;
        
        if let Some(handle) = task_guard.take() {
            handle.abort();
            info!("Stopped automatic key rotation");
        }
        
        Ok(())
    }

    /// Manually trigger a key rotation
    pub async fn rotate_key(&self) -> Result<KeyVersion> {
        info!("Manually triggering key rotation");
        
        let old_key = self.store.get_active_key().await?;
        let new_key = self.generate_new_key().await?;
        
        // Deactivate old key
        if let Some(ref old) = old_key {
            self.store.deactivate_key(&old.id).await?;
        }
        
        // Store new key
        self.store.store_key(&new_key).await?;
        
        // Notify event handler
        self.event_handler.on_key_generated(&new_key).await?;
        if let Some(old) = old_key {
            self.event_handler.on_key_rotated(&old, &new_key).await?;
        }
        
        info!("Key rotation completed, new generation: {}", new_key.generation);
        Ok(new_key)
    }

    /// Get the current active key for signing tokens
    pub async fn get_signing_key(&self) -> Result<Option<KeyVersion>> {
        self.store.get_active_key().await
    }

    /// Get all keys valid for token validation
    pub async fn get_validation_keys(&self) -> Result<Vec<KeyVersion>> {
        self.store.get_valid_keys().await
    }

    /// Get rotation statistics for monitoring
    pub async fn get_rotation_stats(&self) -> Result<RotationStats> {
        let all_keys = self.store.get_all_keys().await?;
        let valid_keys = self.store.get_valid_keys().await?;
        let active_key = self.store.get_active_key().await?;
        
        Ok(RotationStats {
            total_keys: all_keys.len(),
            valid_keys: valid_keys.len(),
            current_generation: *self.current_generation.read().await,
            active_key_id: active_key.as_ref().map(|k| k.id),
            last_rotation: all_keys.iter().map(|k| k.created_at).max(),
            next_rotation: active_key.as_ref().map(|k| k.expires_at),
        })
    }

    /// Internal method to generate a new key with incremented generation
    async fn generate_new_key(&self) -> Result<KeyVersion> {
        let mut generation_guard = self.current_generation.write().await;
        *generation_guard += 1;
        let generation = *generation_guard;
        
        KeyVersion::new(generation, self.config.rotation_interval, self.config.overlap_period)
    }

    /// Internal method to check if rotation is needed and perform it
    async fn perform_rotation_check(
        store: &Arc<S>,
        event_handler: &Arc<E>,
        config: &RotationConfig,
        current_generation: &Arc<RwLock<u64>>,
    ) -> Result<bool> {
        let active_key = store.get_active_key().await?;
        
        let needs_rotation = match active_key {
            Some(ref key) => {
                let now = Utc::now();
                let time_until_expiry = key.expires_at - now;
                
                // Rotate if key expires soon (within 10% of rotation interval)
                let rotation_threshold = config.rotation_interval / 10;
                time_until_expiry <= rotation_threshold
            }
            None => true, // No active key, definitely need to rotate
        };
        
        if needs_rotation {
            debug!("Key rotation needed");
            
            let mut generation_guard = current_generation.write().await;
            *generation_guard += 1;
            let generation = *generation_guard;
            drop(generation_guard);
            
            let new_key = KeyVersion::new(generation, config.rotation_interval, config.overlap_period)?;
            
            // Deactivate old key if it exists
            if let Some(ref old) = active_key {
                store.deactivate_key(&old.id).await?;
            }
            
            // Store new key
            store.store_key(&new_key).await?;
            
            // Notify event handler
            event_handler.on_key_generated(&new_key).await?;
            if let Some(old) = active_key {
                event_handler.on_key_rotated(&old, &new_key).await?;
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }
}

/// Statistics about key rotation for monitoring and debugging
#[derive(Debug, Clone)]
pub struct RotationStats {
    pub total_keys: usize,
    pub valid_keys: usize,
    pub current_generation: u64,
    pub active_key_id: Option<Uuid>,
    pub last_rotation: Option<DateTime<Utc>>,
    pub next_rotation: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    
    // Mock implementations for testing
    
    #[derive(Default)]
    struct MockKeyStore {
        keys: Mutex<HashMap<Uuid, KeyVersion>>,
    }
    
    #[async_trait::async_trait]
    impl KeyStore for MockKeyStore {
        async fn store_key(&self, key: &KeyVersion) -> Result<()> {
            self.keys.lock().await.insert(key.id, key.clone());
            Ok(())
        }
        
        async fn get_key(&self, key_id: &Uuid) -> Result<Option<KeyVersion>> {
            Ok(self.keys.lock().await.get(key_id).cloned())
        }
        
        async fn get_active_key(&self) -> Result<Option<KeyVersion>> {
            let keys = self.keys.lock().await;
            Ok(keys.values().find(|k| k.is_active).cloned())
        }
        
        async fn get_valid_keys(&self) -> Result<Vec<KeyVersion>> {
            let keys = self.keys.lock().await;
            Ok(keys.values().filter(|k| k.is_valid_for_validation()).cloned().collect())
        }
        
        async fn deactivate_key(&self, key_id: &Uuid) -> Result<()> {
            if let Some(key) = self.keys.lock().await.get_mut(key_id) {
                key.is_active = false;
            }
            Ok(())
        }
        
        async fn cleanup_expired_keys(&self) -> Result<usize> {
            let mut keys = self.keys.lock().await;
            let before_count = keys.len();
            keys.retain(|_, v| v.is_valid_for_validation());
            Ok(before_count - keys.len())
        }
        
        async fn get_all_keys(&self) -> Result<Vec<KeyVersion>> {
            Ok(self.keys.lock().await.values().cloned().collect())
        }
    }
    
    #[derive(Default)]
    struct MockEventHandler {
        events: Mutex<Vec<String>>,
    }
    
    #[async_trait::async_trait]
    impl RotationEventHandler for MockEventHandler {
        async fn on_key_generated(&self, key: &KeyVersion) -> Result<()> {
            self.events.lock().await.push(format!("generated:{}", key.generation));
            Ok(())
        }
        
        async fn on_key_rotated(&self, old_key: &KeyVersion, new_key: &KeyVersion) -> Result<()> {
            self.events.lock().await.push(format!("rotated:{}:{}", old_key.generation, new_key.generation));
            Ok(())
        }
        
        async fn on_keys_cleaned_up(&self, count: usize) -> Result<()> {
            self.events.lock().await.push(format!("cleanup:{}", count));
            Ok(())
        }
        
        async fn on_rotation_failed(&self, _error: &anyhow::Error) -> Result<()> {
            self.events.lock().await.push("failed".to_string());
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_manager_initialization() {
        let store = Arc::new(MockKeyStore::default());
        let handler = Arc::new(MockEventHandler::default());
        let config = RotationConfig::default();
        
        let manager = KeyRotationManager::new(store.clone(), handler.clone(), config);
        manager.initialize().await.unwrap();
        
        let active_key = store.get_active_key().await.unwrap();
        assert!(active_key.is_some());
        
        let events = handler.events.lock().await;
        assert!(events.contains(&"generated:1".to_string()));
    }
    
    #[tokio::test]
    async fn test_manual_rotation() {
        let store = Arc::new(MockKeyStore::default());
        let handler = Arc::new(MockEventHandler::default());
        let config = RotationConfig::default();
        
        let manager = KeyRotationManager::new(store.clone(), handler.clone(), config);
        manager.initialize().await.unwrap();
        
        let new_key = manager.rotate_key().await.unwrap();
        assert_eq!(new_key.generation, 2);
        
        let events = handler.events.lock().await;
        assert!(events.contains(&"generated:1".to_string()));
        assert!(events.contains(&"generated:2".to_string()));
        assert!(events.contains(&"rotated:1:2".to_string()));
    }
} 