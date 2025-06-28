use a2a_agent_backbone::a2a_agent_initialization::a2a_agent_config::SimpleAgentReference;
use serde::{Serialize,Deserialize};

pub mod a2a_agent_logic;
pub mod a2a_plan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerAgentConfig {
    pub model_id: String,
    pub agent_configs: Vec<SimpleAgentReference>, // Info to connect to agents
}