use super::tool_runner::ToolRunner;
use std::collections::HashMap;
use std::sync::Arc;

/// A registry for `ToolRunner` instances.
pub struct ToolRegistry {
    runners: HashMap<String, Arc<dyn ToolRunner>>,
}

impl ToolRegistry {
    /// Creates a new, empty `ToolRegistry`.
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    /// Registers a tool runner.
    pub fn register(&mut self, runner: Arc<dyn ToolRunner>) {
        self.runners.insert(runner.name(), runner);
    }

    /// Retrieves a tool runner by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolRunner>> {
        self.runners.get(name).cloned()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
