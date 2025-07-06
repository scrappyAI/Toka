//! Distributed tracing and span management
//!
//! This module provides comprehensive distributed tracing capabilities
//! for end-to-end operation visibility across Toka OS components.

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info};
use uuid::Uuid;

/// Distributed tracing manager
pub struct TracingManager {
    /// Component identifier
    component_id: String,
    /// Active spans
    active_spans: RwLock<HashMap<String, TraceSpan>>,
    /// Sampling rate
    sample_rate: f64,
    /// Tracing enabled
    enabled: bool,
}

/// Trace span wrapper
#[derive(Debug)]
pub struct TraceSpan {
    /// Span identifier
    pub span_id: String,
    /// Trace identifier
    pub trace_id: String,
    /// Span name
    pub name: String,
    /// Start time
    pub start_time: Instant,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Events
    pub events: Vec<TraceEvent>,
    /// Parent span ID
    pub parent_span_id: Option<String>,
}

/// Trace event
#[derive(Debug, Clone)]
pub struct TraceEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: Instant,
    /// Event attributes
    pub attributes: HashMap<String, String>,
}

/// Trace context for propagation
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Baggage
    pub baggage: HashMap<String, String>,
}

impl TracingManager {
    /// Create a new tracing manager
    pub async fn new(component_id: &str, sample_rate: f64) -> Result<Self> {
        Ok(Self {
            component_id: component_id.to_string(),
            active_spans: RwLock::new(HashMap::new()),
            sample_rate,
            enabled: true,
        })
    }
    
    /// Start distributed tracing
    pub async fn start(&self) -> Result<()> {
        ::tracing::info!(
            component = %self.component_id,
            sample_rate = %self.sample_rate,
            "Starting distributed tracing"
        );
        Ok(())
    }
    
    /// Stop distributed tracing
    pub async fn stop(&self) -> Result<()> {
        // End all active spans
        let mut active_spans = self.active_spans.write().await;
        for (_, span) in active_spans.drain() {
            span.end();
        }
        
        ::tracing::info!(
            component = %self.component_id,
            "Stopped distributed tracing"
        );
        Ok(())
    }
    
    /// Start a new trace span
    pub async fn start_span(&self, name: &str) -> Result<TraceSpan> {
        self.start_span_with_parent(name, None).await
    }
    
    /// Start a new trace span with parent
    pub async fn start_span_with_parent(&self, name: &str, parent_span_id: Option<String>) -> Result<TraceSpan> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Tracing is disabled"));
        }
        
        let span_id = Uuid::new_v4().to_string();
        let trace_id = Uuid::new_v4().to_string();
        
        let trace_span = TraceSpan {
            span_id: span_id.clone(),
            trace_id: trace_id.clone(),
            name: name.to_string(),
            start_time: Instant::now(),
            attributes: HashMap::new(),
            events: Vec::new(),
            parent_span_id,
        };
        
        self.active_spans.write().await.insert(span_id.clone(), trace_span.clone());
        
        Ok(trace_span)
    }
    
    /// End a span
    pub async fn end_span(&self, span_id: &str) -> Result<()> {
        if let Some(span) = self.active_spans.write().await.remove(span_id) {
            span.end();
        }
        Ok(())
    }
    
    /// Get active spans count
    pub async fn active_spans_count(&self) -> usize {
        self.active_spans.read().await.len()
    }
    
    /// Health check
    pub async fn health(&self) -> Result<super::ComponentHealth> {
        if self.enabled {
            Ok(super::ComponentHealth::Healthy)
        } else {
            Ok(super::ComponentHealth::Degraded)
        }
    }
}

impl TraceSpan {
    /// Add an attribute to the span
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
    
    /// Record an event
    pub fn record_event(&mut self, name: &str, attributes: HashMap<String, String>) {
        let event = TraceEvent {
            name: name.to_string(),
            timestamp: Instant::now(),
            attributes,
        };
        self.events.push(event);
    }
    
    /// End the span
    pub fn end(self) {
        let duration = self.start_time.elapsed();
        ::tracing::debug!(
            span_id = %self.span_id,
            trace_id = %self.trace_id,
            name = %self.name,
            duration_ms = %duration.as_millis(),
            "Span completed"
        );
    }
    
    /// Get trace context
    pub fn get_context(&self) -> TraceContext {
        TraceContext {
            trace_id: self.trace_id.clone(),
            span_id: self.span_id.clone(),
            baggage: HashMap::new(),
        }
    }
    
    /// Get span duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Clone for TraceSpan {
    fn clone(&self) -> Self {
        Self {
            span_id: self.span_id.clone(),
            trace_id: self.trace_id.clone(),
            name: self.name.clone(),
            start_time: self.start_time,
            attributes: self.attributes.clone(),
            events: self.events.clone(),
            parent_span_id: self.parent_span_id.clone(),
        }
    }
}

/// Tracing error types
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    /// Tracing not enabled
    #[error("Tracing is not enabled")]
    NotEnabled,
    
    /// Span not found
    #[error("Span '{0}' not found")]
    SpanNotFound(String),
    
    /// Invalid context
    #[error("Invalid trace context: {0}")]
    InvalidContext(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tracing_manager_creation() {
        let manager = TracingManager::new("test-component", 0.1).await;
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_span_creation() {
        let manager = TracingManager::new("test-component", 1.0).await.unwrap();
        
        let span = manager.start_span("test_operation").await;
        assert!(span.is_ok());
        
        let span = span.unwrap();
        assert_eq!(span.name, "test_operation");
        assert!(!span.span_id.is_empty());
        assert!(!span.trace_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_span_attributes() {
        let manager = TracingManager::new("test-component", 1.0).await.unwrap();
        
        let mut span = manager.start_span("test_operation").await.unwrap();
        span.set_attribute("key", "value");
        
        assert_eq!(span.attributes.get("key"), Some(&"value".to_string()));
    }
    
    #[tokio::test]
    async fn test_span_events() {
        let manager = TracingManager::new("test-component", 1.0).await.unwrap();
        
        let mut span = manager.start_span("test_operation").await.unwrap();
        
        let mut event_attrs = HashMap::new();
        event_attrs.insert("event_key".to_string(), "event_value".to_string());
        span.record_event("test_event", event_attrs);
        
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test_event");
    }
}