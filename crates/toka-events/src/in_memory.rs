use super::{Event, EventDispatcher, EventError, Subscriber};
use async_trait::async_trait;
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 1024;

/// Simple in-process dispatcher leveraging `tokio::sync::broadcast`.
/// Suitable for unit tests, local development, and embedded deployments.
#[derive(Debug, Clone)]
pub struct InMemoryDispatcher {
    sender: broadcast::Sender<Event>,
}

impl Default for InMemoryDispatcher {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }
}

#[async_trait]
impl EventDispatcher for InMemoryDispatcher {
    async fn publish(&self, event: Event) -> Result<(), EventError> {
        self.sender
            .send(event)
            .map_err(|e| EventError::Dispatch(format!("{e}")))?;
        Ok(())
    }

    async fn subscribe(&self) -> Result<Subscriber, EventError> {
        Ok(self.sender.subscribe())
    }
}
