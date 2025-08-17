
use serde::{Serialize,Deserialize};
use std::fs; // Assuming you might want logging here too

use tracing::{Level};
use tracing_subscriber::{prelude::*, fmt, layer::Layer, Registry, filter};

//////////////////////////////////////////////////////////////////////
// CONFIG FOR MCP
//////////////////////////////////////////////////////////////////////


// The configuration for the MCP runtime
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
    pub agent_mcp_server_api_key:Option<String>, // this is the API-key to connect to your mcp server
    pub agent_mcp_model_id: String,
    pub agent_mcp_llm_url: String, // This is the LLM that will manage interactions with MCP server. LLM_MCP_API_KEY is connected to this one
    pub agent_mcp_system_prompt: String,
    pub agent_mcp_endpoint_url: Option<String>, // This will come from command line or instance config
}

impl AgentMcpConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentMcpConfig> {
        //info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentMcpConfig = toml::from_str(&config_content)?;
        //debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}

//////////////////////////////////////////////////////////////////////
// CONFIG FOR ALL AGENTS
//////////////////////////////////////////////////////////////////////


#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    pub agent_name: String,
    pub agent_host: String,
    pub agent_http_port: String,
    pub agent_ws_port: String,
    pub agent_discovery_url: Option<String>,
    pub agent_system_prompt: Option<String>,
    pub agent_version: String,
    pub agent_description: String,
    pub agent_skill_id: String,
    pub agent_skill_name: String,
    pub agent_skill_description: String,
    pub agent_model_id: String,
    pub agent_llm_url: String, // This is the LLM that will manage interactions with A2A agent. LLM_A2A_API_KEY is connected to this one
    pub agent_mcp_config_path: Option<String>, // The path of the configuration for the MCP runtime
    pub agent_doc_url: Option<String>,
    pub agent_tags: Vec<String>,
    pub agent_examples: Vec<String>,
    pub agent_agents_references: Option<Vec<AgentReference>>,
    pub agent_evaluation_service_url: Option<String>,
    pub agent_memory_service_url: Option<String>,
}

impl AgentConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentConfig> {
        //info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentConfig = toml::from_str(&config_content)?;
        //debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }

    pub fn agent_name(&self) -> String { self.agent_name.clone() }
    pub fn agent_host(&self) -> String { self.agent_host.clone() }
    pub fn agent_http_port(&self) -> u16 { self.agent_http_port.parse().unwrap_or_default() }
    pub fn agent_ws_port(&self) -> u16 { self.agent_ws_port.parse().unwrap_or_default() }
    pub fn agent_discovery_url(&self) -> Option<String> { self.agent_discovery_url.clone() }
    pub fn agent_system_prompt(&self) -> Option<String> { self.agent_system_prompt.clone() }
    pub fn agent_version(&self) -> String { self.agent_version.clone() }
    pub fn agent_description(&self) -> String { self.agent_description.clone() }
    pub fn agent_skill_id(&self) -> String { self.agent_skill_id.clone() }
    pub fn agent_skill_name(&self) -> String { self.agent_skill_name.clone() }
    pub fn agent_skill_description(&self) -> String { self.agent_skill_description.clone() }
    pub fn agent_model_id(&self) -> String { self.agent_model_id.clone() }
    pub fn agent_llm_url(&self) -> String { self.agent_llm_url.clone() }
    pub fn agent_mcp_config_path(&self) -> Option<String> { self.agent_mcp_config_path.clone() }
    pub fn agent_doc_url(&self) -> Option<String> { self.agent_doc_url.clone() }
    pub fn agent_tags(&self) -> Vec<String> { self.agent_tags.clone() }
    pub fn agent_examples(&self) -> Vec<String> { self.agent_examples.clone() }
    pub fn agent_agents_references(&self) -> Option<Vec<AgentReference>> { self.agent_agents_references.clone() }
    pub fn agent_evaluation_service_url(&self) -> Option<String> { self.agent_evaluation_service_url.clone() }
    pub fn agent_memory_service_url(&self) -> Option<String> { self.agent_memory_service_url.clone() }

}

///////////////////////////////////////////////////////////////
// SIMPLE AGENT REFERENCE IMPLEMENTATION
///////////////////////////////////////////////////////////////

// Agent info provider implementation
//#[derive(Clone)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReference {
    pub name: String,
    pub url: String,
    pub is_default:Option<bool>,
}

impl AgentReference {
    pub fn new(name: String, url: String) -> anyhow::Result<AgentReference> {
        // Create the agent card
        Ok(AgentReference {
            name: name,
            url: url,
            is_default:None,
        })
    }
}

impl AgentReference {
    pub async fn get_agent_reference(&self) -> anyhow::Result<AgentReference> {
        Ok(self.clone())
    }
}

///////////////////////////////////////////////////////////////
// SETUP LOGGING LEVEL
///////////////////////////////////////////////////////////////

pub fn setup_logging(log_level: &str) {
    let level = match log_level {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = Registry::default().with(
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(level)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
}