use std::collections::HashMap;

/* 
pub struct TaskRegistry {
    runners: HashMap<String, Arc<dyn TaskRunner>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, runner: Arc<dyn TaskRunner>) {
        self.runners.insert(runner.name(), runner);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn TaskRunner>> {
        self.runners.get(name).cloned()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
*/

// V2 implementation, more flexible

pub struct TaskDefinition {
    pub id: String,         // Unique identifier for the tool (e.g., "web_scraper", "math_solver")
    pub name: String,       // Human-readable name
    pub description: String, // (New) Detailed description of the tool's functionality
    pub input_schema: serde_json::Value, // JSON schema for expected input
    pub output_schema: serde_json::Value, // JSON schema for expected output
}


pub struct TaskRegistry {
    definitions: HashMap<String, TaskDefinition>,
}

impl TaskRegistry {
    pub fn new() -> Self { Self{definitions : HashMap::new() } }
    pub fn register_task(&mut self, definition: TaskDefinition) { self.definitions.insert(definition.id.clone(), definition); }
    pub fn get_task_definition(&self, tool_id: &str) -> Option<&TaskDefinition> {self.definitions.get(tool_id)}
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}