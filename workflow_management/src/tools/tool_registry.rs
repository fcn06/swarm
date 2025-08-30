use std::collections::HashMap;


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
    pub fn get_tool_details(&self) -> String {
       
        if self.definitions.is_empty() {return "No Tool Registered".to_string();}
        let mut details = "Tools registered in the system: \n".to_string();
        for (id, tool_def) in self.definitions.iter() {details.push_str(&format!("* tool_id : {} -- description : {} -- arguments : {}\n", id, tool_def.description,tool_def.input_schema));}
        details.trim_end().to_string()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}