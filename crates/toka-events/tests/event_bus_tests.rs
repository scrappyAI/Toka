use anyhow::Result;
use toka_events::rich::{AuthEvent, EventType, EventBus, ToolEvent, VaultEvent, Event};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn event_bus_creation() {
    let bus = EventBus::new_default();
    // no subscribers initially
    let subs = bus.get_receiver();
    // receiver is usable (non-panicking); basic sanity
    drop(subs);
}

#[tokio::test]
async fn event_emission() -> Result<()> {
    let bus = EventBus::new_default();
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    bus.emit(EventType::Auth(AuthEvent::UserLogin { user_id: "alice".to_string(), timestamp: ts }), "test").await?;
    Ok(())
}

struct CountingSub { id: String, count: tokio::sync::Mutex<usize> }
#[async_trait::async_trait]
impl toka_events::rich::EventSubscriber for CountingSub {
    async fn handle_event(&self, _e: &Event) -> Result<()> {
        let mut c = self.count.lock().await; *c += 1; Ok(())
    }
    fn subscriber_id(&self) -> &str { &self.id }
}

#[tokio::test]
async fn subscriber_functionality() -> Result<()> {
    let bus = EventBus::new_default();
    let sub = Box::new(CountingSub { id: "test".into(), count: tokio::sync::Mutex::new(0) });
    bus.subscribe(sub).await?;
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    bus.emit_tool_event(ToolEvent::Invoked { tool_name: "dummy".into(), user_id: "u".into(), timestamp: ts }, "src").await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(bus.subscriber_count().await, 1);
    Ok(())
}

#[tokio::test]
async fn broadcast_receiver() -> Result<()> {
    let bus = EventBus::new_default();
    let mut rx = bus.get_receiver();
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    bus.emit_vault_event(VaultEvent::SecretCreated { vault_id: "v".into(), secret_key: "k".into(), timestamp: ts }, "svc").await?;
    let ev = rx.recv().await?;
    if let EventType::Vault(VaultEvent::SecretCreated { vault_id, .. }) = ev.event_type { assert_eq!(vault_id, "v"); } else { panic!("unexpected event"); }
    Ok(())
} 