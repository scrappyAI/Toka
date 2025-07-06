//! Capability-based access control system
//!
//! Provides fine-grained permissions for tool operations, enabling secure
//! execution while maintaining flexibility for dynamic tool generation.

use std::collections::HashSet;
use std::fmt;
use serde::{Deserialize, Serialize};

/// Individual capability that can be granted to tools or sessions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// File system operations
    FileRead(FileAccess),
    FileWrite(FileAccess),
    FileExecute(FileAccess),
    
    /// Network operations
    NetworkConnect(NetworkAccess),
    NetworkBind(NetworkAccess),
    
    /// Process management
    ProcessSpawn,
    ProcessKill,
    ProcessSignal,
    
    /// System information
    SystemInfo,
    SystemMetrics,
    
    /// Dynamic execution
    CodeGeneration,
    CodeExecution(ExecutionMode),
    
    /// Tool management
    ToolRegistration,
    ToolDiscovery,
    
    /// Agent operations
    AgentSpawn,
    AgentCommunication,
    
    /// Administrative
    CapabilityManagement,
    ResourceManagement,
}

/// File access patterns for capability control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileAccess {
    /// Access to specific file
    Specific(String),
    /// Access to directory and subdirectories
    Directory(String),
    /// Access to files matching pattern
    Pattern(String),
    /// Full filesystem access (admin only)
    Global,
}

/// Network access patterns for capability control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkAccess {
    /// Access to specific host and port
    Specific { host: String, port: u16 },
    /// Access to any port on specific host
    Host(String),
    /// Access to specific port on any host
    Port(u16),
    /// Access to local network only
    Local,
    /// Full network access
    Global,
}

/// Code execution modes for dynamic capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// WebAssembly sandbox
    Wasm,
    /// Native code with restrictions
    Native,
    /// Scripting languages (Python, etc.)
    Script(String),
}

/// Set of capabilities granted to a tool or session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }
    
    /// Create capability set with predefined capabilities
    pub fn with_capabilities(capabilities: Vec<Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }
    
    /// Add capability to the set
    pub fn add(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }
    
    /// Remove capability from the set
    pub fn remove(&mut self, capability: &Capability) {
        self.capabilities.remove(capability);
    }
    
    /// Check if capability is granted
    pub fn contains(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if file access is allowed
    pub fn allows_file_access(&self, operation: &str, path: &str) -> bool {
        let file_capability = match operation {
            "read" => Capability::FileRead(FileAccess::Global),
            "write" => Capability::FileWrite(FileAccess::Global),
            "execute" => Capability::FileExecute(FileAccess::Global),
            _ => return false,
        };
        
        // Check for global access first
        if self.capabilities.contains(&file_capability) {
            return true;
        }
        
        // Check for specific path permissions
        for cap in &self.capabilities {
            match cap {
                Capability::FileRead(access) | 
                Capability::FileWrite(access) | 
                Capability::FileExecute(access) => {
                    if self.matches_file_access(access, path, operation, cap) {
                        return true;
                    }
                }
                _ => continue,
            }
        }
        
        false
    }
    
    /// Check if network access is allowed
    pub fn allows_network_access(&self, operation: &str, host: &str, port: u16) -> bool {
        let network_capability = match operation {
            "connect" => Capability::NetworkConnect(NetworkAccess::Global),
            "bind" => Capability::NetworkBind(NetworkAccess::Global),
            _ => return false,
        };
        
        // Check for global access
        if self.capabilities.contains(&network_capability) {
            return true;
        }
        
        // Check for specific network permissions
        for cap in &self.capabilities {
            match cap {
                Capability::NetworkConnect(access) | 
                Capability::NetworkBind(access) => {
                    if self.matches_network_access(access, host, port, operation, cap) {
                        return true;
                    }
                }
                _ => continue,
            }
        }
        
        false
    }
    
    /// Merge with another capability set
    pub fn merge(&mut self, other: &CapabilitySet) {
        for capability in &other.capabilities {
            self.capabilities.insert(capability.clone());
        }
    }
    
    /// Get intersection with another capability set
    pub fn intersection(&self, other: &CapabilitySet) -> CapabilitySet {
        let intersection = self.capabilities
            .intersection(&other.capabilities)
            .cloned()
            .collect();
        
        CapabilitySet {
            capabilities: intersection,
        }
    }
    
    /// Check if this set is a subset of another
    pub fn is_subset_of(&self, other: &CapabilitySet) -> bool {
        self.capabilities.is_subset(&other.capabilities)
    }
    
    /// Get iterator over capabilities
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }
    
    /// Get number of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }
    
    /// Check if capability set is empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
    
    fn matches_file_access(
        &self, 
        access: &FileAccess, 
        path: &str, 
        operation: &str,
        capability: &Capability
    ) -> bool {
        // Verify the operation matches the capability type
        let operation_matches = match (operation, capability) {
            ("read", Capability::FileRead(_)) => true,
            ("write", Capability::FileWrite(_)) => true,
            ("execute", Capability::FileExecute(_)) => true,
            _ => false,
        };
        
        if !operation_matches {
            return false;
        }
        
        match access {
            FileAccess::Specific(allowed_path) => path == allowed_path,
            FileAccess::Directory(dir) => path.starts_with(dir),
            FileAccess::Pattern(pattern) => {
                // Simple glob-style matching
                self.matches_pattern(pattern, path)
            },
            FileAccess::Global => true,
        }
    }
    
    fn matches_network_access(
        &self,
        access: &NetworkAccess,
        host: &str,
        port: u16,
        operation: &str,
        capability: &Capability,
    ) -> bool {
        // Verify the operation matches the capability type
        let operation_matches = match (operation, capability) {
            ("connect", Capability::NetworkConnect(_)) => true,
            ("bind", Capability::NetworkBind(_)) => true,
            _ => false,
        };
        
        if !operation_matches {
            return false;
        }
        
        match access {
            NetworkAccess::Specific { host: allowed_host, port: allowed_port } => {
                host == allowed_host && port == *allowed_port
            },
            NetworkAccess::Host(allowed_host) => host == allowed_host,
            NetworkAccess::Port(allowed_port) => port == *allowed_port,
            NetworkAccess::Local => {
                host == "localhost" || host == "127.0.0.1" || host == "::1"
            },
            NetworkAccess::Global => true,
        }
    }
    
    fn matches_pattern(&self, pattern: &str, path: &str) -> bool {
        // Simple glob-style pattern matching
        // For production, consider using a proper glob library
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        }
        
        pattern == path
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capability::FileRead(access) => write!(f, "file:read:{}", access),
            Capability::FileWrite(access) => write!(f, "file:write:{}", access),
            Capability::FileExecute(access) => write!(f, "file:execute:{}", access),
            Capability::NetworkConnect(access) => write!(f, "network:connect:{}", access),
            Capability::NetworkBind(access) => write!(f, "network:bind:{}", access),
            Capability::ProcessSpawn => write!(f, "process:spawn"),
            Capability::ProcessKill => write!(f, "process:kill"),
            Capability::ProcessSignal => write!(f, "process:signal"),
            Capability::SystemInfo => write!(f, "system:info"),
            Capability::SystemMetrics => write!(f, "system:metrics"),
            Capability::CodeGeneration => write!(f, "code:generation"),
            Capability::CodeExecution(mode) => write!(f, "code:execution:{}", mode),
            Capability::ToolRegistration => write!(f, "tool:registration"),
            Capability::ToolDiscovery => write!(f, "tool:discovery"),
            Capability::AgentSpawn => write!(f, "agent:spawn"),
            Capability::AgentCommunication => write!(f, "agent:communication"),
            Capability::CapabilityManagement => write!(f, "admin:capabilities"),
            Capability::ResourceManagement => write!(f, "admin:resources"),
        }
    }
}

impl fmt::Display for FileAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileAccess::Specific(path) => write!(f, "specific:{}", path),
            FileAccess::Directory(dir) => write!(f, "directory:{}", dir),
            FileAccess::Pattern(pattern) => write!(f, "pattern:{}", pattern),
            FileAccess::Global => write!(f, "global"),
        }
    }
}

impl fmt::Display for NetworkAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkAccess::Specific { host, port } => write!(f, "specific:{}:{}", host, port),
            NetworkAccess::Host(host) => write!(f, "host:{}", host),
            NetworkAccess::Port(port) => write!(f, "port:{}", port),
            NetworkAccess::Local => write!(f, "local"),
            NetworkAccess::Global => write!(f, "global"),
        }
    }
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Wasm => write!(f, "wasm"),
            ExecutionMode::Native => write!(f, "native"),
            ExecutionMode::Script(lang) => write!(f, "script:{}", lang),
        }
    }
}

/// Predefined capability sets for common use cases
impl CapabilitySet {
    /// Basic file operations in workspace
    pub fn workspace_files() -> Self {
        Self::with_capabilities(vec![
            Capability::FileRead(FileAccess::Directory("./".to_string())),
            Capability::FileWrite(FileAccess::Directory("./".to_string())),
        ])
    }
    
    /// Network access for API calls
    pub fn api_access() -> Self {
        Self::with_capabilities(vec![
            Capability::NetworkConnect(NetworkAccess::Global),
        ])
    }
    
    /// Process management capabilities
    pub fn process_management() -> Self {
        Self::with_capabilities(vec![
            Capability::ProcessSpawn,
            Capability::ProcessSignal,
        ])
    }
    
    /// Dynamic code execution capabilities
    pub fn code_execution() -> Self {
        Self::with_capabilities(vec![
            Capability::CodeGeneration,
            Capability::CodeExecution(ExecutionMode::Wasm),
            Capability::CodeExecution(ExecutionMode::Script("python".to_string())),
        ])
    }
    
    /// Tool development capabilities
    pub fn tool_development() -> Self {
        Self::with_capabilities(vec![
            Capability::ToolRegistration,
            Capability::ToolDiscovery,
            Capability::CodeGeneration,
        ])
    }
    
    /// Administrative capabilities
    pub fn administrative() -> Self {
        Self::with_capabilities(vec![
            Capability::CapabilityManagement,
            Capability::ResourceManagement,
            Capability::SystemInfo,
            Capability::SystemMetrics,
        ])
    }
    
    /// Full privileges (use sparingly)
    pub fn full_privileges() -> Self {
        Self::with_capabilities(vec![
            Capability::FileRead(FileAccess::Global),
            Capability::FileWrite(FileAccess::Global),
            Capability::FileExecute(FileAccess::Global),
            Capability::NetworkConnect(NetworkAccess::Global),
            Capability::NetworkBind(NetworkAccess::Global),
            Capability::ProcessSpawn,
            Capability::ProcessKill,
            Capability::ProcessSignal,
            Capability::SystemInfo,
            Capability::SystemMetrics,
            Capability::CodeGeneration,
            Capability::CodeExecution(ExecutionMode::Native),
            Capability::ToolRegistration,
            Capability::ToolDiscovery,
            Capability::AgentSpawn,
            Capability::AgentCommunication,
            Capability::CapabilityManagement,
            Capability::ResourceManagement,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_set_operations() {
        let mut set1 = CapabilitySet::new();
        set1.add(Capability::FileRead(FileAccess::Global));
        set1.add(Capability::ProcessSpawn);
        
        let mut set2 = CapabilitySet::new();
        set2.add(Capability::FileRead(FileAccess::Global));
        set2.add(Capability::NetworkConnect(NetworkAccess::Global));
        
        assert!(set1.contains(&Capability::FileRead(FileAccess::Global)));
        assert!(!set1.contains(&Capability::NetworkConnect(NetworkAccess::Global)));
        
        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains(&Capability::FileRead(FileAccess::Global)));
    }
    
    #[test]
    fn test_file_access_permissions() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::FileRead(FileAccess::Directory("./src".to_string())));
        
        assert!(caps.allows_file_access("read", "./src/main.rs"));
        assert!(caps.allows_file_access("read", "./src/lib.rs"));
        assert!(!caps.allows_file_access("read", "./target/debug/app"));
        assert!(!caps.allows_file_access("write", "./src/main.rs"));
    }
    
    #[test]
    fn test_network_access_permissions() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::NetworkConnect(NetworkAccess::Host("api.example.com".to_string())));
        
        assert!(caps.allows_network_access("connect", "api.example.com", 443));
        assert!(caps.allows_network_access("connect", "api.example.com", 80));
        assert!(!caps.allows_network_access("connect", "evil.com", 443));
        assert!(!caps.allows_network_access("bind", "api.example.com", 443));
    }
    
    #[test]
    fn test_predefined_capability_sets() {
        let workspace = CapabilitySet::workspace_files();
        assert!(workspace.allows_file_access("read", "./README.md"));
        assert!(workspace.allows_file_access("write", "./src/main.rs"));
        
        let api = CapabilitySet::api_access();
        assert!(api.allows_network_access("connect", "api.example.com", 443));
    }
} 