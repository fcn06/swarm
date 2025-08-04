pub mod memory_server;
pub mod memory_service_client;

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentData {
    pub agent_id: String,
    pub content: String,
    pub summary: Option<String>,
    pub interaction_type: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub agent_id: String,
    pub timestamp: String,
    pub content: String,
    pub summary: Option<String>,
    pub interaction_type: String,
}
