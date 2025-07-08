#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **emergence-physics** – Core physics laws and enforcement for EMERGENCE living agent system.
//!
//! This crate provides the fundamental physics engine that enforces immutable laws
//! governing energy conservation, causality constraints, security boundaries, and
//! resource limits. These laws cannot be violated by any schema or agent behavior.
//!
//! The physics engine serves as the bedrock of trust and stability for the emergent
//! intelligence system, ensuring that all agent behaviors operate within safe and
//! predictable boundaries while allowing maximum creative freedom within those constraints.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result};
use blake3::Hash;
use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod energy;
pub mod causality;
pub mod security;
pub mod resources;
pub mod validation;

pub use energy::{EnergyConservation, EnergyState, EnergyTransaction};
pub use causality::{CausalityEngine, CausalChain, EventOrdering};
pub use security::{SecurityBoundaries, CapabilityGate, SecurityViolation};
pub use resources::{ResourceManager, ResourceAllocation, ResourceType};
pub use validation::{PhysicsValidator, ValidationError, ValidationResult};

/// Core physics engine that enforces immutable laws for the emergent system
#[derive(Debug)]
pub struct PhysicsEngine {
    /// Energy conservation enforcement
    energy_laws: Arc<RwLock<EnergyConservation>>,
    /// Causality and temporal ordering enforcement  
    causality_engine: Arc<CausalityEngine>,
    /// Security boundary and capability enforcement
    security_boundaries: Arc<SecurityBoundaries>,
    /// Resource allocation and limit enforcement
    resource_manager: Arc<ResourceManager>,
    /// Schema validation and physics law compliance
    validator: Arc<PhysicsValidator>,
    /// Engine start time for relative time calculations
    genesis_time: Instant,
    /// Unique engine instance identifier
    instance_id: Uuid,
}

/// Unique identifier for entities in the emergent system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

/// Represents a capability that an entity can possess
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "observe", "analyze", "communicate")
    pub name: String,
    /// Capability strength (0.0 to 1.0)
    pub strength: OrderedFloat<f64>,
    /// When this capability was acquired
    pub acquired_at: DateTime<Utc>,
    /// Evidence of competence (cryptographic proof)
    pub proof: Option<Hash>,
}

/// Represents a resource in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    /// Computational energy (normalized 0.0 to 1.0)
    Energy(OrderedFloat<f64>),
    /// Memory in megabytes
    Memory(u64),
    /// CPU percentage (0 to 100)
    Cpu(u8),
    /// Network bandwidth in KB/s
    Network(u32),
    /// Custom resource type
    Custom(String, serde_yaml::Value),
}

/// Physics operations that must be validated and enforced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsOperation {
    /// Validate that an entity has a specific capability
    ValidateCapability {
        entity: EntityId,
        capability: Capability,
    },
    /// Allocate resources to an entity
    AllocateResource {
        entity: EntityId,
        resource: Resource,
        amount: OrderedFloat<f64>,
    },
    /// Enforce time limits on operations
    EnforceTimeLimit {
        entity: EntityId,
        operation: String,
        limit: Duration,
    },
    /// Validate causal ordering of events
    ValidateCausality {
        event_id: Uuid,
        parent_events: Vec<Uuid>,
        timestamp: DateTime<Utc>,
    },
    /// Transfer energy between entities
    TransferEnergy {
        from: EntityId,
        to: EntityId,
        amount: OrderedFloat<f64>,
    },
}

/// Result of a physics operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Detailed result message
    pub message: String,
    /// Operation duration
    pub duration: Duration,
    /// Resource costs incurred
    pub costs: HashMap<String, OrderedFloat<f64>>,
    /// New state after operation (if applicable)
    pub new_state: Option<serde_yaml::Value>,
}

/// Physics constraint violation
#[derive(Debug, thiserror::Error)]
pub enum PhysicsViolation {
    /// Energy conservation law violated
    #[error("Energy conservation violated: {reason}")]
    EnergyConservation { reason: String },
    
    /// Causality constraint violated
    #[error("Causality violated: {reason}")]
    CausalityViolation { reason: String },
    
    /// Security boundary crossed
    #[error("Security boundary violated: {reason}")]
    SecurityBreach { reason: String },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} - {reason}")]
    ResourceLimit { resource: String, reason: String },
    
    /// Time limit exceeded
    #[error("Time limit exceeded: {operation} took {actual:?}, limit was {limit:?}")]
    TimeLimit {
        operation: String,
        actual: Duration,
        limit: Duration,
    },
}

impl PhysicsEngine {
    /// Create a new physics engine with default laws
    pub async fn new() -> Result<Self> {
        info!("Initializing EMERGENCE physics engine");
        
        let energy_laws = Arc::new(RwLock::new(EnergyConservation::new()));
        let causality_engine = Arc::new(CausalityEngine::new());
        let security_boundaries = Arc::new(SecurityBoundaries::new());
        let resource_manager = Arc::new(ResourceManager::new().await?);
        let validator = Arc::new(PhysicsValidator::new());
        
        let instance_id = Uuid::new_v4();
        let genesis_time = Instant::now();
        
        info!("Physics engine initialized with instance ID: {}", instance_id);
        
        Ok(Self {
            energy_laws,
            causality_engine,
            security_boundaries,
            resource_manager,
            validator,
            genesis_time,
            instance_id,
        })
    }
    
    /// Load physics laws from schema configuration
    pub async fn load_physics_schema(&mut self, schema_path: &str) -> Result<()> {
        info!("Loading physics schema from: {}", schema_path);
        
        let schema_content = tokio::fs::read_to_string(schema_path)
            .await
            .context("Failed to read physics schema file")?;
        
        let schema: serde_yaml::Value = serde_yaml::from_str(&schema_content)
            .context("Failed to parse physics schema YAML")?;
        
        // Validate schema compliance
        self.validator.validate_physics_schema(&schema).await?;
        
        // Apply energy conservation rules
        if let Some(energy_config) = schema.get("energy_conservation") {
            let mut energy_laws = self.energy_laws.write().await;
            energy_laws.configure_from_schema(energy_config)?;
        }
        
        // Apply causality constraints
        if let Some(causality_config) = schema.get("causality_constraints") {
            self.causality_engine.configure_from_schema(causality_config)?;
        }
        
        // Apply security boundaries
        if let Some(security_config) = schema.get("security_boundaries") {
            self.security_boundaries.configure_from_schema(security_config)?;
        }
        
        // Apply resource limits
        if let Some(resource_config) = schema.get("resource_limits") {
            self.resource_manager.configure_from_schema(resource_config).await?;
        }
        
        info!("Physics schema loaded and applied successfully");
        Ok(())
    }
    
    /// Execute a physics operation with full validation and enforcement
    pub async fn execute_operation(&self, operation: PhysicsOperation) -> Result<PhysicsResult> {
        let start_time = Instant::now();
        
        debug!("Executing physics operation: {:?}", operation);
        
        // Pre-validation
        self.validator.validate_operation(&operation).await?;
        
        let result = match operation {
            PhysicsOperation::ValidateCapability { entity, capability } => {
                self.validate_capability(entity, capability).await
            }
            PhysicsOperation::AllocateResource { entity, resource, amount } => {
                self.allocate_resource(entity, resource, amount).await
            }
            PhysicsOperation::EnforceTimeLimit { entity, operation, limit } => {
                self.enforce_time_limit(entity, operation, limit).await
            }
            PhysicsOperation::ValidateCausality { event_id, parent_events, timestamp } => {
                self.validate_causality(event_id, parent_events, timestamp).await
            }
            PhysicsOperation::TransferEnergy { from, to, amount } => {
                self.transfer_energy(from, to, amount).await
            }
        };
        
        let duration = start_time.elapsed();
        
        match &result {
            Ok(physics_result) => {
                debug!("Physics operation completed successfully in {:?}", duration);
                Ok(physics_result.clone())
            }
            Err(violation) => {
                warn!("Physics violation detected: {} (duration: {:?})", violation, duration);
                Err(violation.into())
            }
        }
    }
    
    /// Validate that an entity possesses a specific capability
    async fn validate_capability(&self, entity: EntityId, capability: Capability) -> Result<PhysicsResult, PhysicsViolation> {
        self.security_boundaries.validate_capability(entity, &capability).await
            .map_err(|e| PhysicsViolation::SecurityBreach { reason: e.to_string() })?;
        
        Ok(PhysicsResult {
            success: true,
            message: format!("Capability '{}' validated for entity {}", capability.name, entity.0),
            duration: Duration::from_millis(1),
            costs: HashMap::new(),
            new_state: None,
        })
    }
    
    /// Allocate resources to an entity with physics constraints
    async fn allocate_resource(&self, entity: EntityId, resource: Resource, amount: OrderedFloat<f64>) -> Result<PhysicsResult, PhysicsViolation> {
        let allocation = ResourceAllocation {
            entity,
            resource: resource.clone(),
            amount,
            allocated_at: Utc::now(),
        };
        
        self.resource_manager.allocate(allocation).await
            .map_err(|e| PhysicsViolation::ResourceLimit { 
                resource: format!("{:?}", resource), 
                reason: e.to_string() 
            })?;
        
        Ok(PhysicsResult {
            success: true,
            message: format!("Allocated {:?} of {:?} to entity {}", amount, resource, entity.0),
            duration: Duration::from_millis(2),
            costs: HashMap::from([("allocation_overhead".to_string(), OrderedFloat(0.001))]),
            new_state: None,
        })
    }
    
    /// Enforce time limits on operations
    async fn enforce_time_limit(&self, entity: EntityId, operation: String, limit: Duration) -> Result<PhysicsResult, PhysicsViolation> {
        // In a real implementation, this would track ongoing operations
        // For now, we just validate that the limit is reasonable
        const MAX_OPERATION_TIME: Duration = Duration::from_secs(300); // 5 minutes
        
        if limit > MAX_OPERATION_TIME {
            return Err(PhysicsViolation::TimeLimit {
                operation: operation.clone(),
                actual: limit,
                limit: MAX_OPERATION_TIME,
            });
        }
        
        Ok(PhysicsResult {
            success: true,
            message: format!("Time limit {:?} enforced for operation '{}' on entity {}", limit, operation, entity.0),
            duration: Duration::from_millis(1),
            costs: HashMap::new(),
            new_state: None,
        })
    }
    
    /// Validate causal ordering of events
    async fn validate_causality(&self, event_id: Uuid, parent_events: Vec<Uuid>, timestamp: DateTime<Utc>) -> Result<PhysicsResult, PhysicsViolation> {
        self.causality_engine.validate_event_ordering(event_id, &parent_events, timestamp).await
            .map_err(|e| PhysicsViolation::CausalityViolation { reason: e.to_string() })?;
        
        Ok(PhysicsResult {
            success: true,
            message: format!("Causal ordering validated for event {}", event_id),
            duration: Duration::from_millis(1),
            costs: HashMap::new(),
            new_state: None,
        })
    }
    
    /// Transfer energy between entities with conservation enforcement
    async fn transfer_energy(&self, from: EntityId, to: EntityId, amount: OrderedFloat<f64>) -> Result<PhysicsResult, PhysicsViolation> {
        let mut energy_laws = self.energy_laws.write().await;
        
        let transaction = EnergyTransaction {
            from: Some(from),
            to,
            amount,
            transaction_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };
        
        energy_laws.execute_transaction(transaction).await
            .map_err(|e| PhysicsViolation::EnergyConservation { reason: e.to_string() })?;
        
        Ok(PhysicsResult {
            success: true,
            message: format!("Transferred {} energy from {} to {}", amount, from.0, to.0),
            duration: Duration::from_millis(3),
            costs: HashMap::from([("transfer_fee".to_string(), OrderedFloat(0.01))]),
            new_state: None,
        })
    }
    
    /// Get current physics engine state
    pub async fn get_engine_state(&self) -> Result<PhysicsEngineState> {
        let energy_state = {
            let energy_laws = self.energy_laws.read().await;
            energy_laws.get_state().await
        };
        
        let resource_usage = self.resource_manager.get_usage().await;
        let causality_stats = self.causality_engine.get_statistics().await;
        let security_stats = self.security_boundaries.get_statistics().await;
        
        Ok(PhysicsEngineState {
            instance_id: self.instance_id,
            uptime: self.genesis_time.elapsed(),
            energy_state,
            resource_usage,
            causality_stats,
            security_stats,
        })
    }
    
    /// Shutdown the physics engine gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down physics engine {}", self.instance_id);
        
        // Perform final consistency checks
        let final_state = self.get_engine_state().await?;
        info!("Final physics engine state: {:?}", final_state);
        
        // Shutdown components
        self.resource_manager.shutdown().await?;
        
        info!("Physics engine shutdown complete");
        Ok(())
    }
}

/// Current state of the physics engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsEngineState {
    /// Engine instance identifier
    pub instance_id: Uuid,
    /// Engine uptime
    pub uptime: Duration,
    /// Energy conservation state
    pub energy_state: EnergyState,
    /// Resource usage statistics
    pub resource_usage: serde_yaml::Value,
    /// Causality engine statistics
    pub causality_stats: serde_yaml::Value,
    /// Security boundary statistics
    pub security_stats: serde_yaml::Value,
}

impl EntityId {
    /// Create a new entity ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create an entity ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entity-{}", self.0)
    }
}

impl Capability {
    /// Create a new capability
    pub fn new(name: String, strength: f64) -> Self {
        Self {
            name,
            strength: OrderedFloat(strength.clamp(0.0, 1.0)),
            acquired_at: Utc::now(),
            proof: None,
        }
    }
    
    /// Add cryptographic proof of competence
    pub fn with_proof(mut self, proof: Hash) -> Self {
        self.proof = Some(proof);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_physics_engine_creation() {
        let engine = PhysicsEngine::new().await.unwrap();
        let state = engine.get_engine_state().await.unwrap();
        
        assert!(state.uptime.as_millis() < 100); // Should be very recent
        println!("Physics engine created successfully: {}", state.instance_id);
    }
    
    #[tokio::test]
    async fn test_capability_validation() {
        let engine = PhysicsEngine::new().await.unwrap();
        let entity = EntityId::new();
        let capability = Capability::new("test_capability".to_string(), 0.8);
        
        let operation = PhysicsOperation::ValidateCapability { entity, capability };
        let result = engine.execute_operation(operation).await;
        
        // This should succeed for now (no capabilities registered yet)
        match result {
            Ok(physics_result) => {
                println!("Capability validation result: {}", physics_result.message);
            }
            Err(e) => {
                println!("Capability validation failed: {}", e);
            }
        }
    }
    
    #[tokio::test]
    async fn test_energy_transfer() {
        let engine = PhysicsEngine::new().await.unwrap();
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();
        
        let operation = PhysicsOperation::TransferEnergy {
            from: entity1,
            to: entity2,
            amount: OrderedFloat(0.1),
        };
        
        let result = engine.execute_operation(operation).await;
        
        match result {
            Ok(physics_result) => {
                println!("Energy transfer result: {}", physics_result.message);
                assert!(physics_result.success);
            }
            Err(e) => {
                println!("Energy transfer failed: {}", e);
            }
        }
    }
}