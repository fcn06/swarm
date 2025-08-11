use serde::{Deserialize, Serialize};

use std::fs; // Assuming you might want logging here too

use agent_protocol_backbone::business_logic::agent::{AgentConfig, AgentReference};


// Config file is mostly the same as the one for stand alone agent
// only list of agents connected to full agent is additional
// todo:refactor 
#[derive(Deserialize, Debug, Clone)]
pub struct OrchestrationAgentConfig {
    pub agent_full_name: String,
    pub agent_full_host: String,
    pub agent_full_http_port: String,
    pub agent_full_ws_port: String,
    pub agent_full_discovery_url: Option<String>,
    pub agent_full_system_prompt: Option<String>,
    pub agent_full_version: String,
    pub agent_full_description: String,
    pub agent_full_skill_id: String,
    pub agent_full_skill_name: String,
    pub agent_full_skill_description: String,
    pub agent_full_model_id: String,
    pub agent_full_llm_url: String, // This is the LLM that will manage interactions with Full agent. LLM_FULL_API_KEY is connected to this one
    pub agent_full_mcp_config_path: Option<String>, // The path of the configuration for the MCP runtime
    pub agent_full_agents_references:Vec<AgentReference>, // List all agents connected to full agents
    pub agent_full_doc_url: Option<String>,
    pub agent_full_tags: Vec<String>,
    pub agent_full_examples: Vec<String>,
}

impl OrchestrationAgentConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<OrchestrationAgentConfig> {
        let config_content = fs::read_to_string(path)?;
        let config: OrchestrationAgentConfig = toml::from_str(&config_content)?;
        Ok(config)
    }
}


impl AgentConfig for OrchestrationAgentConfig {
    fn agent_name(&self) -> String { self.agent_full_name.clone() }
    fn agent_host(&self) -> String { self.agent_full_host.clone() }
    fn agent_http_port(&self) -> u16 { self.agent_full_http_port.parse().unwrap_or_default() }
    fn agent_ws_port(&self) -> u16 { self.agent_full_ws_port.parse().unwrap_or_default() }
    fn agent_discovery_url(&self) -> Option<String> { self.agent_full_discovery_url.clone() }
    fn agent_system_prompt(&self) -> Option<String> { self.agent_full_system_prompt.clone() }
    fn agent_version(&self) -> String { self.agent_full_version.clone() }
    fn agent_description(&self) -> String { self.agent_full_description.clone() }
    fn agent_skill_id(&self) -> String { self.agent_full_skill_id.clone() }
    fn agent_skill_name(&self) -> String { self.agent_full_skill_name.clone() }
    fn agent_skill_description(&self) -> String { self.agent_full_skill_description.clone() }
    fn agent_model_id(&self) -> String { self.agent_full_model_id.clone() }
    fn agent_llm_url(&self) -> String { self.agent_full_llm_url.clone() }
    fn agent_mcp_config_path(&self) -> Option<String> { self.agent_full_mcp_config_path.clone() }
    fn agent_doc_url(&self) -> Option<String> { self.agent_full_doc_url.clone() }
    fn agent_tags(&self) -> Vec<String> { self.agent_full_tags.clone() }
    fn agent_examples(&self) -> Vec<String> { self.agent_full_examples.clone() }
    fn agents_references(&self) -> Option<Vec<AgentReference>> { self.agent_full_references.clone()  }
}