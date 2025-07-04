//! Agent configuration loading and validation.
//!
//! This module provides functionality to load agent configurations from YAML files
//! and validate them against the expected schema. It handles both individual agent
//! configurations and bulk loading from directories.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::AgentConfig;

/// Main orchestration configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Agent configurations
    pub agents: Vec<AgentConfig>,
    /// Global timeout for orchestration
    pub global_timeout: Duration,
    /// Maximum number of concurrent agents
    pub max_concurrent_agents: usize,
}

/// Agent configuration loader.
pub struct AgentConfigLoader {
    /// Base directory for agent configurations
    base_dir: PathBuf,
    /// Loaded configurations cache
    cache: HashMap<String, AgentConfig>,
}

impl AgentConfigLoader {
    /// Create a new agent configuration loader.
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
        }
    }

    /// Load all agent configurations from the base directory.
    pub fn load_all(&mut self) -> Result<Vec<AgentConfig>> {
        info!("Loading agent configurations from: {}", self.base_dir.display());

        if !self.base_dir.exists() {
            return Err(anyhow::anyhow!(
                "Agent configuration directory does not exist: {}",
                self.base_dir.display()
            ));
        }

        let mut configs = Vec::new();

        // Read all YAML files in the directory
        let entries = fs::read_dir(&self.base_dir)
            .with_context(|| format!("Failed to read directory: {}", self.base_dir.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            // Skip non-YAML files
            if !path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                continue;
            }

            match self.load_config_file(&path) {
                Ok(config) => {
                    debug!("Loaded agent configuration: {}", config.metadata.name);
                    configs.push(config);
                }
                Err(e) => {
                    warn!("Failed to load agent configuration from {}: {}", path.display(), e);
                    // Continue loading other configs instead of failing completely
                }
            }
        }

        if configs.is_empty() {
            warn!("No valid agent configurations found in: {}", self.base_dir.display());
        } else {
            info!("Loaded {} agent configurations", configs.len());
        }

        Ok(configs)
    }

    /// Load a single agent configuration file.
    pub fn load_config_file(&mut self, path: &Path) -> Result<AgentConfig> {
        debug!("Loading agent configuration file: {}", path.display());

        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let config: AgentConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse YAML file: {}", path.display()))?;

        // Validate the configuration
        self.validate_config(&config)
            .with_context(|| format!("Invalid configuration in file: {}", path.display()))?;

        // Cache the configuration
        self.cache.insert(config.metadata.name.clone(), config.clone());

        Ok(config)
    }

    /// Get a cached configuration by name.
    pub fn get_config(&self, name: &str) -> Option<&AgentConfig> {
        self.cache.get(name)
    }

    /// Validate an agent configuration.
    fn validate_config(&self, config: &AgentConfig) -> Result<()> {
        // Validate metadata
        if config.metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Agent name cannot be empty"));
        }

        if config.metadata.version.is_empty() {
            return Err(anyhow::anyhow!("Agent version cannot be empty"));
        }

        if config.metadata.workstream.is_empty() {
            return Err(anyhow::anyhow!("Agent workstream cannot be empty"));
        }

        // Validate spec
        if config.spec.name.is_empty() {
            return Err(anyhow::anyhow!("Agent spec name cannot be empty"));
        }

        if config.spec.domain.is_empty() {
            return Err(anyhow::anyhow!("Agent domain cannot be empty"));
        }

        // Validate capabilities
        if config.capabilities.primary.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one primary capability"));
        }

        // Validate objectives
        if config.objectives.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one objective"));
        }

        for objective in &config.objectives {
            if objective.description.is_empty() {
                return Err(anyhow::anyhow!("Objective description cannot be empty"));
            }
            if objective.deliverable.is_empty() {
                return Err(anyhow::anyhow!("Objective deliverable cannot be empty"));
            }
        }

        // Validate tasks
        if config.tasks.default.is_empty() {
            return Err(anyhow::anyhow!("Agent must have at least one default task"));
        }

        for task in &config.tasks.default {
            if task.description.is_empty() {
                return Err(anyhow::anyhow!("Task description cannot be empty"));
            }
        }

        // Validate security config
        if config.security.capabilities_required.is_empty() {
            return Err(anyhow::anyhow!("Agent must declare required capabilities"));
        }

        // Validate resource limits
        if config.security.resource_limits.max_memory.is_empty() {
            return Err(anyhow::anyhow!("Agent must specify max memory limit"));
        }

        if config.security.resource_limits.max_cpu.is_empty() {
            return Err(anyhow::anyhow!("Agent must specify max CPU limit"));
        }

        if config.security.resource_limits.timeout.is_empty() {
            return Err(anyhow::anyhow!("Agent must specify timeout"));
        }

        Ok(())
    }
}

impl OrchestrationConfig {
    /// Load orchestration configuration from a directory.
    pub fn from_directory(dir: impl AsRef<Path>) -> Result<Self> {
        let mut loader = AgentConfigLoader::new(dir);
        let agents = loader.load_all()?;

        Ok(Self {
            agents,
            global_timeout: Duration::from_secs(3600), // 1 hour default
            max_concurrent_agents: 10,
        })
    }

    /// Load orchestration configuration from a directory with custom settings.
    pub fn from_directory_with_settings(
        dir: impl AsRef<Path>,
        global_timeout: Duration,
        max_concurrent_agents: usize,
    ) -> Result<Self> {
        let mut loader = AgentConfigLoader::new(dir);
        let agents = loader.load_all()?;

        Ok(Self {
            agents,
            global_timeout,
            max_concurrent_agents,
        })
    }

    /// Get agent configuration by name.
    pub fn get_agent_config(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.iter().find(|config| config.metadata.name == name)
    }

    /// Get agents by priority.
    pub fn get_agents_by_priority(&self, priority: crate::AgentPriority) -> Vec<&AgentConfig> {
        self.agents.iter()
            .filter(|config| config.spec.priority == priority)
            .collect()
    }

    /// Get agents by workstream.
    pub fn get_agents_by_workstream(&self, workstream: &str) -> Vec<&AgentConfig> {
        self.agents.iter()
            .filter(|config| config.metadata.workstream == workstream)
            .collect()
    }

    /// Validate all agent configurations.
    pub fn validate(&self) -> Result<()> {
        let loader = AgentConfigLoader::new(".");
        
        for config in &self.agents {
            loader.validate_config(config)?;
        }

        // Validate no duplicate names
        let mut names = std::collections::HashSet::new();
        for config in &self.agents {
            if !names.insert(&config.metadata.name) {
                return Err(anyhow::anyhow!("Duplicate agent name: {}", config.metadata.name));
            }
        }

        Ok(())
    }
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            agents: Vec::new(),
            global_timeout: Duration::from_secs(3600),
            max_concurrent_agents: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_agent_config_loader_validation() {
        use crate::*;
        let loader = AgentConfigLoader::new(".");
        
        // Test invalid config - empty name
        let invalid_config = AgentConfig {
            metadata: AgentMetadata {
                name: "".to_string(),
                version: "v1.0".to_string(),
                created: "2024-01-01".to_string(),
                workstream: "test".to_string(),
                branch: "main".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Test Agent".to_string(),
                domain: "test".to_string(),
                priority: crate::AgentPriority::Medium,
            },
            capabilities: AgentCapabilities {
                primary: vec!["testing".to_string()],
                secondary: vec![],
            },
            objectives: vec![AgentObjective {
                description: "Test objective".to_string(),
                deliverable: "Test deliverable".to_string(),
                validation: "Test validation".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![crate::TaskConfig {
                    description: "Test task".to_string(),
                    priority: crate::TaskPriority::Medium,
                }],
            },
            dependencies: AgentDependencies {
                required: HashMap::new(),
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: crate::ReportingFrequency::Daily,
                channels: vec!["test".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["test".to_string()],
                resource_limits: crate::ResourceLimits {
                    max_memory: "100MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "1h".to_string(),
                },
            },
        };

        assert!(loader.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_orchestration_config_validation() {
        let config = OrchestrationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_config_loading_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("configs");
        fs::create_dir_all(&config_dir).unwrap();

        // Create a valid config file
        let config_yaml = r#"
metadata:
  name: "test-agent"
  version: "v1.0"
  created: "2024-01-01"
  workstream: "testing"
  branch: "main"

spec:
  name: "Test Agent"
  domain: "testing"
  priority: "medium"

capabilities:
  primary:
    - "testing"
  secondary: []

objectives:
  - description: "Test objective"
    deliverable: "Test deliverable"
    validation: "Test validation"

tasks:
  default:
    - description: "Test task"
      priority: "medium"

dependencies:
  required: {}
  optional: {}

reporting:
  frequency: "daily"
  channels:
    - "test"
  metrics: {}

security:
  sandbox: true
  capabilities_required:
    - "test"
  resource_limits:
    max_memory: "100MB"
    max_cpu: "50%"
    timeout: "1h"
"#;

        fs::write(config_dir.join("test-agent.yaml"), config_yaml).unwrap();

        let config = OrchestrationConfig::from_directory(&config_dir);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].metadata.name, "test-agent");
    }
} 