#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! **toka-kernel** – Deterministic state-machine core of Toka OS.
//!
//! The kernel validates capability tokens, executes opcode handlers on an
//! in-memory `WorldState`, and emits typed events onto the shared event bus.
//! All operations are synchronous and **deterministic** for v0.1.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use toka_types::{EntityId, Message, Operation, TaskSpec, AgentSpec, Role};
use toka_events::bus::{Event as KernelEvent, EventBus};
use toka_auth::{TokenValidator, Claims};

//─────────────────────────────
//  World-state
//─────────────────────────────

/// In-memory tables representing the canonical world-state.
#[derive(Debug, Default)]
pub struct WorldState {
    /// Simple ledger – maps entity → balance (single asset for v0.1).
    pub balances: HashMap<EntityId, u64>,
    /// Total supply tracked per asset entity.
    pub supply: HashMap<EntityId, u64>,
    /// Agent inboxes (queued tasks).
    pub agent_tasks: HashMap<EntityId, Vec<TaskSpec>>,
    /// Basic user registry (`alias`, role).
    pub users: HashMap<EntityId, (String, Role)>,
}

//─────────────────────────────
//  Kernel error type
//─────────────────────────────

/// Deterministic error codes produced by the kernel.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum KernelError {
    /// Submitted capability token is not authorised.
    #[error("capability denied")]
    CapabilityDenied,
    /// Insufficient balance for debit.
    #[error("insufficient balance for entity {entity:?}: need {needed}, have {available}")]
    InsufficientBalance { entity: EntityId, needed: u64, available: u64 },
    /// Unknown entity referenced in the operation.
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    /// Invalid operation semantic (e.g. negative amount).
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

//─────────────────────────────
//  Kernel struct
//─────────────────────────────

/// Deterministic state-machine executor.
pub struct Kernel {
    state: Arc<RwLock<WorldState>>,
    auth: Arc<dyn TokenValidator>,
    bus: Arc<dyn EventBus>,
}

impl Kernel {
    /// Create a new kernel backed by `state`, `auth` validator and `bus`.
    pub fn new(state: WorldState, auth: Arc<dyn TokenValidator>, bus: Arc<dyn EventBus>) -> Self {
        Self { state: Arc::new(RwLock::new(state)), auth, bus }
    }

    /// Expose internal state pointer (read-only usage outside kernel).
    pub fn state_ptr(&self) -> Arc<RwLock<WorldState>> {
        Arc::clone(&self.state)
    }

    /// Submit a message, triggering capability validation, execution and event emission.
    pub async fn submit(&self, msg: Message) -> Result<KernelEvent> {
        // 1. Capability validation
        let _claims: Claims = self
            .auth
            .validate(&msg.capability)
            .await
            .map_err(|_| KernelError::CapabilityDenied)?;

        // 2. Dispatch → handler
        let evt = match msg.op {
            Operation::TransferFunds { from, to, amount } => {
                self.handle_transfer(from, to, amount).await?
            }
            Operation::MintAsset { asset, to, amount } => {
                self.handle_mint(asset, to, amount).await?
            }
            Operation::BurnAsset { asset, from, amount } => {
                self.handle_burn(asset, from, amount).await?
            }
            Operation::ScheduleAgentTask { agent, task } => {
                self.handle_schedule_task(agent, task).await?
            }
            Operation::SpawnSubAgent { parent, spec } => {
                self.handle_spawn_agent(parent, spec).await?
            }
            Operation::EmitObservation { agent, data } => {
                self.handle_observation(agent, data).await?
            }
            Operation::CreateUser { alias } => {
                self.handle_create_user(alias).await?
            }
            Operation::AssignRole { user, role } => {
                self.handle_assign_role(user, role).await?
            }
        };

        // 3. Emit event
        self.bus.publish(&evt)?;
        Ok(evt)
    }

    //───────────────────── handlers ─────────────────────

    async fn handle_transfer(&self, from: EntityId, to: EntityId, amount: u64) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        let src_balance = *state.balances.get(&from).unwrap_or(&0);
        if src_balance < amount {
            return Err(KernelError::InsufficientBalance { entity: from, needed: amount, available: src_balance }.into());
        }
        state.balances.insert(from, src_balance - amount);
        *state.balances.entry(to).or_insert(0) += amount;
        Ok(KernelEvent::FundsTransferred { from, to, amount })
    }

    async fn handle_mint(&self, asset: EntityId, to: EntityId, amount: u64) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        *state.supply.entry(asset).or_insert(0) += amount;
        *state.balances.entry(to).or_insert(0) += amount;
        Ok(KernelEvent::AssetMinted { asset, to, amount })
    }

    async fn handle_burn(&self, asset: EntityId, from: EntityId, amount: u64) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        let src_balance = *state.balances.get(&from).unwrap_or(&0);
        if src_balance < amount {
            return Err(KernelError::InsufficientBalance { entity: from, needed: amount, available: src_balance }.into());
        }
        let supply = state.supply.entry(asset).or_insert(0);
        if *supply < amount {
            return Err(KernelError::InvalidOperation("burn exceeds supply".into()).into());
        }
        *supply -= amount;
        state.balances.insert(from, src_balance - amount);
        Ok(KernelEvent::AssetBurned { asset, from, amount })
    }

    async fn handle_schedule_task(&self, agent: EntityId, task: TaskSpec) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        state.agent_tasks.entry(agent).or_default().push(task.clone());
        Ok(KernelEvent::TaskScheduled { agent, task })
    }

    async fn handle_spawn_agent(&self, parent: EntityId, spec: AgentSpec) -> Result<KernelEvent> {
        // For v0.1 we simply emit an event; state updates deferred to v0.2.
        Ok(KernelEvent::AgentSpawned { parent, spec })
    }

    async fn handle_observation(&self, agent: EntityId, data: Vec<u8>) -> Result<KernelEvent> {
        Ok(KernelEvent::ObservationEmitted { agent, data })
    }

    async fn handle_create_user(&self, alias: String) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        let new_id = EntityId(rand::random());
        state.users.insert(new_id, (alias.clone(), Role::Member));
        Ok(KernelEvent::UserCreated { alias, id: new_id })
    }

    async fn handle_assign_role(&self, user: EntityId, role: Role) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        if let Some((alias, _)) = state.users.get_mut(&user) {
            *state.users.get_mut(&user).unwrap() = (alias.clone(), role.clone());
            Ok(KernelEvent::RoleAssigned { user, role })
        } else {
            Err(KernelError::UnknownEntity(user).into())
        }
    }
}

//─────────────────────────────
//  Tests
//─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use async_trait::async_trait;

    struct AllowAllValidator;
    #[async_trait]
    impl TokenValidator for AllowAllValidator {
        async fn validate(&self, _raw: &str) -> Result<Claims, toka_auth::Error> {
            Ok(Claims {
                sub: "test".into(),
                vault: "test".into(),
                permissions: vec!["*".into()],
                iat: 0,
                exp: u64::MAX,
                jti: "test".into(),
            })
        }
    }

    fn test_kernel() -> Kernel {
        let auth = Arc::new(AllowAllValidator);
        let bus = Arc::new(toka_events::bus::InMemoryBus::default());
        Kernel::new(WorldState::default(), auth, bus)
    }

    #[test]
    fn mint_and_transfer() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let kernel = test_kernel();
            let asset = EntityId(1);
            let alice = EntityId(100);
            let bob = EntityId(200);

            // Mint 1000 to Alice
            let msg_mint = Message {
                origin: alice,
                capability: "allow".into(),
                op: Operation::MintAsset { asset, to: alice, amount: 1000 },
            };
            kernel.submit(msg_mint).await.unwrap();

            // Transfer 300 to Bob
            let msg_transfer = Message {
                origin: alice,
                capability: "allow".into(),
                op: Operation::TransferFunds { from: alice, to: bob, amount: 300 },
            };
            kernel.submit(msg_transfer).await.unwrap();

            // Assert balances
            let state = kernel.state.read().await;
            assert_eq!(state.balances.get(&alice), Some(&700));
            assert_eq!(state.balances.get(&bob), Some(&300));
        });
    }

    #[test]
    fn insufficient_balance() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let kernel = test_kernel();
            let alice = EntityId(50);
            let bob = EntityId(60);

            // Try to transfer without balance
            let res = kernel
                .submit(Message {
                    origin: alice,
                    capability: "allow".into(),
                    op: Operation::TransferFunds { from: alice, to: bob, amount: 10 },
                })
                .await;
            assert!(matches!(res, Err(e) if e.downcast_ref::<KernelError>().map_or(false, |k| matches!(k, KernelError::InsufficientBalance{..}))));
        });
    }
}
