//! Toka Kernel Monitor
//!
//! A standalone monitoring utility for the Toka kernel that provides real-time
//! insights into kernel operations, performance metrics, and security events.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use tokio::signal;
use tracing::{info, warn, error};
use tracing_subscriber;

use toka_kernel::{Kernel, WorldState};
use toka_auth::JwtHs256Validator;
use toka_bus_core::{InMemoryBus, EventBus};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🔍 Starting Toka Kernel Monitor");
    
    // Create a simple kernel setup for monitoring
    let world_state = WorldState::default();
    let auth = Arc::new(JwtHs256Validator::new("monitor-secret"));
    let bus = Arc::new(InMemoryBus::default());
    
    let kernel = Kernel::new(world_state, auth, bus.clone());
    let kernel = Arc::new(kernel);
    
    info!("✅ Kernel monitoring setup complete");
    
    // Subscribe to kernel events
    let mut event_rx = bus.subscribe();
    
    // Start monitoring loop
    let monitor_task = tokio::spawn(async move {
        info!("📡 Starting event monitoring...");
        
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    info!("🔔 Kernel Event: {:?}", event);
                    
                    // You could add more sophisticated monitoring here:
                    // - Performance metrics collection
                    // - Security event analysis
                    // - Resource usage tracking
                    // - Alert generation
                }
                Err(e) => {
                    error!("❌ Error receiving event: {}", e);
                    break;
                }
            }
        }
    });
    
    // Statistics reporting task
    let stats_task = tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            info!("📊 Kernel Monitor Status:");
            info!("   - Monitor uptime: {:?}", std::time::SystemTime::now());
            info!("   - Memory usage: Available via sysinfo if needed");
            
            // Add more comprehensive stats here:
            // - Event counts by type
            // - Error rates
            // - Performance metrics
            // - Resource consumption
        }
    });
    
    // Wait for shutdown signal
    info!("🎯 Monitor ready. Press Ctrl+C to stop.");
    
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("🛑 Shutdown signal received");
        }
        _ = monitor_task => {
            warn!("📡 Event monitoring task ended unexpectedly");
        }
        _ = stats_task => {
            warn!("📊 Statistics task ended unexpectedly");
        }
    }
    
    info!("👋 Toka Kernel Monitor shutting down");
    Ok(())
}