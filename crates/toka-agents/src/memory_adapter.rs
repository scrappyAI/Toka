use anyhow::Result;
use std::collections::HashMap;
use std::sync::Mutex;

/// Abstraction over key-value storage for serialised agent state.
///
/// This replaces the old `toka_secrets::MemoryAdapter` without pulling in
/// heavyweight persistence crates.  Implementations are **expected to be
/// best-effort**: callers should not assume writes are durable unless the
/// adapter guarantees it in its own documentation.
pub trait MemoryAdapter: Send + Sync {
    /// Persist a UTF-8 JSON string under the given key.
    fn save_json(&self, key: &str, json: &str) -> Result<()>;

    /// Load previously saved JSON string (if any).
    fn load_json(&self, key: &str) -> Result<Option<String>>;
}

/// Simple in-memory `HashMap` implementation used mainly in tests.
#[derive(Default, Debug)]
pub struct InMemoryAdapter {
    map: Mutex<HashMap<String, String>>, // coarse-grained lock is fine here
}

impl InMemoryAdapter {
    /// Construct a new empty adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MemoryAdapter for InMemoryAdapter {
    fn save_json(&self, key: &str, json: &str) -> Result<()> {
        let mut guard = self.map.lock().expect("poisoned mutex");
        guard.insert(key.to_owned(), json.to_owned());
        Ok(())
    }

    fn load_json(&self, key: &str) -> Result<Option<String>> {
        let guard = self.map.lock().expect("poisoned mutex");
        Ok(guard.get(key).cloned())
    }
} 