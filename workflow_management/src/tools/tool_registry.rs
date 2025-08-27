use std::collections::HashMap;

/* 
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

*/

// V2 implementation, more flexible

pub struct ToolDefinition {
    pub id: String,         // Unique identifier for the tool (e.g., "web_scraper", "math_solver")
    pub name: String,       // Human-readable name
    pub description: String, // (New) Detailed description of the tool's functionality
    pub input_schema: serde_json::Value, // JSON schema for expected input
    pub output_schema: serde_json::Value, // JSON schema for expected output
}


pub struct ToolRegistry {
    definitions: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self { Self{definitions : HashMap::new() } }
    pub fn register_tool(&mut self, definition: ToolDefinition) { self.definitions.insert(definition.id.clone(), definition); }
    pub fn get_tool_definition(&self, tool_id: &str) -> Option<&ToolDefinition> {self.definitions.get(tool_id)}
    pub fn get_tool_details(&self) -> Option<String> {
        if self.definitions.is_empty() {return None;}
        let mut details = String::new();
        for (id, tool_def) in self.definitions.iter() {details.push_str(&format!("* {} -- {}\n", id, tool_def.description));}
        Some(details.trim_end().to_string())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}