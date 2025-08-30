//use crate::graph::graph_definition::Activity;

use std::sync::Arc;
use super::agent_registry::AgentRegistry;
use super::agent_invoker::AgentInvoker;



// V2 implementation, more flexible

pub struct AgentRunner {
    pub agent_registry: Arc<AgentRegistry>, // To get metadata
    agent_invoker: Arc<dyn AgentInvoker>, // To perform actual interaction
}

impl AgentRunner {
    // Constructor using dependency injection
    pub fn new(agent_registry: Arc<AgentRegistry>, agent_invoker: Arc<dyn AgentInvoker>) -> Self {
        AgentRunner { agent_registry, agent_invoker }
    }

    /// Interacts with an agent identified by its ID.
    pub async fn interact(&self, agent_id: String, message: String,skill:String) -> anyhow::Result<serde_json::Value> {
        // Optional: Fetch metadata for logging or task routing
        if let Some(agent_def) = self.agent_registry.get_agent_definition(&agent_id) {
            // You now have easy access to agent_def.skills here
            println!("Interacting with agent: {} (Skills: {:?})", agent_def.name, agent_def.skills);
        } else {
            anyhow::bail!(format!("Agent '{}' not found in registry.", agent_id));
        }

        // Delegate the actual, protocol-specific interaction to the injected invoker
        self.agent_invoker.interact(agent_id,message ,skill).await
    }
}