use serde::{Deserialize};

use std::fs; // Assuming you might want logging here too

use agent_protocol_backbone::business_logic::agent::{AgentConfig, AgentReference};

//////////////////////////////////////////////////////////////////////
// NEW VERSION OF AGENT CONFIG
//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug, Clone)]
pub struct BasicAgentConfig {
    pub agent_a2a_name: String,
    pub agent_a2a_host: String,
    pub agent_a2a_http_port: String,
    pub agent_a2a_ws_port: String,
    pub agent_a2a_discovery_url: Option<String>,
    pub agent_a2a_system_prompt: Option<String>,
    pub agent_a2a_version: String,
    pub agent_a2a_description: String,
    pub agent_a2a_skill_id: String,
    pub agent_a2a_skill_name: String,
    pub agent_a2a_skill_description: String,
    pub agent_a2a_model_id: String,
    pub agent_a2a_llm_url: String, // This is the LLM that will manage interactions with A2A agent. LLM_A2A_API_KEY is connected to this one
    pub agent_a2a_mcp_config_path: Option<String>, // The path of the configuration for the MCP runtime
    pub agent_a2a_doc_url: Option<String>,
    pub agent_a2a_tags: Vec<String>,
    pub agent_a2a_examples: Vec<String>,
}

impl BasicAgentConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<BasicAgentConfig> {
        //info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: BasicAgentConfig = toml::from_str(&config_content)?;
        //debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}

impl AgentConfig for BasicAgentConfig {
    fn agent_name(&self) -> String { self.agent_a2a_name.clone() }
    fn agent_host(&self) -> String { self.agent_a2a_host.clone() }
    fn agent_http_port(&self) -> u16 { self.agent_a2a_http_port.parse().unwrap_or_default() }
    fn agent_ws_port(&self) -> u16 { self.agent_a2a_ws_port.parse().unwrap_or_default() }
    fn agent_discovery_url(&self) -> Option<String> { self.agent_a2a_discovery_url.clone() }
    fn agent_system_prompt(&self) -> Option<String> { self.agent_a2a_system_prompt.clone() }
    fn agent_version(&self) -> String { self.agent_a2a_version.clone() }
    fn agent_description(&self) -> String { self.agent_a2a_description.clone() }
    fn agent_skill_id(&self) -> String { self.agent_a2a_skill_id.clone() }
    fn agent_skill_name(&self) -> String { self.agent_a2a_skill_name.clone() }
    fn agent_skill_description(&self) -> String { self.agent_a2a_skill_description.clone() }
    fn agent_model_id(&self) -> String { self.agent_a2a_model_id.clone() }
    fn agent_llm_url(&self) -> String { self.agent_a2a_llm_url.clone() }
    fn agent_mcp_config_path(&self) -> Option<String> { self.agent_a2a_mcp_config_path.clone() }
    fn agent_doc_url(&self) -> Option<String> { self.agent_a2a_doc_url.clone() }
    fn agent_tags(&self) -> Vec<String> { self.agent_a2a_tags.clone() }
    fn agent_examples(&self) -> Vec<String> { self.agent_a2a_examples.clone() }
    fn agents_references(&self) -> Option<Vec<AgentReference>> { None }
}