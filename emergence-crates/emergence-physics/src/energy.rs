//! Energy conservation laws for the EMERGENCE system.
//!
//! This module implements thermodynamic-like conservation laws for computational energy,
//! ensuring that the total energy in the system remains constant and agents cannot
//! create energy from nothing. Energy can be transferred, transformed, and dissipated,
//! but never created or destroyed.

use std::collections::HashMap;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{debug, warn};

use crate::EntityId;

/// Energy conservation enforcement engine
#[derive(Debug)]
pub struct EnergyConservation {
    /// Total system energy (conserved quantity)
    total_energy: OrderedFloat<f64>,
    /// Current energy allocations per entity
    allocations: HashMap<EntityId, OrderedFloat<f64>>,
    /// Transaction history for auditing
    transaction_log: Vec<EnergyTransaction>,
    /// Configuration parameters
    config: EnergyConfig,
}

/// Configuration for energy conservation laws
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    /// Total energy available in the system
    pub total_system_energy: OrderedFloat<f64>,
    /// Energy decay rate for idle entities (per second)
    pub decay_rate: OrderedFloat<f64>,
    /// Maximum energy transfer rate (per second)
    pub max_transfer_rate: OrderedFloat<f64>,
    /// Minimum energy threshold below which entities become dormant
    pub dormancy_threshold: OrderedFloat<f64>,
    /// Energy required for basic operations
    pub base_operation_cost: OrderedFloat<f64>,
}

/// Current energy state of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyState {
    /// Total energy in the system
    pub total_energy: OrderedFloat<f64>,
    /// Energy currently allocated to entities
    pub allocated_energy: OrderedFloat<f64>,
    /// Free energy available for allocation
    pub free_energy: OrderedFloat<f64>,
    /// Number of active entities
    pub active_entities: usize,
    /// Energy distribution statistics
    pub energy_distribution: EnergyDistribution,
}

/// Energy distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyDistribution {
    /// Mean energy per entity
    pub mean: OrderedFloat<f64>,
    /// Energy variance
    pub variance: OrderedFloat<f64>,
    /// Minimum energy allocation
    pub min: OrderedFloat<f64>,
    /// Maximum energy allocation
    pub max: OrderedFloat<f64>,
}

/// Energy transaction between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyTransaction {
    /// Source entity (None for system allocation)
    pub from: Option<EntityId>,
    /// Destination entity
    pub to: EntityId,
    /// Amount of energy transferred
    pub amount: OrderedFloat<f64>,
    /// Unique transaction identifier
    pub transaction_id: Uuid,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
}

/// Energy operation errors
#[derive(Debug, thiserror::Error)]
pub enum EnergyError {
    /// Insufficient energy available
    #[error("Insufficient energy: requested {requested}, available {available}")]
    InsufficientEnergy {
        requested: OrderedFloat<f64>,
        available: OrderedFloat<f64>,
    },
    
    /// Energy conservation violated
    #[error("Energy conservation violated: total energy changed from {before} to {after}")]
    ConservationViolated {
        before: OrderedFloat<f64>,
        after: OrderedFloat<f64>,
    },
    
    /// Transfer rate limit exceeded
    #[error("Transfer rate limit exceeded: {rate} > {limit}")]
    RateLimitExceeded {
        rate: OrderedFloat<f64>,
        limit: OrderedFloat<f64>,
    },
    
    /// Entity not found
    #[error("Entity {entity} not found in energy system")]
    EntityNotFound { entity: EntityId },
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            total_system_energy: OrderedFloat(1.0),
            decay_rate: OrderedFloat(0.01), // 1% per second
            max_transfer_rate: OrderedFloat(0.1), // 10% per second
            dormancy_threshold: OrderedFloat(0.05), // 5% energy minimum
            base_operation_cost: OrderedFloat(0.001), // 0.1% per operation
        }
    }
}

impl EnergyConservation {
    /// Create new energy conservation system with default configuration
    pub fn new() -> Self {
        let config = EnergyConfig::default();
        let total_energy = config.total_system_energy;
        
        Self {
            total_energy,
            allocations: HashMap::new(),
            transaction_log: Vec::new(),
            config,
        }
    }
    
    /// Configure energy laws from schema
    pub fn configure_from_schema(&mut self, schema: &serde_yaml::Value) -> Result<()> {
        if let Some(total_energy) = schema.get("total_system_energy") {
            if let Some(energy_val) = total_energy.as_f64() {
                self.config.total_system_energy = OrderedFloat(energy_val);
                self.total_energy = OrderedFloat(energy_val);
            }
        }
        
        if let Some(allocation_rules) = schema.get("allocation_rules") {
            if let Some(rules) = allocation_rules.as_sequence() {
                for rule in rules {
                    if let Some(name) = rule.get("name").and_then(|n| n.as_str()) {
                        match name {
                            "energy_decay" => {
                                if let Some(law) = rule.get("law").and_then(|l| l.as_str()) {
                                    // Parse decay rate from law string
                                    if let Some(rate_str) = law.split_whitespace().find(|s| s.starts_with("0.")) {
                                        if let Ok(rate) = rate_str.parse::<f64>() {
                                            self.config.decay_rate = OrderedFloat(rate);
                                        }
                                    }
                                }
                            }
                            "energy_transfer" => {
                                if let Some(law) = rule.get("law").and_then(|l| l.as_str()) {
                                    // Parse transfer rate from law string
                                    if let Some(rate_str) = law.split('=').nth(1) {
                                        if let Some(rate_val) = rate_str.split('/').next() {
                                            if let Ok(rate) = rate_val.parse::<f64>() {
                                                self.config.max_transfer_rate = OrderedFloat(rate);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        debug!("Energy conservation configured: {:?}", self.config);
        Ok(())
    }
    
    /// Allocate energy to an entity
    pub async fn allocate_energy(&mut self, entity: EntityId, amount: OrderedFloat<f64>) -> Result<(), EnergyError> {
        let current_allocation = self.allocations.get(&entity).copied().unwrap_or(OrderedFloat(0.0));
        let total_allocated = self.get_total_allocated();
        let available = self.total_energy - total_allocated;
        
        if amount > available {
            return Err(EnergyError::InsufficientEnergy {
                requested: amount,
                available,
            });
        }
        
        // Record transaction
        let transaction = EnergyTransaction {
            from: None, // System allocation
            to: entity,
            amount,
            transaction_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };
        
        self.allocations.insert(entity, current_allocation + amount);
        self.transaction_log.push(transaction);
        
        self.verify_conservation()?;
        
        debug!("Allocated {} energy to entity {}", amount, entity);
        Ok(())
    }
    
    /// Execute energy transfer between entities
    pub async fn execute_transaction(&mut self, transaction: EnergyTransaction) -> Result<(), EnergyError> {
        // Validate source has sufficient energy
        if let Some(from_entity) = transaction.from {
            let source_energy = self.allocations.get(&from_entity)
                .copied()
                .ok_or(EnergyError::EntityNotFound { entity: from_entity })?;
            
            if source_energy < transaction.amount {
                return Err(EnergyError::InsufficientEnergy {
                    requested: transaction.amount,
                    available: source_energy,
                });
            }
            
            // Deduct from source
            self.allocations.insert(from_entity, source_energy - transaction.amount);
        }
        
        // Add to destination
        let dest_energy = self.allocations.get(&transaction.to).copied().unwrap_or(OrderedFloat(0.0));
        self.allocations.insert(transaction.to, dest_energy + transaction.amount);
        
        // Record transaction
        self.transaction_log.push(transaction.clone());
        
        self.verify_conservation()?;
        
        debug!("Energy transaction completed: {} -> {}, amount: {}", 
               transaction.from.map(|e| e.to_string()).unwrap_or("system".to_string()),
               transaction.to, 
               transaction.amount);
        
        Ok(())
    }
    
    /// Apply energy decay to idle entities
    pub async fn apply_decay(&mut self, delta_time: f64) -> Result<(), EnergyError> {
        let decay_amount = self.config.decay_rate * OrderedFloat(delta_time);
        
        for (entity, energy) in self.allocations.iter_mut() {
            let new_energy = (*energy - decay_amount).max(OrderedFloat(0.0));
            *energy = new_energy;
            
            if new_energy < self.config.dormancy_threshold {
                debug!("Entity {} entering dormancy due to low energy: {}", entity, new_energy);
            }
        }
        
        // Remove entities with zero energy
        self.allocations.retain(|_, energy| *energy > OrderedFloat(0.0));
        
        Ok(())
    }
    
    /// Get current energy allocation for an entity
    pub fn get_entity_energy(&self, entity: EntityId) -> OrderedFloat<f64> {
        self.allocations.get(&entity).copied().unwrap_or(OrderedFloat(0.0))
    }
    
    /// Get current energy state
    pub async fn get_state(&self) -> EnergyState {
        let allocated_energy = self.get_total_allocated();
        let free_energy = self.total_energy - allocated_energy;
        let active_entities = self.allocations.len();
        
        let energy_distribution = if active_entities > 0 {
            let energies: Vec<OrderedFloat<f64>> = self.allocations.values().copied().collect();
            let mean = energies.iter().sum::<OrderedFloat<f64>>() / OrderedFloat(active_entities as f64);
            let variance = energies.iter()
                .map(|e| (*e - mean) * (*e - mean))
                .sum::<OrderedFloat<f64>>() / OrderedFloat(active_entities as f64);
            let min = energies.iter().copied().min().unwrap_or(OrderedFloat(0.0));
            let max = energies.iter().copied().max().unwrap_or(OrderedFloat(0.0));
            
            EnergyDistribution { mean, variance, min, max }
        } else {
            EnergyDistribution {
                mean: OrderedFloat(0.0),
                variance: OrderedFloat(0.0),
                min: OrderedFloat(0.0),
                max: OrderedFloat(0.0),
            }
        };
        
        EnergyState {
            total_energy: self.total_energy,
            allocated_energy,
            free_energy,
            active_entities,
            energy_distribution,
        }
    }
    
    /// Get transaction history
    pub fn get_transaction_history(&self) -> &[EnergyTransaction] {
        &self.transaction_log
    }
    
    /// Verify energy conservation invariant
    fn verify_conservation(&self) -> Result<(), EnergyError> {
        let allocated = self.get_total_allocated();
        
        // Allow small floating point errors
        let epsilon = OrderedFloat(1e-10);
        if (allocated - self.total_energy).abs() > epsilon && allocated > self.total_energy {
            return Err(EnergyError::ConservationViolated {
                before: self.total_energy,
                after: allocated,
            });
        }
        
        Ok(())
    }
    
    /// Calculate total allocated energy
    fn get_total_allocated(&self) -> OrderedFloat<f64> {
        self.allocations.values().sum()
    }
}

impl Default for EnergyConservation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_energy_allocation() {
        let mut energy_system = EnergyConservation::new();
        let entity = EntityId::new();
        
        // Allocate some energy
        let result = energy_system.allocate_energy(entity, OrderedFloat(0.3)).await;
        assert!(result.is_ok());
        
        // Check allocation
        let entity_energy = energy_system.get_entity_energy(entity);
        assert_eq!(entity_energy, OrderedFloat(0.3));
        
        // Check total conservation
        let state = energy_system.get_state().await;
        assert_eq!(state.allocated_energy, OrderedFloat(0.3));
        assert_eq!(state.free_energy, OrderedFloat(0.7));
    }
    
    #[tokio::test]
    async fn test_energy_transfer() {
        let mut energy_system = EnergyConservation::new();
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();
        
        // Allocate energy to first entity
        energy_system.allocate_energy(entity1, OrderedFloat(0.5)).await.unwrap();
        
        // Transfer some energy
        let transaction = EnergyTransaction {
            from: Some(entity1),
            to: entity2,
            amount: OrderedFloat(0.2),
            transaction_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };
        
        let result = energy_system.execute_transaction(transaction).await;
        assert!(result.is_ok());
        
        // Check final allocations
        assert_eq!(energy_system.get_entity_energy(entity1), OrderedFloat(0.3));
        assert_eq!(energy_system.get_entity_energy(entity2), OrderedFloat(0.2));
        
        // Verify conservation
        let state = energy_system.get_state().await;
        assert_eq!(state.allocated_energy, OrderedFloat(0.5));
    }
    
    #[tokio::test]
    async fn test_insufficient_energy() {
        let mut energy_system = EnergyConservation::new();
        let entity = EntityId::new();
        
        // Try to allocate more energy than available
        let result = energy_system.allocate_energy(entity, OrderedFloat(1.5)).await;
        assert!(matches!(result, Err(EnergyError::InsufficientEnergy { .. })));
    }
    
    #[tokio::test]
    async fn test_energy_decay() {
        let mut energy_system = EnergyConservation::new();
        let entity = EntityId::new();
        
        // Allocate energy
        energy_system.allocate_energy(entity, OrderedFloat(0.5)).await.unwrap();
        
        // Apply decay for 1 second
        energy_system.apply_decay(1.0).await.unwrap();
        
        // Energy should have decayed by 1%
        let remaining_energy = energy_system.get_entity_energy(entity);
        assert!((remaining_energy.0 - 0.495).abs() < 1e-10);
    }
}