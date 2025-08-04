use serde::{Deserialize, Serialize};

use std::fs; // Assuming you might want logging here too

use a2a_rs::domain::AgentCard;

//////////////////////////////////////////////////////////////////////
// NEW VERSION OF AGENT CONFIG
//////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug, Clone)]
pub struct AgentA2aConfig {
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

impl AgentA2aConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentA2aConfig> {
        //info!("Loading agent configuration from: {}", path);
        let config_content = fs::read_to_string(path)?;
        let config: AgentA2aConfig = toml::from_str(&config_content)?;
        //debug!("Loaded agent configuration: {:?}", config);
        Ok(config)
    }
}


// Config file is mostly the same as the one for stand alone agent
// only list of agents connected to full agent is additional
// todo:refactor 
#[derive(Deserialize, Debug, Clone)]
pub struct AgentFullConfig {
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
    pub agent_full_agents_references:Vec<SimpleAgentReference>, // List all agents connected to full agents
    pub agent_full_doc_url: Option<String>,
    pub agent_full_tags: Vec<String>,
    pub agent_full_examples: Vec<String>,
}

impl AgentFullConfig {
    /// Loads agent configuration from a TOML file.
    pub fn load_agent_config(path: &str) -> anyhow::Result<AgentFullConfig> {
        let config_content = fs::read_to_string(path)?;
        let config: AgentFullConfig = toml::from_str(&config_content)?;
        Ok(config)
    }
}

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


///////////////////////////////////////////////////////////////
// SIMPLE AGENT REFERENCE IMPLEMENTATION
///////////////////////////////////////////////////////////////

// Agent info provider implementation
//#[derive(Clone)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAgentReference {
    pub name: String,
    pub url: String,
    pub is_default:Option<bool>,
}

impl SimpleAgentReference {
    pub fn new(name: String, url: String) -> anyhow::Result<SimpleAgentReference> {
        // Create the agent card
        Ok(SimpleAgentReference {
            name: name,
            url: url,
            is_default:None,
        })
    }
}

impl SimpleAgentReference {
    pub async fn get_agent_reference(&self) -> anyhow::Result<SimpleAgentReference> {
        Ok(self.clone())
    }
}

///////////////////////////////////////////////////////////////
// INTERACTION WITH DISCOVERY SERVICE
// to be moved to the discovery service crate
///////////////////////////////////////////////////////////////


pub trait DiscoveryServiceInteraction {
    //async fn register(&self, agent_card:AgentCard) -> Result<(), Box<dyn std::error::Error>> ;
    fn register(&self, agent_card:AgentCard) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send ;
}


impl DiscoveryServiceInteraction for AgentA2aConfig {
    /// Start both HTTP and WebSocket servers (simplified for now)
    async fn register(&self, agent_card:AgentCard) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Registering Agent ...");

        let discovery_url=self.agent_a2a_discovery_url.clone().expect("NO DISCOVERY URL");

        let register_uri=format!("{}/register",discovery_url);

        let agent_registered = reqwest::Client::new()
        .post(register_uri)
        .json(&agent_card)
        .send()
        .await;

        match agent_registered {
            Ok(response) => { println!("Successfully registered server agent: {:?}", response);}
            Err(e) => {
                if e.is_connect() {
                    eprintln!("Connection error: The target server is not up or reachable. Details: {:?}", e);
                } else if e.is_timeout() {
                    eprintln!("Request timed out: {:?}", e);
                } else if e.is_status() {
                    // Handle HTTP status errors (e.g., 404, 500)
                    eprintln!("HTTP status error: {:?}", e.status());
                } else {
                    eprintln!("An unexpected reqwest error occurred: {:?}", e);
                }
                //return Err(e);
            }
        }

        Ok(())
    }

}


impl DiscoveryServiceInteraction for AgentFullConfig {
    /// Start both HTTP and WebSocket servers (simplified for now)
    async fn register(&self, agent_card:AgentCard) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Registering Agent ...");

        let discovery_url=self.agent_full_discovery_url.clone().expect("NO DISCOVERY URL");

        let register_uri=format!("{}/register",discovery_url);

        let agent_registered = reqwest::Client::new()
        .post(register_uri)
        .json(&agent_card)
        .send()
        .await;

        match agent_registered {
            Ok(response) => { println!("Successfully registered server agent: {:?}", response);}
            Err(e) => {
                if e.is_connect() {
                    eprintln!("Connection error: The target server is not up or reachable. Details: {:?}", e);
                } else if e.is_timeout() {
                    eprintln!("Request timed out: {:?}", e);
                } else if e.is_status() {
                    // Handle HTTP status errors (e.g., 404, 500)
                    eprintln!("HTTP status error: {:?}", e.status());
                } else {
                    eprintln!("An unexpected reqwest error occurred: {:?}", e);
                }
                //return Err(e);
            }
        }

        Ok(())
    }

}