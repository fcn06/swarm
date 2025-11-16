
use serde::{Serialize,Deserialize};
use std::fs; // Assuming you might want logging here too

use tracing_subscriber::{prelude::*, fmt, layer::Layer, Registry};
use tracing_subscriber::EnvFilter;

//////////////////////////////////////////////////////////////////////
// CONFIG FOR MCP
//////////////////////////////////////////////////////////////////////


// The configuration for the MCP runtime
#[derive(Deserialize, Debug, Clone)]
pub struct McpRuntimeConfig {
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
    pub agent_mcp_llm_api_key_env_var: Option<String>, // New field: name of environment variable holding the LLM API key
    pub agent_mcp_system_prompt: String,
    pub agent_mcp_endpoint_url: Option<String>, // This will come from command line or instance config
}

impl McpRuntimeConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<McpRuntimeConfig> {
        //info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: McpRuntimeConfig = toml::from_str(&config_content)?;
        //debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}

//////////////////////////////////////////////////////////////////////
// CONFIG FOR ALL AGENTS
//////////////////////////////////////////////////////////////////////


#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_name: String,
    pub agent_http_endpoint: String,
    pub agent_ws_endpoint: String,
    //pub agent_host: String,
    //pub agent_http_port: String,
    //pub agent_ws_port: String,
    //pub agent_discovery_url: Option<String>, // to remove from config and make it runtime
    pub agent_discoverable: Option<bool>,
    pub agent_executor_url: Option<String>, // todo: to remove from config and make it runtime
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
    //pub agent_evaluation_service_url: Option<String>, // to remove from config and make it runtime
    //pub agent_memory_service_url: Option<String>, // to remove from config and make it runtime
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

    pub fn builder() -> AgentConfigBuilder {
        AgentConfigBuilder::new()
    }

    pub fn agent_id(&self) -> String { self.agent_id.clone() }
    pub fn agent_name(&self) -> String { self.agent_name.clone() }
    pub fn agent_http_endpoint(&self) -> String { self.agent_http_endpoint.clone() }
    pub fn agent_ws_endpoint(&self) -> String { self.agent_ws_endpoint.clone() }
    //pub fn agent_host(&self) -> String { self.agent_host.clone() }
    //pub fn agent_http_port(&self) -> u16 { self.agent_http_port.parse().unwrap_or_default() }
    //pub fn agent_ws_port(&self) -> u16 { self.agent_ws_port.parse().unwrap_or_default() }
    //pub fn agent_discovery_url(&self) -> Option<String> { self.agent_discovery_url.clone() }
    pub fn agent_discoverable(&self) -> Option<bool> { self.agent_discoverable.or(Some(false)) } // false by defaul except when explicitly set to true
    pub fn agent_executor_url(&self) -> Option<String> { self.agent_executor_url.clone() }
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
    //pub fn agent_evaluation_service_url(&self) -> Option<String> { self.agent_evaluation_service_url.clone() }
    //pub fn agent_memory_service_url(&self) -> Option<String> { self.agent_memory_service_url.clone() }

}

pub struct AgentConfigBuilder {
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,
    pub agent_http_endpoint: Option<String>,
    pub agent_ws_endpoint: Option<String>,
    //pub agent_host: Option<String>,
    //pub agent_http_port: Option<String>,
    //pub agent_ws_port: Option<String>,
    //pub agent_discovery_url: Option<String>,
    pub agent_discoverable: Option<bool>,
    pub agent_executor_url: Option<String>,
    pub agent_system_prompt: Option<String>,
    pub agent_version: Option<String>,
    pub agent_description: Option<String>,
    pub agent_skill_id: Option<String>,
    pub agent_skill_name: Option<String>,
    pub agent_skill_description: Option<String>,
    pub agent_model_id: Option<String>,
    pub agent_llm_url: Option<String>,
    pub agent_mcp_config_path: Option<String>,
    pub agent_doc_url: Option<String>,
    pub agent_tags: Option<Vec<String>>,
    pub agent_examples: Option<Vec<String>>,
    pub agent_agents_references: Option<Vec<AgentReference>>,
    //pub agent_evaluation_service_url: Option<String>,
    //pub agent_memory_service_url: Option<String>,
}

impl AgentConfigBuilder {
    pub fn new() -> Self {
        AgentConfigBuilder {
            agent_id: None, 
            agent_name: None,
            agent_http_endpoint: None,
            agent_ws_endpoint: None,
            //agent_host: None,
            //agent_http_port: None,
            //agent_ws_port: None,
            //agent_discovery_url: None,
            agent_discoverable: None,
            agent_executor_url: None,
            agent_system_prompt: None,
            agent_version: None,
            agent_description: None,
            agent_skill_id: None,
            agent_skill_name: None,
            agent_skill_description: None,
            agent_model_id: None,
            agent_llm_url: None,
            agent_mcp_config_path: None,
            agent_doc_url: None,
            agent_tags: None,
            agent_examples: None,
            agent_agents_references: None,
            //agent_evaluation_service_url: None,
            //agent_memory_service_url: None,
        }
    }

    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn agent_name(mut self, agent_name: String) -> Self {
        self.agent_name = Some(agent_name);
        self
    }

    pub fn agent_http_endpoint(mut self, agent_http_endpoint: String) -> Self {
        self.agent_http_endpoint = Some(agent_http_endpoint);
        self
    }

    pub fn agent_ws_endpoint(mut self, agent_ws_endpoint: String) -> Self {
        self.agent_ws_endpoint = Some(agent_ws_endpoint);
        self
    }

    /* 

    pub fn agent_host(mut self, agent_host: String) -> Self {
        self.agent_host = Some(agent_host);
        self
    }

    pub fn agent_http_port(mut self, agent_http_port: String) -> Self {
        self.agent_http_port = Some(agent_http_port);
        self
    }

    pub fn agent_ws_port(mut self, agent_ws_port: String) -> Self {
        self.agent_ws_port = Some(agent_ws_port);
        self
    }

    pub fn agent_discovery_url(mut self, agent_discovery_url: String) -> Self {
        self.agent_discovery_url = Some(agent_discovery_url);
        self
    }
    */

    pub fn agent_discoverable(mut self, agent_discoverable: bool) -> Self {
        self.agent_discoverable = Some(agent_discoverable);
        self
    }

    pub fn agent_executor_url(mut self, agent_executor_url: String) -> Self {
        self.agent_executor_url = Some(agent_executor_url);
        self
    }

    pub fn agent_system_prompt(mut self, agent_system_prompt: String) -> Self {
        self.agent_system_prompt = Some(agent_system_prompt);
        self
    }

    pub fn agent_version(mut self, agent_version: String) -> Self {
        self.agent_version = Some(agent_version);
        self
    }

    pub fn agent_description(mut self, agent_description: String) -> Self {
        self.agent_description = Some(agent_description);
        self
    }

    pub fn agent_skill_id(mut self, agent_skill_id: String) -> Self {
        self.agent_skill_id = Some(agent_skill_id);
        self
    }

    pub fn agent_skill_name(mut self, agent_skill_name: String) -> Self {
        self.agent_skill_name = Some(agent_skill_name);
        self
    }

    pub fn agent_skill_description(mut self, agent_skill_description: String) -> Self {
        self.agent_skill_description = Some(agent_skill_description);
        self
    }

    pub fn agent_model_id(mut self, agent_model_id: String) -> Self {
        self.agent_model_id = Some(agent_model_id);
        self
    }

    pub fn agent_llm_url(mut self, agent_llm_url: String) -> Self {
        self.agent_llm_url = Some(agent_llm_url);
        self
    }

    pub fn agent_mcp_config_path(mut self, agent_mcp_config_path: String) -> Self {
        self.agent_mcp_config_path = Some(agent_mcp_config_path);
        self
    }

    pub fn agent_doc_url(mut self, agent_doc_url: String) -> Self {
        self.agent_doc_url = Some(agent_doc_url);
        self
    }

    pub fn agent_tags(mut self, agent_tags: Vec<String>) -> Self {
        self.agent_tags = Some(agent_tags);
        self
    }

    pub fn agent_examples(mut self, agent_examples: Vec<String>) -> Self {
        self.agent_examples = Some(agent_examples);
        self
    }

    pub fn agent_agents_references(mut self, agent_agents_references: Vec<AgentReference>) -> Self {
        self.agent_agents_references = Some(agent_agents_references);
        self
    }

    /* 
    pub fn agent_evaluation_service_url(mut self, agent_evaluation_service_url: String) -> Self {
        self.agent_evaluation_service_url = Some(agent_evaluation_service_url);
        self
    }

    pub fn agent_memory_service_url(mut self, agent_memory_service_url: String) -> Self {
        self.agent_memory_service_url = Some(agent_memory_service_url);
        self
    }
    */

    pub fn build(self) -> anyhow::Result<AgentConfig> {
        Ok(AgentConfig {
            agent_id: self.agent_id.ok_or_else(|| anyhow::anyhow!("agent_id is required"))?,
            agent_name: self.agent_name.ok_or_else(|| anyhow::anyhow!("agent_name is required"))?,
            agent_http_endpoint: self.agent_http_endpoint.ok_or_else(|| anyhow::anyhow!("agent_http_endpoint is required"))?,
            agent_ws_endpoint: self.agent_ws_endpoint.ok_or_else(|| anyhow::anyhow!("agent_ws_endpoint is required"))?,
            //agent_host: self.agent_host.ok_or_else(|| anyhow::anyhow!("agent_host is required"))?,
            //agent_http_port: self.agent_http_port.ok_or_else(|| anyhow::anyhow!("agent_http_port is required"))?,
            //agent_ws_port: self.agent_ws_port.ok_or_else(|| anyhow::anyhow!("agent_ws_port is required"))?,
            //agent_discovery_url: self.agent_discovery_url,
            agent_discoverable: self.agent_discoverable,
            agent_executor_url: self.agent_executor_url,
            agent_system_prompt: self.agent_system_prompt,
            agent_version: self.agent_version.ok_or_else(|| anyhow::anyhow!("agent_version is required"))?,
            agent_description: self.agent_description.ok_or_else(|| anyhow::anyhow!("agent_description is required"))?,
            agent_skill_id: self.agent_skill_id.ok_or_else(|| anyhow::anyhow!("agent_skill_id is required"))?,
            agent_skill_name: self.agent_skill_name.ok_or_else(|| anyhow::anyhow!("agent_skill_name is required"))?,
            agent_skill_description: self.agent_skill_description.ok_or_else(|| anyhow::anyhow!("agent_skill_description is required"))?,
            agent_model_id: self.agent_model_id.ok_or_else(|| anyhow::anyhow!("agent_model_id is required"))?,
            agent_llm_url: self.agent_llm_url.ok_or_else(|| anyhow::anyhow!("agent_llm_url is required"))?,
            agent_mcp_config_path: self.agent_mcp_config_path,
            agent_doc_url: self.agent_doc_url,
            agent_tags: self.agent_tags.unwrap_or_default(),
            agent_examples: self.agent_examples.unwrap_or_default(),
            agent_agents_references: self.agent_agents_references,
            //agent_evaluation_service_url: self.agent_evaluation_service_url,
            //agent_memory_service_url: self.agent_memory_service_url,
        })
    }
}


///////////////////////////////////////////////////////////////
// SIMPLE AGENT REFERENCE IMPLEMENTATION
///////////////////////////////////////////////////////////////

// Agent info provider implementation
//#[derive(Clone)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReference {
    pub id: String,
    pub url: String,
    pub is_default:Option<bool>,
}

impl AgentReference {
    pub fn new(id: String, url: String) -> anyhow::Result<AgentReference> {
        // Create the agent card
        Ok(AgentReference {
            id: id,
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

    let default_filter = log_level.to_string(); // Use the provided log_level as the default

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    let subscriber = Registry::default().with(
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(env_filter),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
