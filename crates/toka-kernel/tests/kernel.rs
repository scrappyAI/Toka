use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use toka_auth::{Claims, TokenValidator};
use toka_events::bus::{Event as KernelEvent, EventBus, InMemoryBus};
use toka_kernel::{register_handler, Kernel, KernelError, OpcodeHandler, WorldState};
use toka_types::{EntityId, Message, Operation, TaskSpec};

//──────────────────────────────────────────────────────────────────────────────
//  Mock helpers
//──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct AllowAllValidator;

#[async_trait]
impl TokenValidator for AllowAllValidator {
    async fn validate(&self, _raw: &str) -> toka_auth::Result<Claims> {
        Ok(Claims {
            sub: "tester".into(),
            vault: "demo".into(),
            permissions: vec![],
            iat: 0,
            exp: u64::MAX,
            jti: "fixed".into(),
        })
    }
}

#[derive(Clone, Debug)]
struct DenyValidator;

#[async_trait]
impl TokenValidator for DenyValidator {
    async fn validate(&self, _raw: &str) -> toka_auth::Result<Claims> {
        Err(toka_auth::Error::new("denied"))
    }
}

//──────────────────────────────────────────────────────────────────────────────
//  External handler used by the registry test
//──────────────────────────────────────────────────────────────────────────────

struct ObservationHandler;

#[async_trait]
impl OpcodeHandler for ObservationHandler {
    fn dispatch(
        &self,
        op: &Operation,
        state: &mut WorldState,
    ) -> std::result::Result<Option<KernelEvent>, KernelError> {
        if let Operation::EmitObservation { agent, data } = op {
            // Side-effect: add a dummy task so we can assert state mutation.
            state
                .agent_tasks
                .entry(*agent)
                .or_default()
                .push(TaskSpec {
                    description: "generated from observation".into(),
                });
            return Ok(Some(KernelEvent::ObservationEmitted {
                agent: *agent,
                data: data.clone(),
            }));
        }
        Ok(None)
    }
}

//──────────────────────────────────────────────────────────────────────────────
//  Tests
//──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_kernel_schedule_task_happy_path() -> Result<()> {
    let bus: Arc<dyn EventBus> = Arc::new(InMemoryBus::default());
    let kernel = Kernel::new(WorldState::default(), Arc::new(AllowAllValidator), bus);

    let agent = EntityId(42);
    let task = TaskSpec {
        description: "demo task".into(),
    };
    let msg = Message {
        origin: agent,
        capability: "token".into(),
        op: Operation::ScheduleAgentTask {
            agent,
            task: task.clone(),
        },
    };

    let evt = kernel.submit(msg).await?;
    assert_eq!(
        evt,
        KernelEvent::TaskScheduled {
            agent,
            task: task.clone()
        }
    );

    // World-state must reflect the queued task.
    let state_arc = kernel.state_ptr();
    let state = state_arc.read().await;
    assert_eq!(state.agent_tasks.get(&agent).unwrap(), &vec![task]);
    Ok(())
}

#[tokio::test]
async fn test_kernel_capability_denied() {
    let bus: Arc<dyn EventBus> = Arc::new(InMemoryBus::default());
    let kernel = Kernel::new(WorldState::default(), Arc::new(DenyValidator), bus);

    let agent = EntityId(1);
    let msg = Message {
        origin: agent,
        capability: "invalid".into(),
        op: Operation::EmitObservation {
            agent,
            data: vec![],
        },
    };

    let err = kernel.submit(msg).await.unwrap_err();
    let kerr = err
        .downcast_ref::<KernelError>()
        .expect("error should downcast to KernelError");
    assert_eq!(*kerr, KernelError::CapabilityDenied);
}

#[tokio::test]
async fn test_external_opcode_handler_intercepts_operation() -> Result<()> {
    // Register custom handler.
    register_handler("observation", Box::new(ObservationHandler));

    let bus: Arc<dyn EventBus> = Arc::new(InMemoryBus::default());
    let kernel = Kernel::new(WorldState::default(), Arc::new(AllowAllValidator), bus);

    let agent = EntityId(99);
    let payload = vec![1, 2, 3];
    let msg = Message {
        origin: agent,
        capability: "cap".into(),
        op: Operation::EmitObservation {
            agent,
            data: payload.clone(),
        },
    };

    let evt = kernel.submit(msg).await?;
    assert_eq!(
        evt,
        KernelEvent::ObservationEmitted {
            agent,
            data: payload.clone()
        }
    );

    // Ensure our handler mutated state.
    let state_arc = kernel.state_ptr();
    let state = state_arc.read().await;
    assert!(state.agent_tasks.get(&agent).is_some());
    Ok(())
}