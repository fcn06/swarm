//use super::agent_runner::AgentRunner;
use std::collections::HashMap;

use a2a_rs::AgentSkill;

/* 
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
*/

// V2 implementation, more flexible

pub struct AgentDefinition {
    pub id: String,         // Unique identifier for the agent
    pub name: String,       // Human-readable name
    pub description: String, // Description of the agent's overall purpose
    pub skills: Vec<AgentSkill>, // (New) List of skills this agent possesses
}

pub struct AgentRegistry {
    definitions: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self { Self{definitions : HashMap::new() } }
    pub fn register_agent(&mut self, definition: AgentDefinition) { self.definitions.insert(definition.id.clone(), definition); }
    pub fn get_agent_definition(&self, agent_id: &str) -> Option<&AgentDefinition> { self.definitions.get(agent_id)}
    pub fn get_agent_details(&self) -> String {

        let mut description = "Agents registered in the system: \n".to_string();
        if self.definitions.is_empty() {
            description.push_str("- No agents connected.");
        } else {

            for (id, agent_def) in self.definitions.iter() {
                description.push_str(&format!("* agent_id : {} -- description :{} -- ", id, agent_def.description));
            
                if agent_def.skills.is_empty() {
                    description.push_str(" No specific skills listed.");
                } else {
                    for skill in agent_def.skills.clone() {
                        description.push_str(&format!(" skill.id : '{}' -- skill.description : '{}' ", skill.id,skill.description.clone()));
                    }
                }
                description.push_str(&format!("\n"));
            }
        }

        description.trim_end().to_string()

        /* 
        if self.definitions.is_empty() {return None;}
        let mut details = String::new();
        // we could also retrieve skills
        for (id, agent_def) in self.definitions.iter() {details.push_str(&format!("* {} -- {}\n", id, agent_def.description));}
        Some(details.trim_end().to_string())
        */

    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}