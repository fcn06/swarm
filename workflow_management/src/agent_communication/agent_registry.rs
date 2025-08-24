use super::agent_runner::AgentRunner;
use std::collections::HashMap;
use std::sync::Arc;

use a2a_rs::AgentSkill;

pub struct AgentRegistry {
    runners: HashMap<String, Arc<dyn AgentRunner>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, runner: Arc<dyn AgentRunner>) {
        self.runners.insert(runner.name(), runner);
    }

    pub fn register_with_name(&mut self, name: String, runner: Arc<dyn AgentRunner>) {
        self.runners.insert(name, runner);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn AgentRunner>> {
        self.runners.get(name).cloned()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// V2 implementation, more flexible

pub struct AgentDefinition {
    pub id: String,         // Unique identifier for the agent
    pub name: String,       // Human-readable name
    pub description: String, // Description of the agent's overall purpose
    pub skills: Vec<AgentSkill>, // (New) List of skills this agent possesses
}

pub struct AgentRegistryV2 {
    definitions: HashMap<String, AgentDefinition>,
}

impl AgentRegistryV2 {
    pub fn new() -> Self { Self{definitions : HashMap::new() } }
    pub fn register_agent(&mut self, definition: AgentDefinition) { self.definitions.insert(definition.id.clone(), definition); }
    pub fn get_agent_definition(&self, agent_id: &str) -> Option<&AgentDefinition> { self.definitions.get(agent_id)}
}
