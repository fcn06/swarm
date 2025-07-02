use configuration::SimpleAgentReference;

use serde::{Serialize,Deserialize};

pub mod a2a_agent_logic;
pub mod a2a_plan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerAgentDefinition {
    pub model_id: String,
    pub llm_url:String,
    pub agent_configs: Vec<SimpleAgentReference>, // Info to connect to agents
}