//! Storage implementations for authentication data

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{AuthError, AuthEvent, AuthResult, AuthSession, AuthStorage, OAuthState};

/// In-memory storage implementation for testing and development
#[derive(Debug)]
pub struct MemoryStorage {
    /// OAuth states
    oauth_states: Arc<RwLock<HashMap<String, OAuthState>>>,
    /// Authentication sessions
    sessions: Arc<RwLock<HashMap<Uuid, AuthSession>>>,
    /// Authentication events
    events: Arc<RwLock<Vec<AuthEvent>>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            oauth_states: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Get the number of stored OAuth states
    pub async fn oauth_state_count(&self) -> usize {
        self.oauth_states.read().await.len()
    }
    
    /// Get the number of stored sessions
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
    
    /// Get the number of stored events
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }
}

#[async_trait::async_trait]
impl AuthStorage for MemoryStorage {
    async fn store_oauth_state(&self, state: &OAuthState) -> AuthResult<()> {
        let mut states = self.oauth_states.write().await;
        states.insert(state.state.clone(), state.clone());
        debug!("Stored OAuth state: {}", state.state);
        Ok(())
    }
    
    async fn consume_oauth_state(&self, state_id: &str) -> AuthResult<Option<OAuthState>> {
        let mut states = self.oauth_states.write().await;
        let state = states.remove(state_id);
        if state.is_some() {
            debug!("Consumed OAuth state: {}", state_id);
        }
        Ok(state)
    }
    
    async fn store_session(&self, session: &AuthSession) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id, session.clone());
        debug!("Stored session: {}", session.session_id);
        Ok(())
    }
    
    async fn get_session(&self, session_id: &Uuid) -> AuthResult<Option<AuthSession>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).cloned();
        if session.is_some() {
            debug!("Retrieved session: {}", session_id);
        }
        Ok(session)
    }
    
    async fn update_session(&self, session: &AuthSession) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id, session.clone());
        debug!("Updated session: {}", session.session_id);
        Ok(())
    }
    
    async fn remove_session(&self, session_id: &Uuid) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(session_id).is_some();
        if removed {
            debug!("Removed session: {}", session_id);
        }
        Ok(())
    }
    
    async fn cleanup_expired(&self) -> AuthResult<u64> {
        let now = Utc::now();
        let mut cleaned = 0u64;
        
        // Clean up expired OAuth states
        {
            let mut states = self.oauth_states.write().await;
            let initial_count = states.len();
            states.retain(|_, state| state.expires_at > now);
            let expired_states = initial_count - states.len();
            cleaned += expired_states as u64;
            if expired_states > 0 {
                debug!("Cleaned up {} expired OAuth states", expired_states);
            }
        }
        
        // Clean up expired sessions
        {
            let mut sessions = self.sessions.write().await;
            let initial_count = sessions.len();
            sessions.retain(|_, session| session.expires_at > now);
            let expired_sessions = initial_count - sessions.len();
            cleaned += expired_sessions as u64;
            if expired_sessions > 0 {
                debug!("Cleaned up {} expired sessions", expired_sessions);
            }
        }
        
        if cleaned > 0 {
            info!("Total cleanup: {} expired items", cleaned);
        }
        
        Ok(cleaned)
    }
    
    async fn store_auth_event(&self, event: &AuthEvent) -> AuthResult<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        debug!("Stored auth event: {:?}", event.event_type);
        Ok(())
    }
}

/// SQLite storage implementation
#[cfg(feature = "sqlite-storage")]
pub mod sqlite {
    use super::*;
    use sqlx::{SqlitePool, Row};
    use serde_json;
    
    /// SQLite storage implementation
    #[derive(Debug)]
    pub struct SQLiteStorage {
        pool: SqlitePool,
    }
    
    impl SQLiteStorage {
        /// Create a new SQLite storage with the given database URL
        pub async fn new(database_url: &str) -> AuthResult<Self> {
            let pool = SqlitePool::connect(database_url).await?;
            
            // Run migrations
            let storage = Self { pool };
            storage.migrate().await?;
            
            Ok(storage)
        }
        
        /// Run database migrations
        async fn migrate(&self) -> AuthResult<()> {
            debug!("Running database migrations");
            
            // Create OAuth states table
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS oauth_states (
                    state TEXT PRIMARY KEY,
                    code_verifier TEXT NOT NULL,
                    expires_at TEXT NOT NULL,
                    redirect_url TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
            "#)
            .execute(&self.pool)
            .await?;
            
            // Create sessions table
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    github_user_id INTEGER NOT NULL,
                    github_user_data TEXT NOT NULL,
                    capabilities TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    expires_at TEXT NOT NULL,
                    last_activity TEXT NOT NULL,
                    jwt_token TEXT NOT NULL
                )
            "#)
            .execute(&self.pool)
            .await?;
            
            // Create auth events table
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS auth_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    github_user_data TEXT,
                    session_id TEXT,
                    details TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
            "#)
            .execute(&self.pool)
            .await?;
            
            // Create indexes
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_github_user_id ON sessions(github_user_id)")
                .execute(&self.pool)
                .await?;
            
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at)")
                .execute(&self.pool)
                .await?;
            
            sqlx::query("CREATE INDEX IF NOT EXISTS idx_oauth_states_expires_at ON oauth_states(expires_at)")
                .execute(&self.pool)
                .await?;
            
            info!("Database migrations completed");
            Ok(())
        }
    }
    
    #[async_trait::async_trait]
    impl AuthStorage for SQLiteStorage {
        async fn store_oauth_state(&self, state: &OAuthState) -> AuthResult<()> {
            sqlx::query(r#"
                INSERT OR REPLACE INTO oauth_states 
                (state, code_verifier, expires_at, redirect_url)
                VALUES (?, ?, ?, ?)
            "#)
            .bind(&state.state)
            .bind(&state.code_verifier)
            .bind(state.expires_at.to_rfc3339())
            .bind(&state.redirect_url)
            .execute(&self.pool)
            .await?;
            
            debug!("Stored OAuth state in SQLite: {}", state.state);
            Ok(())
        }
        
        async fn consume_oauth_state(&self, state_id: &str) -> AuthResult<Option<OAuthState>> {
            let row = sqlx::query(r#"
                SELECT state, code_verifier, expires_at, redirect_url
                FROM oauth_states 
                WHERE state = ?
            "#)
            .bind(state_id)
            .fetch_optional(&self.pool)
            .await?;
            
            if let Some(row) = row {
                // Delete the state after reading
                sqlx::query("DELETE FROM oauth_states WHERE state = ?")
                    .bind(state_id)
                    .execute(&self.pool)
                    .await?;
                
                let state = OAuthState {
                    state: row.get("state"),
                    code_verifier: row.get("code_verifier"),
                    expires_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("expires_at"))
                        .map_err(|e| AuthError::Storage(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    redirect_url: row.get("redirect_url"),
                };
                
                debug!("Consumed OAuth state from SQLite: {}", state_id);
                Ok(Some(state))
            } else {
                Ok(None)
            }
        }
        
        async fn store_session(&self, session: &AuthSession) -> AuthResult<()> {
            let github_user_data = serde_json::to_string(&session.github_user)
                .map_err(|e| AuthError::Storage(format!("Failed to serialize GitHub user: {}", e)))?;
            
            let capabilities_data = serde_json::to_string(&session.capabilities)
                .map_err(|e| AuthError::Storage(format!("Failed to serialize capabilities: {}", e)))?;
            
            sqlx::query(r#"
                INSERT OR REPLACE INTO sessions 
                (session_id, github_user_id, github_user_data, capabilities, 
                 created_at, expires_at, last_activity, jwt_token)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(session.session_id.to_string())
            .bind(session.github_user.id as i64)
            .bind(github_user_data)
            .bind(capabilities_data)
            .bind(session.created_at.to_rfc3339())
            .bind(session.expires_at.to_rfc3339())
            .bind(session.last_activity.to_rfc3339())
            .bind(&session.jwt_token)
            .execute(&self.pool)
            .await?;
            
            debug!("Stored session in SQLite: {}", session.session_id);
            Ok(())
        }
        
        async fn get_session(&self, session_id: &Uuid) -> AuthResult<Option<AuthSession>> {
            let row = sqlx::query(r#"
                SELECT session_id, github_user_data, capabilities,
                       created_at, expires_at, last_activity, jwt_token
                FROM sessions 
                WHERE session_id = ?
            "#)
            .bind(session_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
            
            if let Some(row) = row {
                let github_user = serde_json::from_str(&row.get::<String, _>("github_user_data"))
                    .map_err(|e| AuthError::Storage(format!("Failed to deserialize GitHub user: {}", e)))?;
                
                let capabilities = serde_json::from_str(&row.get::<String, _>("capabilities"))
                    .map_err(|e| AuthError::Storage(format!("Failed to deserialize capabilities: {}", e)))?;
                
                let session = AuthSession {
                    session_id: *session_id,
                    github_user,
                    capabilities,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| AuthError::Storage(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    expires_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("expires_at"))
                        .map_err(|e| AuthError::Storage(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    last_activity: DateTime::parse_from_rfc3339(&row.get::<String, _>("last_activity"))
                        .map_err(|e| AuthError::Storage(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    jwt_token: row.get("jwt_token"),
                };
                
                debug!("Retrieved session from SQLite: {}", session_id);
                Ok(Some(session))
            } else {
                Ok(None)
            }
        }
        
        async fn update_session(&self, session: &AuthSession) -> AuthResult<()> {
            self.store_session(session).await
        }
        
        async fn remove_session(&self, session_id: &Uuid) -> AuthResult<()> {
            sqlx::query("DELETE FROM sessions WHERE session_id = ?")
                .bind(session_id.to_string())
                .execute(&self.pool)
                .await?;
            
            debug!("Removed session from SQLite: {}", session_id);
            Ok(())
        }
        
        async fn cleanup_expired(&self) -> AuthResult<u64> {
            let now = Utc::now().to_rfc3339();
            
            // Clean up expired OAuth states
            let oauth_result = sqlx::query("DELETE FROM oauth_states WHERE expires_at < ?")
                .bind(&now)
                .execute(&self.pool)
                .await?;
            
            // Clean up expired sessions
            let session_result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
                .bind(&now)
                .execute(&self.pool)
                .await?;
            
            let total_cleaned = oauth_result.rows_affected() + session_result.rows_affected();
            
            if total_cleaned > 0 {
                info!("Cleaned up {} expired items from SQLite", total_cleaned);
            }
            
            Ok(total_cleaned)
        }
        
        async fn store_auth_event(&self, event: &AuthEvent) -> AuthResult<()> {
            let github_user_data = if let Some(ref user) = event.github_user {
                Some(serde_json::to_string(user)
                    .map_err(|e| AuthError::Storage(format!("Failed to serialize GitHub user: {}", e)))?)
            } else {
                None
            };
            
            let details_data = serde_json::to_string(&event.details)
                .map_err(|e| AuthError::Storage(format!("Failed to serialize event details: {}", e)))?;
            
            sqlx::query(r#"
                INSERT INTO auth_events 
                (timestamp, event_type, github_user_data, session_id, details)
                VALUES (?, ?, ?, ?, ?)
            "#)
            .bind(event.timestamp.to_rfc3339())
            .bind(format!("{:?}", event.event_type))
            .bind(github_user_data)
            .bind(event.session_id.map(|id| id.to_string()))
            .bind(details_data)
            .execute(&self.pool)
            .await?;
            
            debug!("Stored auth event in SQLite: {:?}", event.event_type);
            Ok(())
        }
    }
}

/// Storage factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage instance based on configuration
    pub async fn create_storage(
        backend: &crate::config::StorageBackend,
        database_url: Option<&str>,
    ) -> AuthResult<Arc<dyn AuthStorage>> {
        match backend {
            crate::config::StorageBackend::Memory => {
                info!("Using in-memory storage");
                Ok(Arc::new(MemoryStorage::new()))
            }
            #[cfg(feature = "sqlite-storage")]
            crate::config::StorageBackend::SQLite => {
                let url = database_url.ok_or_else(|| {
                    AuthError::Config("Database URL required for SQLite storage".to_string())
                })?;
                info!("Using SQLite storage: {}", url);
                let storage = sqlite::SQLiteStorage::new(url).await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "sqlite-storage"))]
            crate::config::StorageBackend::SQLite => {
                Err(AuthError::Config("SQLite storage not enabled in build".to_string()))
            }
            crate::config::StorageBackend::PostgreSQL => {
                Err(AuthError::Config("PostgreSQL storage not yet implemented".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitHubRole, GitHubUser, TokaCapability};
    
    #[tokio::test]
    async fn test_memory_storage_oauth_state() {
        let storage = MemoryStorage::new();
        
        let state = OAuthState {
            state: "test-state".to_string(),
            code_verifier: "test-verifier".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(10),
            redirect_url: None,
        };
        
        // Store state
        storage.store_oauth_state(&state).await.unwrap();
        assert_eq!(storage.oauth_state_count().await, 1);
        
        // Consume state
        let retrieved = storage.consume_oauth_state("test-state").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().state, "test-state");
        assert_eq!(storage.oauth_state_count().await, 0);
        
        // Try to consume again
        let retrieved = storage.consume_oauth_state("test-state").await.unwrap();
        assert!(retrieved.is_none());
    }
    
    #[tokio::test]
    async fn test_memory_storage_session() {
        let storage = MemoryStorage::new();
        let session_id = Uuid::new_v4();
        
        let session = AuthSession {
            session_id,
            github_user: GitHubUser {
                id: 12345,
                login: "testuser".to_string(),
                name: Some("Test User".to_string()),
                email: Some("test@example.com".to_string()),
                avatar_url: "https://github.com/avatar.png".to_string(),
                role: GitHubRole::Collaborator,
            },
            capabilities: vec![TokaCapability::Read, TokaCapability::Write],
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            last_activity: Utc::now(),
            jwt_token: "test-token".to_string(),
        };
        
        // Store session
        storage.store_session(&session).await.unwrap();
        assert_eq!(storage.session_count().await, 1);
        
        // Retrieve session
        let retrieved = storage.get_session(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().github_user.login, "testuser");
        
        // Remove session
        storage.remove_session(&session_id).await.unwrap();
        assert_eq!(storage.session_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_memory_storage_cleanup() {
        let storage = MemoryStorage::new();
        
        // Create expired OAuth state
        let expired_state = OAuthState {
            state: "expired-state".to_string(),
            code_verifier: "test-verifier".to_string(),
            expires_at: Utc::now() - chrono::Duration::minutes(10),
            redirect_url: None,
        };
        
        // Create valid OAuth state
        let valid_state = OAuthState {
            state: "valid-state".to_string(),
            code_verifier: "test-verifier".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(10),
            redirect_url: None,
        };
        
        storage.store_oauth_state(&expired_state).await.unwrap();
        storage.store_oauth_state(&valid_state).await.unwrap();
        assert_eq!(storage.oauth_state_count().await, 2);
        
        // Clean up expired items
        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(storage.oauth_state_count().await, 1);
        
        // Verify valid state still exists
        let retrieved = storage.consume_oauth_state("valid-state").await.unwrap();
        assert!(retrieved.is_some());
    }
} 