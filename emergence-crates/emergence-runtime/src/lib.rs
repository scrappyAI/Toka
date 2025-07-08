//! **emergence-runtime** – Dynamic behavior composition and execution engine for EMERGENCE.

use emergence_physics::PhysicsEngine;
use emergence_nervous_system::NervousSystem;
use emergence_memory::MemorySubstrate;

pub struct ExecutionEngine {
    physics: PhysicsEngine,
    nervous_system: NervousSystem,
    memory: MemorySubstrate,
}

impl ExecutionEngine {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            physics: PhysicsEngine::new().await?,
            nervous_system: NervousSystem::new(),
            memory: MemorySubstrate::new(),
        })
    }
}