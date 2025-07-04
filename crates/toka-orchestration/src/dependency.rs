//! Dependency resolution for agent spawning.
//!
//! This module provides functionality to analyze agent dependencies and determine
//! the optimal order for spawning agents to respect dependency constraints while
//! maximizing parallelism where possible.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use tracing::debug;

use crate::{AgentConfig, AgentPriority};

/// Dependency resolver for agent spawning order.
pub struct DependencyResolver {
    /// Dependency graph mapping agent names to their dependencies
    dependency_graph: HashMap<String, HashSet<String>>,
    /// Reverse dependency graph (dependents)
    #[allow(dead_code)]
    reverse_graph: HashMap<String, HashSet<String>>,
    /// Agent priorities
    priorities: HashMap<String, AgentPriority>,
    /// Optional dependencies (nice-to-have but not blocking)
    optional_deps: HashMap<String, HashSet<String>>,
}

/// Dependency resolution result.
#[derive(Debug, Clone)]
pub struct DependencyResolution {
    /// Agents that can be spawned immediately (no dependencies)
    pub immediate: Vec<String>,
    /// Agents grouped by spawn waves (each wave can be spawned in parallel)
    pub waves: Vec<Vec<String>>,
    /// Agents that have circular dependencies or other issues
    pub problematic: Vec<String>,
}

/// Dependency analysis result.
#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    /// Total number of agents
    pub total_agents: usize,
    /// Number of agents with no dependencies
    pub no_dependencies: usize,
    /// Number of agents with only optional dependencies
    pub optional_only: usize,
    /// Maximum dependency depth
    pub max_depth: usize,
    /// Agents with circular dependencies
    pub circular_dependencies: Vec<String>,
    /// Critical path agents (must be spawned first)
    pub critical_path: Vec<String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver from agent configurations.
    pub fn new(agents: &[AgentConfig]) -> Result<Self> {
        let mut dependency_graph = HashMap::new();
        let mut reverse_graph = HashMap::new();
        let mut priorities = HashMap::new();
        let mut optional_deps = HashMap::new();

        // Build dependency graphs
        for agent in agents {
            let agent_name = &agent.metadata.name;
            
            // Store priority
            priorities.insert(agent_name.clone(), agent.spec.priority.clone());
            
            // Initialize dependency sets
            dependency_graph.insert(agent_name.clone(), HashSet::new());
            reverse_graph.insert(agent_name.clone(), HashSet::new());
            optional_deps.insert(agent_name.clone(), HashSet::new());
            
            // Process required dependencies
            for (dep_name, _reason) in &agent.dependencies.required {
                dependency_graph.get_mut(agent_name).unwrap().insert(dep_name.clone());
                reverse_graph.entry(dep_name.clone()).or_default().insert(agent_name.clone());
            }
            
            // Process optional dependencies
            for (dep_name, _reason) in &agent.dependencies.optional {
                optional_deps.get_mut(agent_name).unwrap().insert(dep_name.clone());
            }
        }

        // Validate that all dependencies reference existing agents
        for (agent_name, deps) in &dependency_graph {
            for dep in deps {
                if !dependency_graph.contains_key(dep) {
                    return Err(anyhow::anyhow!(
                        "Agent '{}' depends on non-existent agent '{}'",
                        agent_name, dep
                    ));
                }
            }
        }

        Ok(Self {
            dependency_graph,
            reverse_graph,
            priorities,
            optional_deps,
        })
    }

    /// Resolve the spawn order for all agents.
    pub fn resolve_all(&self) -> Result<DependencyResolution> {
        let all_agents: Vec<String> = self.dependency_graph.keys().cloned().collect();
        self.resolve_waves(&all_agents)
    }

    /// Resolve spawn order for a subset of agents.
    pub fn resolve_spawn_order(&self, agents: &[String]) -> Result<Vec<String>> {
        debug!("Resolving spawn order for {} agents", agents.len());

        // First, detect circular dependencies
        let circular_deps = self.detect_circular_dependencies(agents)?;
        if !circular_deps.is_empty() {
            return Err(anyhow::anyhow!(
                "Circular dependencies detected: {:?}",
                circular_deps
            ));
        }

        // Topological sort with priority consideration
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let agent_set: HashSet<String> = agents.iter().cloned().collect();

        // Helper function for DFS-based topological sort
        fn visit(
            agent: &str,
            dependency_graph: &HashMap<String, HashSet<String>>,
            priorities: &HashMap<String, AgentPriority>,
            agent_set: &HashSet<String>,
            visited: &mut HashSet<String>,
            visiting: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) -> Result<()> {
            if visiting.contains(agent) {
                return Err(anyhow::anyhow!("Circular dependency detected involving agent: {}", agent));
            }
            
            if visited.contains(agent) {
                return Ok(());
            }

            visiting.insert(agent.to_string());

            // Visit dependencies first (only those in our agent set)
            if let Some(deps) = dependency_graph.get(agent) {
                // Sort dependencies by priority (critical first)
                let mut sorted_deps: Vec<_> = deps.iter()
                    .filter(|dep| agent_set.contains(*dep))
                    .collect();
                
                sorted_deps.sort_by(|a, b| {
                    let priority_a = priorities.get(*a).unwrap_or(&AgentPriority::Low);
                    let priority_b = priorities.get(*b).unwrap_or(&AgentPriority::Low);
                    priority_order(priority_a).cmp(&priority_order(priority_b))
                });

                for dep in sorted_deps {
                    visit(dep, dependency_graph, priorities, agent_set, visited, visiting, result)?;
                }
            }

            visiting.remove(agent);
            visited.insert(agent.to_string());
            result.push(agent.to_string());
            Ok(())
        }

        // Sort agents by priority for deterministic ordering
        let mut sorted_agents = agents.to_vec();
        sorted_agents.sort_by(|a, b| {
            let priority_a = self.priorities.get(a).unwrap_or(&AgentPriority::Low);
            let priority_b = self.priorities.get(b).unwrap_or(&AgentPriority::Low);
            priority_order(priority_a).cmp(&priority_order(priority_b))
        });

        for agent in &sorted_agents {
            visit(
                agent,
                &self.dependency_graph,
                &self.priorities,
                &agent_set,
                &mut visited,
                &mut visiting,
                &mut result,
            )?;
        }

        debug!("Resolved spawn order: {:?}", result);
        Ok(result)
    }

    /// Resolve agents into spawn waves for parallel execution.
    pub fn resolve_waves(&self, agents: &[String]) -> Result<DependencyResolution> {
        debug!("Resolving spawn waves for {} agents", agents.len());

        // First get the basic spawn order
        let ordered_agents = self.resolve_spawn_order(agents)?;
        
        // Group agents into waves where agents in the same wave can be spawned in parallel
        let mut waves = Vec::new();
        let mut current_wave = Vec::new();
        let mut completed = HashSet::new();
        let agent_set: HashSet<String> = agents.iter().cloned().collect();

        let empty_deps = HashSet::new();
        for agent in &ordered_agents {
            // Check if all dependencies are satisfied
            let deps = self.dependency_graph.get(agent).unwrap_or(&empty_deps);
            let deps_satisfied = deps.iter()
                .filter(|dep| agent_set.contains(*dep))
                .all(|dep| completed.contains(dep));

            if deps_satisfied {
                // Can add to current wave
                current_wave.push(agent.clone());
            } else {
                // Need to start a new wave
                if !current_wave.is_empty() {
                    // Mark agents in current wave as completed
                    for agent in &current_wave {
                        completed.insert(agent.clone());
                    }
                    waves.push(current_wave.clone());
                    current_wave.clear();
                }
                current_wave.push(agent.clone());
            }
        }

        // Add final wave if not empty
        if !current_wave.is_empty() {
            waves.push(current_wave);
        }

        // Determine immediate agents (no dependencies)
        let empty_set = HashSet::new();
        let immediate = agents.iter()
            .filter(|agent| {
                let deps = self.dependency_graph.get(*agent).unwrap_or(&empty_set);
                deps.iter().filter(|dep| agent_set.contains(*dep)).count() == 0
            })
            .cloned()
            .collect();

        Ok(DependencyResolution {
            immediate,
            waves,
            problematic: Vec::new(), // We already handled circular dependencies
        })
    }

    /// Detect circular dependencies in the dependency graph.
    pub fn detect_circular_dependencies(&self, agents: &[String]) -> Result<Vec<String>> {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut circular = Vec::new();
        let agent_set: HashSet<String> = agents.iter().cloned().collect();

        fn dfs_cycle_detection(
            agent: &str,
            dependency_graph: &HashMap<String, HashSet<String>>,
            agent_set: &HashSet<String>,
            visiting: &mut HashSet<String>,
            visited: &mut HashSet<String>,
            circular: &mut Vec<String>,
        ) {
            if visited.contains(agent) {
                return;
            }
            
            if visiting.contains(agent) {
                circular.push(agent.to_string());
                return;
            }

            visiting.insert(agent.to_string());

            if let Some(deps) = dependency_graph.get(agent) {
                for dep in deps {
                    if agent_set.contains(dep) {
                        dfs_cycle_detection(dep, dependency_graph, agent_set, visiting, visited, circular);
                    }
                }
            }

            visiting.remove(agent);
            visited.insert(agent.to_string());
        }

        for agent in agents {
            if !visited.contains(agent) {
                dfs_cycle_detection(
                    agent,
                    &self.dependency_graph,
                    &agent_set,
                    &mut visiting,
                    &mut visited,
                    &mut circular,
                );
            }
        }

        Ok(circular)
    }

    /// Analyze the dependency structure.
    pub fn analyze_dependencies(&self, agents: &[String]) -> DependencyAnalysis {
        let total_agents = agents.len();
        let agent_set: HashSet<String> = agents.iter().cloned().collect();
        
        let empty_set = HashSet::new();
        let no_dependencies = agents.iter()
            .filter(|agent| {
                let deps = self.dependency_graph.get(*agent).unwrap_or(&empty_set);
                deps.iter().filter(|dep| agent_set.contains(*dep)).count() == 0
            })
            .count();

        let optional_only = agents.iter()
            .filter(|agent| {
                let required_deps = self.dependency_graph.get(*agent).unwrap_or(&empty_set);
                let required_count = required_deps.iter().filter(|dep| agent_set.contains(*dep)).count();
                let optional_deps = self.optional_deps.get(*agent).unwrap_or(&empty_set);
                let optional_count = optional_deps.iter().filter(|dep| agent_set.contains(*dep)).count();
                
                required_count == 0 && optional_count > 0
            })
            .count();

        let max_depth = self.calculate_max_depth(agents);
        let circular_dependencies = self.detect_circular_dependencies(agents).unwrap_or_default();
        
        let critical_path = agents.iter()
            .filter(|agent| {
                matches!(self.priorities.get(*agent), Some(AgentPriority::Critical))
            })
            .cloned()
            .collect();

        DependencyAnalysis {
            total_agents,
            no_dependencies,
            optional_only,
            max_depth,
            circular_dependencies,
            critical_path,
        }
    }

    /// Calculate the maximum dependency depth.
    fn calculate_max_depth(&self, agents: &[String]) -> usize {
        let agent_set: HashSet<String> = agents.iter().cloned().collect();
        let mut max_depth = 0;

        fn calculate_depth(
            agent: &str,
            dependency_graph: &HashMap<String, HashSet<String>>,
            agent_set: &HashSet<String>,
            visited: &mut HashSet<String>,
        ) -> usize {
            if visited.contains(agent) {
                return 0; // Prevent infinite recursion
            }
            
            visited.insert(agent.to_string());
            
            let empty_set = HashSet::new();
            let deps = dependency_graph.get(agent).unwrap_or(&empty_set);
            let max_dep_depth = deps.iter()
                .filter(|dep| agent_set.contains(*dep))
                .map(|dep| calculate_depth(dep, dependency_graph, agent_set, visited))
                .max()
                .unwrap_or(0);
            
            visited.remove(agent);
            max_dep_depth + 1
        }

        for agent in agents {
            let mut visited = HashSet::new();
            let depth = calculate_depth(agent, &self.dependency_graph, &agent_set, &mut visited);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Get agents that can be spawned immediately (no unsatisfied dependencies).
    pub fn get_ready_agents(&self, agents: &[String], completed: &HashSet<String>) -> Vec<String> {
        let agent_set: HashSet<String> = agents.iter().cloned().collect();
        let empty_set = HashSet::new();
        
        agents.iter()
            .filter(|agent| {
                if completed.contains(*agent) {
                    return false;
                }
                
                let deps = self.dependency_graph.get(*agent).unwrap_or(&empty_set);
                deps.iter()
                    .filter(|dep| agent_set.contains(*dep))
                    .all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect()
    }

    /// Check if an agent can be spawned given current completion state.
    pub fn can_spawn_agent(&self, agent: &str, completed: &HashSet<String>) -> bool {
        let empty_set = HashSet::new();
        let deps = self.dependency_graph.get(agent).unwrap_or(&empty_set);
        deps.iter().all(|dep| completed.contains(dep))
    }
}

/// Helper function to convert priority to ordering value.
fn priority_order(priority: &AgentPriority) -> u8 {
    match priority {
        AgentPriority::Critical => 0,
        AgentPriority::High => 1,
        AgentPriority::Medium => 2,
        AgentPriority::Low => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::{AgentConfig, AgentMetadata, AgentSpecConfig, AgentCapabilities, AgentObjective, AgentTasks, AgentDependencies, ReportingConfig, SecurityConfig, TaskConfig, TaskPriority, ReportingFrequency, ResourceLimits};

    fn create_test_agent(name: &str, priority: AgentPriority, deps: Vec<&str>) -> AgentConfig {
        let mut dependencies = HashMap::new();
        for dep in deps {
            dependencies.insert(dep.to_string(), "test dependency".to_string());
        }

        AgentConfig {
            metadata: AgentMetadata {
                name: name.to_string(),
                version: "v1.0".to_string(),
                created: "2024-01-01".to_string(),
                workstream: "test".to_string(),
                branch: "main".to_string(),
            },
            spec: AgentSpecConfig {
                name: name.to_string(),
                domain: "test".to_string(),
                priority,
            },
            capabilities: AgentCapabilities {
                primary: vec!["test".to_string()],
                secondary: vec![],
            },
            objectives: vec![AgentObjective {
                description: "Test objective".to_string(),
                deliverable: "Test deliverable".to_string(),
                validation: "Test validation".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![TaskConfig {
                    description: "Test task".to_string(),
                    priority: TaskPriority::Medium,
                }],
            },
            dependencies: AgentDependencies {
                required: dependencies,
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: ReportingFrequency::Daily,
                channels: vec!["test".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["test".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "100MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "1h".to_string(),
                },
            },
        }
    }

    #[test]
    fn test_dependency_resolver_simple() {
        let agents = vec![
            create_test_agent("a", AgentPriority::High, vec![]),
            create_test_agent("b", AgentPriority::Medium, vec!["a"]),
            create_test_agent("c", AgentPriority::Low, vec!["b"]),
        ];

        let resolver = DependencyResolver::new(&agents).unwrap();
        let order = resolver.resolve_spawn_order(&["a".to_string(), "b".to_string(), "c".to_string()]).unwrap();
        
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_dependency_resolver_parallel() {
        let agents = vec![
            create_test_agent("a", AgentPriority::Critical, vec![]),
            create_test_agent("b", AgentPriority::High, vec!["a"]),
            create_test_agent("c", AgentPriority::High, vec!["a"]),
            create_test_agent("d", AgentPriority::Medium, vec!["b", "c"]),
        ];

        let resolver = DependencyResolver::new(&agents).unwrap();
        let resolution = resolver.resolve_waves(&["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]).unwrap();
        
        assert_eq!(resolution.immediate, vec!["a"]);
        assert!(resolution.waves.len() >= 2);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let agents = vec![
            create_test_agent("a", AgentPriority::High, vec!["b"]),
            create_test_agent("b", AgentPriority::Medium, vec!["a"]),
        ];

        let resolver = DependencyResolver::new(&agents).unwrap();
        let circular = resolver.detect_circular_dependencies(&["a".to_string(), "b".to_string()]).unwrap();
        
        assert!(!circular.is_empty());
    }

    #[test]
    fn test_dependency_analysis() {
        let agents = vec![
            create_test_agent("a", AgentPriority::Critical, vec![]),
            create_test_agent("b", AgentPriority::High, vec!["a"]),
            create_test_agent("c", AgentPriority::Medium, vec![]),
        ];

        let resolver = DependencyResolver::new(&agents).unwrap();
        let analysis = resolver.analyze_dependencies(&["a".to_string(), "b".to_string(), "c".to_string()]);
        
        assert_eq!(analysis.total_agents, 3);
        assert_eq!(analysis.no_dependencies, 2);
        assert_eq!(analysis.critical_path.len(), 1);
    }
} 