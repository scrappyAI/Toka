//! Audit event handler for comprehensive security logging

use crate::{KeyVersion, RotationEventHandler};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Comprehensive audit event types for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEvent {
    /// A new key was generated
    KeyGenerated {
        key_id: Uuid,
        generation: u64,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    },
    /// A key rotation occurred
    KeyRotated {
        old_key_id: Uuid,
        old_generation: u64,
        new_key_id: Uuid,
        new_generation: u64,
        rotated_at: DateTime<Utc>,
    },
    /// Expired keys were cleaned up
    KeysCleanedUp {
        count: usize,
        cleaned_at: DateTime<Utc>,
    },
    /// Key rotation failed
    RotationFailed {
        error_message: String,
        failed_at: DateTime<Utc>,
    },
    /// Key validation attempt
    KeyValidation {
        key_id: Uuid,
        success: bool,
        validated_at: DateTime<Utc>,
        client_info: Option<ClientInfo>,
    },
    /// Suspicious activity detected
    SecurityAlert {
        alert_type: SecurityAlertType,
        description: String,
        severity: AlertSeverity,
        detected_at: DateTime<Utc>,
        metadata: serde_json::Value,
    },
}

/// Client information for audit trails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

/// Types of security alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAlertType {
    /// Multiple failed validation attempts
    RepeatedFailures,
    /// Token reuse detected
    TokenReuse,
    /// Expired key usage attempt
    ExpiredKeyUsage,
    /// Invalid key format
    InvalidKeyFormat,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Suspicious timing patterns
    TimingAnomaly,
}

/// Severity levels for security alerts
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Low => write!(f, "LOW"),
            AlertSeverity::Medium => write!(f, "MEDIUM"),
            AlertSeverity::High => write!(f, "HIGH"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Trait for persisting audit events
#[async_trait]
pub trait AuditStore: Send + Sync {
    /// Store an audit event
    async fn store_event(&self, event: &AuditEvent) -> Result<()>;
    
    /// Query audit events by time range
    async fn query_events(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        event_types: Option<Vec<String>>,
    ) -> Result<Vec<AuditEvent>>;
    
    /// Get security alerts above a certain severity
    async fn get_alerts(&self, min_severity: AlertSeverity) -> Result<Vec<AuditEvent>>;
    
    /// Archive old audit events
    async fn archive_events(&self, before: DateTime<Utc>) -> Result<usize>;
}

/// Comprehensive audit event handler with configurable storage and alerting
pub struct AuditEventHandler<S>
where
    S: AuditStore,
{
    store: Arc<S>,
    config: AuditConfig,
    metrics: Arc<RwLock<AuditMetrics>>,
}

/// Configuration for audit event handling
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Whether to enable structured logging
    pub structured_logging: bool,
    /// Whether to enable real-time alerting
    pub real_time_alerts: bool,
    /// Minimum severity for real-time alerts
    pub alert_threshold: AlertSeverity,
    /// Whether to enable event batching for performance
    pub batch_events: bool,
    /// Maximum batch size before flush
    pub batch_size: usize,
    /// Maximum time to wait before flushing batch
    pub batch_timeout_secs: u64,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            structured_logging: true,
            real_time_alerts: true,
            alert_threshold: AlertSeverity::Medium,
            batch_events: false,
            batch_size: 100,
            batch_timeout_secs: 30,
        }
    }
}

/// Metrics for audit event handling
#[derive(Debug, Default, Clone)]
pub struct AuditMetrics {
    pub total_events: u64,
    pub events_by_type: std::collections::HashMap<String, u64>,
    pub alerts_generated: u64,
    pub storage_errors: u64,
    pub last_event_time: Option<DateTime<Utc>>,
}

impl<S> AuditEventHandler<S>
where
    S: AuditStore,
{
    /// Create a new audit event handler
    pub fn new(store: Arc<S>, config: AuditConfig) -> Self {
        Self {
            store,
            config,
            metrics: Arc::new(RwLock::new(AuditMetrics::default())),
        }
    }

    /// Log a security alert
    pub async fn log_security_alert(
        &self,
        alert_type: SecurityAlertType,
        description: String,
        severity: AlertSeverity,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let event = AuditEvent::SecurityAlert {
            alert_type,
            description: description.clone(),
            severity,
            detected_at: Utc::now(),
            metadata,
        };

        self.process_event(&event).await?;

        // Real-time alerting for high-severity events
        if self.config.real_time_alerts && severity >= self.config.alert_threshold {
            self.send_real_time_alert(&event).await?;
        }

        Ok(())
    }

    /// Log a key validation event
    pub async fn log_key_validation(
        &self,
        key_id: Uuid,
        success: bool,
        client_info: Option<ClientInfo>,
    ) -> Result<()> {
        let event = AuditEvent::KeyValidation {
            key_id,
            success,
            validated_at: Utc::now(),
            client_info,
        };

        self.process_event(&event).await
    }

    /// Get current audit metrics
    pub async fn get_metrics(&self) -> AuditMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Process and store an audit event
    async fn process_event(&self, event: &AuditEvent) -> Result<()> {
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_events += 1;
            metrics.last_event_time = Some(Utc::now());
            
            let event_type = match event {
                AuditEvent::KeyGenerated { .. } => "key_generated",
                AuditEvent::KeyRotated { .. } => "key_rotated",
                AuditEvent::KeysCleanedUp { .. } => "keys_cleaned_up",
                AuditEvent::RotationFailed { .. } => "rotation_failed",
                AuditEvent::KeyValidation { .. } => "key_validation",
                AuditEvent::SecurityAlert { .. } => "security_alert",
            };
            
            *metrics.events_by_type.entry(event_type.to_string()).or_insert(0) += 1;
            
            if matches!(event, AuditEvent::SecurityAlert { .. }) {
                metrics.alerts_generated += 1;
            }
        }

        // Structured logging
        if self.config.structured_logging {
            match event {
                AuditEvent::KeyGenerated { key_id, generation, .. } => {
                    info!(
                        event_type = "key_generated",
                        key_id = %key_id,
                        generation = generation,
                        "New JWT signing key generated"
                    );
                }
                AuditEvent::KeyRotated { old_key_id, new_key_id, old_generation, new_generation, .. } => {
                    info!(
                        event_type = "key_rotated",
                        old_key_id = %old_key_id,
                        new_key_id = %new_key_id,
                        old_generation = old_generation,
                        new_generation = new_generation,
                        "JWT signing key rotated"
                    );
                }
                AuditEvent::SecurityAlert { alert_type, severity, description, .. } => {
                    match severity {
                        AlertSeverity::Low => {
                            info!(
                                event_type = "security_alert",
                                alert_type = ?alert_type,
                                severity = %severity,
                                description = description,
                                "Security alert generated"
                            );
                        }
                        AlertSeverity::Medium => {
                            warn!(
                                event_type = "security_alert",
                                alert_type = ?alert_type,
                                severity = %severity,
                                description = description,
                                "Security alert generated"
                            );
                        }
                        AlertSeverity::High | AlertSeverity::Critical => {
                            error!(
                                event_type = "security_alert",
                                alert_type = ?alert_type,
                                severity = %severity,
                                description = description,
                                "Security alert generated"
                            );
                        }
                    }
                }
                _ => {
                    info!(event = ?event, "Audit event processed");
                }
            }
        }

        // Store event
        if let Err(e) = self.store.store_event(event).await {
            error!("Failed to store audit event: {}", e);
            let mut metrics = self.metrics.write().await;
            metrics.storage_errors += 1;
            return Err(e);
        }

        Ok(())
    }

    /// Send real-time alert for high-severity events
    async fn send_real_time_alert(&self, event: &AuditEvent) -> Result<()> {
        // In a real implementation, this would integrate with alerting systems
        // like PagerDuty, Slack, email, etc.
        match event {
            AuditEvent::SecurityAlert { severity, description, .. } => {
                warn!(
                    severity = %severity,
                    description = description,
                    "REAL-TIME SECURITY ALERT: {}", description
                );
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<S> RotationEventHandler for AuditEventHandler<S>
where
    S: AuditStore,
{
    async fn on_key_generated(&self, key: &KeyVersion) -> Result<()> {
        let event = AuditEvent::KeyGenerated {
            key_id: key.id,
            generation: key.generation,
            created_at: key.created_at,
            expires_at: key.expires_at,
        };
        
        self.process_event(&event).await
    }

    async fn on_key_rotated(&self, old_key: &KeyVersion, new_key: &KeyVersion) -> Result<()> {
        let event = AuditEvent::KeyRotated {
            old_key_id: old_key.id,
            old_generation: old_key.generation,
            new_key_id: new_key.id,
            new_generation: new_key.generation,
            rotated_at: Utc::now(),
        };
        
        self.process_event(&event).await
    }

    async fn on_keys_cleaned_up(&self, count: usize) -> Result<()> {
        let event = AuditEvent::KeysCleanedUp {
            count,
            cleaned_at: Utc::now(),
        };
        
        self.process_event(&event).await
    }

    async fn on_rotation_failed(&self, error: &anyhow::Error) -> Result<()> {
        let event = AuditEvent::RotationFailed {
            error_message: error.to_string(),
            failed_at: Utc::now(),
        };
        
        self.process_event(&event).await?;
        
        // This is also a security alert
        self.log_security_alert(
            SecurityAlertType::TimingAnomaly,
            format!("Key rotation failed: {}", error),
            AlertSeverity::High,
            serde_json::json!({
                "error": error.to_string(),
                "component": "key_rotation"
            }),
        ).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct MockAuditStore {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditStore for MockAuditStore {
        async fn store_event(&self, event: &AuditEvent) -> Result<()> {
            self.events.lock().await.push(event.clone());
            Ok(())
        }

        async fn query_events(
            &self,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
            _event_types: Option<Vec<String>>,
        ) -> Result<Vec<AuditEvent>> {
            Ok(self.events.lock().await.clone())
        }

        async fn get_alerts(&self, min_severity: AlertSeverity) -> Result<Vec<AuditEvent>> {
            let events = self.events.lock().await;
            Ok(events.iter()
                .filter(|e| matches!(e, AuditEvent::SecurityAlert { severity, .. } if *severity >= min_severity))
                .cloned()
                .collect())
        }

        async fn archive_events(&self, _before: DateTime<Utc>) -> Result<usize> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_audit_event_handler() {
        let store = Arc::new(MockAuditStore::default());
        let config = AuditConfig::default();
        let handler = AuditEventHandler::new(store.clone(), config);

        // Test key generation event
        let key = KeyVersion::new(1, chrono::Duration::hours(24), chrono::Duration::hours(1)).unwrap();
        handler.on_key_generated(&key).await.unwrap();

        // Test security alert
        handler.log_security_alert(
            SecurityAlertType::RepeatedFailures,
            "Multiple failed validation attempts".to_string(),
            AlertSeverity::Medium,
            serde_json::json!({"attempts": 5}),
        ).await.unwrap();

        let events = store.events.lock().await;
        assert_eq!(events.len(), 2);

        let metrics = handler.get_metrics().await;
        assert_eq!(metrics.total_events, 2);
        assert_eq!(metrics.alerts_generated, 1);
    }

    #[tokio::test]
    async fn test_key_validation_logging() {
        let store = Arc::new(MockAuditStore::default());
        let config = AuditConfig::default();
        let handler = AuditEventHandler::new(store.clone(), config);

        let key_id = Uuid::new_v4();
        let client_info = ClientInfo {
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            request_id: Some("req-123".to_string()),
        };

        handler.log_key_validation(key_id, true, Some(client_info)).await.unwrap();

        let events = store.events.lock().await;
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            AuditEvent::KeyValidation { key_id: logged_id, success, client_info, .. } => {
                assert_eq!(*logged_id, key_id);
                assert!(success);
                assert!(client_info.is_some());
            }
            _ => panic!("Expected KeyValidation event"),
        }
    }
} 