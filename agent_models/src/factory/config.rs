use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use toml;

// Store factory parameters in databases

// Contains framework level configuration


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryConfig {
    pub factory_discovery_url: Option<String>,
    pub factory_evaluation_service_url: Option<String>, // to remove from config and make it runtime
    pub factory_memory_service_url: Option<String>, // to remove from config and make it runtime
}

impl FactoryConfig {
    /// Loads factory configuration from a TOML file.
    pub fn load_factory_config(path: &str) -> anyhow::Result<FactoryConfig> {
        let config_content = fs::read_to_string(path)?;
        let config: FactoryConfig = toml::from_str(&config_content)?;
        Ok(config)
    }
}

// Contains agent level configuration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryAgentConfig {
    pub factory_agent_url:String,
    pub factory_agent_type:AgentType,
    pub factory_agent_domains:Option<AgentDomain>, // Apply only if agent is domain specialist
    pub factory_agent_name:String,
    pub factory_agent_description:String,
    pub factory_agent_llm_provider_url:LlmProviderUrl,
    pub factory_agent_llm_model_id:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    #[serde(rename = "specialist")]
    Specialist,
    #[serde(rename = "planner")]
    Planner,
    #[serde(rename = "executor")]
    Executor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentDomain {
    #[serde(rename = "general")]
    General,
    #[serde(rename = "finance")]
    Finance,
    #[serde(rename = "customer")]
    Customer,
    #[serde(rename = "weather")]
    Weather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProviderUrl {
    #[serde(rename = "https://api.groq.com/openai/v1/chat/completions")]
    Groq,
    #[serde(rename = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")]
    Google,
    #[serde(rename = "http://localhost:2000/v1/chat/completions")]
    LlamaCpp,
}

impl fmt::Display for LlmProviderUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LlmProviderUrl::Groq => write!(f, "https://api.groq.com/openai/v1/chat/completions"),
            LlmProviderUrl::Google => write!(f, "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"),
            LlmProviderUrl::LlamaCpp => write!(f, "http://localhost:2000/v1/chat/completions"),
        }
    }
}
