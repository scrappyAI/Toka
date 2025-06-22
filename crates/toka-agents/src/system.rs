use crate::{Agent, BaseAgent, EventBus};
use anyhow::Result;
use async_trait::async_trait;

/// Built-in system agent kinds managed by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAgentKind {
    Watcher,
    Logger,
    Health,
    ToolManager,
}

/// Thin wrapper around `BaseAgent` exposing specialised behaviour for runtime-native tasks.
pub struct SystemAgent {
    inner: BaseAgent,
    kind: SystemAgentKind,
}

impl SystemAgent {
    pub fn new(kind: SystemAgentKind, id: &str) -> Self {
        let agent = BaseAgent::new(id);
        Self { inner: agent, kind }
    }

    pub fn set_event_bus(&mut self, bus: EventBus) {
        self.inner.set_event_bus(bus);
    }

    pub fn kind(&self) -> SystemAgentKind {
        self.kind
    }
}

#[async_trait]
impl Agent for SystemAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn process_event(&mut self, event_type: &str, data: &str) -> Result<()> {
        match self.kind {
            SystemAgentKind::Watcher => {
                // TODO: implement file watcher logic (placeholder)
                if event_type.contains("fs::change") {
                    tracing::info!(target: "watcher", "Observed change: {}", data);
                }
            }
            SystemAgentKind::Logger => {
                tracing::info!(target: "logger", "{} -> {}", event_type, data);
            }
            SystemAgentKind::Health => {
                // Future: health checks
            }
            SystemAgentKind::ToolManager => {
                // Future: dynamic tool updates
            }
        }
        // Also delegate to BaseAgent belief update pipeline for uniformity.
        self.inner.process_event(event_type, data).await
    }

    async fn save_state(&self, adapter: &dyn toka_security_vault::MemoryAdapter) -> Result<()> {
        self.inner.save_state(adapter).await
    }

    async fn load_state(&mut self, adapter: &dyn toka_security_vault::MemoryAdapter) -> Result<()> {
        self.inner.load_state(adapter).await
    }
}
