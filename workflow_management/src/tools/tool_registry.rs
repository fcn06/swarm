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

// V2 implementation, more flexible

pub struct ToolDefinition {
    pub id: String,         // Unique identifier for the tool (e.g., "web_scraper", "math_solver")
    pub name: String,       // Human-readable name
    pub description: String, // (New) Detailed description of the tool's functionality
    pub input_schema: serde_json::Value, // JSON schema for expected input
    pub output_schema: serde_json::Value, // JSON schema for expected output
}


pub struct ToolRegistryV2 {
    definitions: HashMap<String, ToolDefinition>,
}

impl ToolRegistryV2 {
    pub fn new() -> Self { Self{definitions : HashMap::new() } }
    pub fn register_tool(&mut self, definition: ToolDefinition) { self.definitions.insert(definition.id.clone(), definition); }
    pub fn get_tool_definition(&self, tool_id: &str) -> Option<&ToolDefinition> {self.definitions.get(tool_id)}
}