use std::sync::Arc;
use tokio::runtime::Runtime;

use toka_kernel::{Kernel, register_handler, OpcodeHandler, WorldState};
use toka_types::{Message, Operation, EntityId};
use toka_events::bus::{InMemoryBus, Event as KernelEvent, EventBus};
use toka_auth::{TokenValidator, Claims};
use anyhow::Result;

/// Dummy validator that always approves.
#[derive(Clone)]
struct NoopValidator;
#[async_trait::async_trait]
impl TokenValidator for NoopValidator {
    async fn validate(&self, _raw: &str) -> Result<Claims, toka_auth::Error> {
        Ok(Claims {
            sub: "test".into(),
            vault: "demo".into(),
            permissions: vec![],
            iat: 0,
            exp: u64::MAX,
            jti: "nokey".into(),
        })
    }
}

/// Dummy handler intercepts EmitEvent with topic "ext.test".
struct DummyHandler;

impl OpcodeHandler for DummyHandler {
    fn dispatch(&self, op: &Operation, _state: &mut WorldState) -> Result<Option<KernelEvent>, toka_kernel::KernelError> {
        if let Operation::EmitEvent { topic, data } = op {
            if topic == "ext.test" {
                return Ok(Some(KernelEvent::EventEmitted { topic: "handled".into(), data: data.clone() }));
            }
        }
        Ok(None)
    }
}

#[test]
fn external_handler_intercepts() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Register handler before kernel use.
        register_handler("dummy", Box::new(DummyHandler));

        let kernel = Kernel::new(WorldState::default(), Arc::new(NoopValidator), Arc::new(InMemoryBus::default()));

        let msg = Message {
            origin: EntityId(1),
            capability: "token".into(),
            op: Operation::EmitEvent { topic: "ext.test".into(), data: vec![1, 2, 3] },
        };

        let evt = kernel.submit(msg).await.unwrap();
        match evt {
            KernelEvent::EventEmitted { topic, .. } => assert_eq!(topic, "handled"),
            _ => panic!("unexpected event {:?}", evt),
        }
    });
}