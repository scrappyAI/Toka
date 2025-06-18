use anyhow::Result;
use toka_events::{Event, EventDispatcher, EventKind, InMemoryDispatcher};

#[tokio::test]
async fn test_publish_subscribe_flow() -> Result<()> {
    let bus = InMemoryDispatcher::default();

    let mut sub = bus.subscribe().await?;

    // Publish an event
    let payload = serde_json::json!({"msg": "hello"});
    let event = Event::new(EventKind::Log, &payload);

    bus.publish(event.clone()).await?;

    // Subscriber should receive it
    let received = sub.recv().await?;
    assert_eq!(received.kind, EventKind::Log);
    assert_eq!(received.payload, payload);

    Ok(())
}
