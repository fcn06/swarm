use serde::{Deserialize, Serialize};

use log::{debug, info};
use std::fs; // Assuming you might want logging here too

//////////////////////////////////////////////////////////////////////
// NEW VERSION OF AGENT CONFIG
//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug, Clone)]
pub struct AgentA2aConfig {
    pub agent_a2a_name: String,
    pub agent_a2a_host: String,
    pub agent_a2a_http_port: String,
    pub agent_a2a_ws_port: String,
    pub agent_a2a_system_prompt: Option<String>,
    pub agent_a2a_version: String,
    pub agent_a2a_description: String,
    pub agent_a2a_skill_id: String,
    pub agent_a2a_skill_name: String,
    pub agent_a2a_skill_description: String,
    pub agent_a2a_model_id: String,
    pub agent_a2a_llm_url: String,
    pub agent_a2a_mcp_config_path: Option<String>,
    pub agent_a2a_doc_url: Option<String>,
    pub agent_a2a_tags: Vec<String>,
    pub agent_a2a_examples: Vec<String>,
}

impl AgentA2aConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentA2aConfig> {
        info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentA2aConfig = toml::from_str(&config_content)?;
        debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentMcpConfig {
    //pub agent_mcp_system_prompt: String,
    pub agent_mcp_role_tool: String,
    pub agent_mcp_role_assistant: String,
    pub agent_mcp_tool_choice_auto: String,
    pub agent_mcp_finish_reason_tool_calls: String,
    pub agent_mcp_finish_reason_stop: String,
    pub agent_mcp_max_loops: u32, // Use appropriate type
    pub agent_mcp_server_url: Option<String>,
    pub agent_mcp_server_api_key:Option<String>,
    pub agent_mcp_model_id: String,
    pub agent_mcp_llm_url: String,
    pub agent_mcp_system_message: String,
    pub agent_mcp_endpoint_url: Option<String>, // This will come from command line or instance config
}

impl AgentMcpConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentMcpConfig> {
        info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentMcpConfig = toml::from_str(&config_content)?;
        debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}


///////////////////////////////////////////////////////////////
// SIMPLE AGENT REFERENCE IMPLEMENTATION
///////////////////////////////////////////////////////////////

// Agent info provider implementation
//#[derive(Clone)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAgentReference {
    pub name: String,
    pub url: String,
}

impl SimpleAgentReference {
    pub fn new(name: String, url: String) -> anyhow::Result<SimpleAgentReference> {
        // Create the agent card
        Ok(SimpleAgentReference {
            name: name,
            url: url,
        })
    }
}

impl SimpleAgentReference {
    pub async fn get_agent_reference(&self) -> anyhow::Result<SimpleAgentReference> {
        Ok(self.clone())
    }
}


#[derive(Deserialize, Debug, Clone)]
pub struct AgentPlannerConfig {
    pub agent_planner_name: String,
    pub agent_planner_system_prompt: String,
    pub agent_planner_model_id: String,
    pub agent_planner_llm_url: String,
    pub agent_planner_discovery_url: Option<String>, // future use
    pub agent_planner_agents_references:Vec<SimpleAgentReference>,
}

impl AgentPlannerConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentPlannerConfig> {
        info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentPlannerConfig = toml::from_str(&config_content)?;
        debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}
