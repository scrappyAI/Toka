use toka_events::bus::{Event, EventBus, InMemoryBus};
use toka_types::{EntityId, TaskSpec};

#[tokio::test]
async fn test_inmemory_bus_publish_and_receive() {
    let bus = InMemoryBus::default();
    let mut rx = bus.subscribe();

    let evt = Event::TaskScheduled {
        agent: EntityId(7),
        task: TaskSpec {
            description: "unit test task".into(),
        },
    };

    bus.publish(&evt).unwrap();
    let received = rx.recv().await.unwrap();

    assert_eq!(evt, received);
}