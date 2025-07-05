//! Permission hierarchy management for capability delegation
//!
//! This module implements permission hierarchy functionality that allows
//! for parent-child permission relationships. This enables more sophisticated
//! delegation patterns where granting a parent permission automatically
//! grants all implied child permissions.

use crate::{PermissionHierarchy, DelegationError};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Simple in-memory permission hierarchy implementation
/// 
/// This implementation stores permission relationships in memory using
/// a directed graph structure. For production use, this should be backed
/// by a persistent store.
pub struct SimplePermissionHierarchy {
    /// Permission hierarchy graph: parent -> children
    hierarchy: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl SimplePermissionHierarchy {
    /// Create a new permission hierarchy
    pub fn new() -> Self {
        Self {
            hierarchy: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build a hierarchy from a configuration
    pub async fn from_config(config: Vec<(String, Vec<String>)>) -> Self {
        let hierarchy = Self::new();
        
        for (parent, children) in config {
            for child in children {
                if let Err(e) = hierarchy.add_implication(&parent, &child).await {
                    tracing::warn!(
                        parent = %parent,
                        child = %child,
                        error = %e,
                        "Failed to add permission implication"
                    );
                }
            }
        }
        
        hierarchy
    }

    /// Check for circular dependencies in the hierarchy
    async fn has_cycle(&self, parent: &str, child: &str) -> bool {
        let hierarchy = self.hierarchy.read().await;
        
        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut stack = vec![child.to_string()];
        
        while let Some(current) = stack.pop() {
            if current == parent {
                return true; // Found a cycle
            }
            
            if visited.contains(&current) {
                continue;
            }
            
            visited.insert(current.clone());
            
            if let Some(children) = hierarchy.get(&current) {
                for child in children {
                    if !visited.contains(child) {
                        stack.push(child.clone());
                    }
                }
            }
        }
        
        false
    }

    /// Get all permissions transitively implied by a permission
    async fn get_all_implied_permissions(&self, permission: &str) -> Result<HashSet<String>> {
        let hierarchy = self.hierarchy.read().await;
        let mut result = HashSet::new();
        let mut to_process = vec![permission.to_string()];
        
        while let Some(current) = to_process.pop() {
            if let Some(children) = hierarchy.get(&current) {
                for child in children {
                    if !result.contains(child) {
                        result.insert(child.clone());
                        to_process.push(child.clone());
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// Get statistics about the hierarchy
    pub async fn get_stats(&self) -> HierarchyStats {
        let hierarchy = self.hierarchy.read().await;
        
        let total_permissions = hierarchy.keys().len();
        let total_relationships = hierarchy.values()
            .map(|children| children.len())
            .sum();
        
        let max_depth = self.calculate_max_depth(&hierarchy);
        let leaf_permissions = hierarchy.keys()
            .filter(|perm| {
                // A permission is a leaf if it's not a parent of any other permission
                !hierarchy.values().any(|children| children.contains(*perm))
            })
            .count();
        
        HierarchyStats {
            total_permissions,
            total_relationships,
            max_depth,
            leaf_permissions,
        }
    }

    /// Calculate maximum depth of the hierarchy
    fn calculate_max_depth(&self, hierarchy: &HashMap<String, HashSet<String>>) -> usize {
        let mut max_depth = 0;
        
        for permission in hierarchy.keys() {
            let depth = self.calculate_depth_for_permission(permission, hierarchy, &mut HashSet::new());
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }

    /// Calculate depth for a specific permission
    fn calculate_depth_for_permission(
        &self,
        permission: &str,
        hierarchy: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
    ) -> usize {
        if visited.contains(permission) {
            return 0; // Avoid infinite recursion
        }
        
        visited.insert(permission.to_string());
        
        let mut max_child_depth = 0;
        if let Some(children) = hierarchy.get(permission) {
            for child in children {
                let child_depth = self.calculate_depth_for_permission(child, hierarchy, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }
        
        visited.remove(permission);
        1 + max_child_depth
    }
}

impl Default for SimplePermissionHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionHierarchy for SimplePermissionHierarchy {
    async fn implies(&self, parent: &str, child: &str) -> Result<bool> {
        let implied_permissions = self.get_all_implied_permissions(parent).await?;
        Ok(implied_permissions.contains(child))
    }

    async fn get_implied_permissions(&self, permission: &str) -> Result<Vec<String>> {
        let implied = self.get_all_implied_permissions(permission).await?;
        Ok(implied.into_iter().collect())
    }

    async fn add_implication(&self, parent: &str, child: &str) -> Result<()> {
        // Check for cycles before adding
        if self.has_cycle(parent, child).await {
            return Err(anyhow::anyhow!(
                "Adding implication {} -> {} would create a cycle",
                parent, child
            ));
        }

        let mut hierarchy = self.hierarchy.write().await;
        hierarchy.entry(parent.to_string())
            .or_insert_with(HashSet::new)
            .insert(child.to_string());

        debug!(
            parent = %parent,
            child = %child,
            "Added permission implication"
        );

        Ok(())
    }

    async fn remove_implication(&self, parent: &str, child: &str) -> Result<()> {
        let mut hierarchy = self.hierarchy.write().await;
        
        if let Some(children) = hierarchy.get_mut(parent) {
            children.remove(child);
            
            // Remove parent entry if it has no children
            if children.is_empty() {
                hierarchy.remove(parent);
            }

            debug!(
                parent = %parent,
                child = %child,
                "Removed permission implication"
            );
        }

        Ok(())
    }

    async fn get_hierarchy(&self) -> Result<HashMap<String, Vec<String>>> {
        let hierarchy = self.hierarchy.read().await;
        let result = hierarchy.iter()
            .map(|(parent, children)| {
                (parent.clone(), children.iter().cloned().collect())
            })
            .collect();

        Ok(result)
    }
}

/// Statistics about the permission hierarchy
#[derive(Debug, Clone)]
pub struct HierarchyStats {
    /// Total number of permissions in the hierarchy
    pub total_permissions: usize,
    /// Total number of parent-child relationships
    pub total_relationships: usize,
    /// Maximum depth of the hierarchy
    pub max_depth: usize,
    /// Number of leaf permissions (permissions with no children)
    pub leaf_permissions: usize,
}

/// Common permission hierarchies that can be used as templates
pub mod presets {
    use super::*;

    /// Create a file system permission hierarchy
    /// 
    /// Hierarchy: admin -> write -> read
    pub async fn filesystem_hierarchy() -> SimplePermissionHierarchy {
        SimplePermissionHierarchy::from_config(vec![
            ("admin".to_string(), vec!["write".to_string(), "read".to_string()]),
            ("write".to_string(), vec!["read".to_string()]),
        ]).await
    }

    /// Create a database permission hierarchy
    /// 
    /// Hierarchy: db_admin -> db_write -> db_read
    pub async fn database_hierarchy() -> SimplePermissionHierarchy {
        SimplePermissionHierarchy::from_config(vec![
            ("db_admin".to_string(), vec!["db_write".to_string(), "db_read".to_string()]),
            ("db_write".to_string(), vec!["db_read".to_string()]),
        ]).await
    }

    /// Create a comprehensive system hierarchy
    /// 
    /// Complex multi-level hierarchy for system administration
    pub async fn system_hierarchy() -> SimplePermissionHierarchy {
        SimplePermissionHierarchy::from_config(vec![
            // System administration
            ("system_admin".to_string(), vec![
                "user_admin".to_string(),
                "service_admin".to_string(),
                "network_admin".to_string(),
            ]),
            
            // User management
            ("user_admin".to_string(), vec![
                "user_write".to_string(),
                "user_read".to_string(),
            ]),
            ("user_write".to_string(), vec!["user_read".to_string()]),
            
            // Service management
            ("service_admin".to_string(), vec![
                "service_write".to_string(),
                "service_read".to_string(),
            ]),
            ("service_write".to_string(), vec!["service_read".to_string()]),
            
            // Network management
            ("network_admin".to_string(), vec![
                "network_write".to_string(),
                "network_read".to_string(),
            ]),
            ("network_write".to_string(), vec!["network_read".to_string()]),
        ]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_hierarchy() {
        let hierarchy = SimplePermissionHierarchy::new();
        
        // Add some implications
        hierarchy.add_implication("admin", "write").await.unwrap();
        hierarchy.add_implication("admin", "read").await.unwrap();
        hierarchy.add_implication("write", "read").await.unwrap();
        
        // Test direct implications
        assert!(hierarchy.implies("admin", "write").await.unwrap());
        assert!(hierarchy.implies("admin", "read").await.unwrap());
        assert!(hierarchy.implies("write", "read").await.unwrap());
        
        // Test transitive implications
        assert!(hierarchy.implies("admin", "read").await.unwrap());
        
        // Test non-implications
        assert!(!hierarchy.implies("read", "write").await.unwrap());
        assert!(!hierarchy.implies("write", "admin").await.unwrap());
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let hierarchy = SimplePermissionHierarchy::new();
        
        // Add valid hierarchy
        hierarchy.add_implication("admin", "write").await.unwrap();
        hierarchy.add_implication("write", "read").await.unwrap();
        
        // Try to create a cycle - should fail
        let result = hierarchy.add_implication("read", "admin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_implication() {
        let hierarchy = SimplePermissionHierarchy::new();
        
        hierarchy.add_implication("admin", "write").await.unwrap();
        hierarchy.add_implication("admin", "read").await.unwrap();
        
        assert!(hierarchy.implies("admin", "write").await.unwrap());
        
        hierarchy.remove_implication("admin", "write").await.unwrap();
        
        assert!(!hierarchy.implies("admin", "write").await.unwrap());
        assert!(hierarchy.implies("admin", "read").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_hierarchy() {
        let hierarchy = SimplePermissionHierarchy::new();
        
        hierarchy.add_implication("admin", "write").await.unwrap();
        hierarchy.add_implication("admin", "read").await.unwrap();
        hierarchy.add_implication("write", "read").await.unwrap();
        
        let graph = hierarchy.get_hierarchy().await.unwrap();
        
        assert_eq!(graph.get("admin").unwrap().len(), 2);
        assert_eq!(graph.get("write").unwrap().len(), 1);
        assert!(graph.get("read").is_none());
    }

    #[tokio::test]
    async fn test_filesystem_preset() {
        let hierarchy = presets::filesystem_hierarchy().await;
        
        assert!(hierarchy.implies("admin", "write").await.unwrap());
        assert!(hierarchy.implies("admin", "read").await.unwrap());
        assert!(hierarchy.implies("write", "read").await.unwrap());
        
        assert!(!hierarchy.implies("read", "write").await.unwrap());
    }

    #[tokio::test]
    async fn test_stats() {
        let hierarchy = SimplePermissionHierarchy::new();
        
        hierarchy.add_implication("admin", "write").await.unwrap();
        hierarchy.add_implication("admin", "read").await.unwrap();
        hierarchy.add_implication("write", "read").await.unwrap();
        
        let stats = hierarchy.get_stats().await;
        
        assert_eq!(stats.total_permissions, 2); // admin, write (read has no children)
        assert_eq!(stats.total_relationships, 3);
        assert_eq!(stats.max_depth, 2);
        assert_eq!(stats.leaf_permissions, 1); // read is a leaf
    }
} 