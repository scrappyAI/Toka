use std::sync::Arc;

use anyhow::Result;
use toka_events::bus::{EventBus, InMemoryBus};
use toka_kernel::{Kernel, WorldState};
use toka_types::{EntityId, Message, Operation, TaskSpec, AgentSpec};

use async_trait::async_trait;
use toka_auth::{Claims, TokenValidator};

#[derive(Clone, Debug)]
struct AllowAll;

#[async_trait]
impl TokenValidator for AllowAll {
    async fn validate(&self, _raw: &str) -> toka_auth::Result<Claims> {
        Ok(Claims {
            sub: "e2e".into(),
            vault: "demo".into(),
            permissions: vec![],
            iat: 0,
            exp: u64::MAX,
            jti: "e2e".into(),
        })
    }
}

#[tokio::test]
async fn e2e_kernel_happy_flow() -> Result<()> {
    let bus = Arc::new(InMemoryBus::default());
    let mut rx = bus.subscribe();
    let kernel = Kernel::new(WorldState::default(), Arc::new(AllowAll), bus.clone());

    // 1. Schedule task
    let agent = EntityId(10);
    let task = TaskSpec { description: "demo".into() };
    let msg = Message { origin: agent, capability: "cap".into(), op: Operation::ScheduleAgentTask { agent, task: task.clone() } };
    let evt1 = kernel.submit(msg).await?;

    // 2. Spawn sub agent
    let child_spec = AgentSpec { name: "child".into() };
    let msg2 = Message { origin: agent, capability: "cap".into(), op: Operation::SpawnSubAgent { parent: agent, spec: child_spec.clone() } };
    let evt2 = kernel.submit(msg2).await?;

    // 3. Emit observation
    let data = vec![1,2,3];
    let msg3 = Message { origin: agent, capability: "cap".into(), op: Operation::EmitObservation { agent, data: data.clone() } };
    let evt3 = kernel.submit(msg3).await?;

    // Collect three events from bus (order preserved).
    let got1 = rx.recv().await?;
    let got2 = rx.recv().await?;
    let got3 = rx.recv().await?;

    assert_eq!(evt1, got1);
    assert_eq!(evt2, got2);
    assert_eq!(evt3, got3);

    Ok(())
}