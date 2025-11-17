use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use toml;

// Store factory parameters in databases


/******************************************************************/
// Factory Configuration
// Contains framework level configuration
/******************************************************************/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryConfig {
    pub factory_discovery_url: String,
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


/******************************************************************/
// Factory Agent Configuration
// Contains agent level configuration
/******************************************************************/

// todo: property to define if it is going to be evaluated or not

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryAgentConfig {
    pub factory_agent_url: String,
    pub factory_agent_type: AgentType,
    pub factory_agent_domains: Option<AgentDomain>, // Apply only if agent is domain specialist
    pub factory_agent_id: String,
    pub factory_agent_name: String,
    pub factory_agent_description: String,
    pub factory_agent_llm_provider_url: LlmProviderUrl,
    pub factory_agent_llm_provider_api_key: String, // to be injected at runtime
    pub factory_agent_llm_model_id: String,
    pub factory_agent_mcp_runtime_config: Option<FactoryMcpRuntimeConfig>,
    pub factory_agent_is_evaluated: bool,
    pub factory_agent_executor_url: Option<String>,
}

impl FactoryAgentConfig {
    pub fn builder() -> FactoryAgentConfigBuilder {
        FactoryAgentConfigBuilder::new()
    }
}

pub struct FactoryAgentConfigBuilder {
    factory_agent_url: Option<String>,
    factory_agent_type: Option<AgentType>,
    factory_agent_domains: Option<AgentDomain>,
    factory_agent_name: Option<String>,
    factory_agent_id: Option<String>,
    factory_agent_description: Option<String>,
    factory_agent_llm_provider_url: Option<LlmProviderUrl>,
    factory_agent_llm_provider_api_key: Option<String>,
    factory_agent_llm_model_id: Option<String>,
    factory_agent_mcp_runtime_config: Option<FactoryMcpRuntimeConfig>,
    factory_agent_is_evaluated: bool,
    factory_agent_executor_url: Option<String>,
}

impl FactoryAgentConfigBuilder {
    pub fn new() -> Self {
        FactoryAgentConfigBuilder {
            factory_agent_url: None,
            factory_agent_type: None,
            factory_agent_domains: None,
            factory_agent_name: None,
            factory_agent_id: None,
            factory_agent_description: None,
            factory_agent_llm_provider_url: None,
            factory_agent_llm_provider_api_key: None,
            factory_agent_llm_model_id: None,
            factory_agent_mcp_runtime_config:None,
            factory_agent_is_evaluated:false, // false by default
            factory_agent_executor_url: None,
        }
    }

    pub fn with_factory_agent_url(mut self, factory_agent_url: String) -> Self {
        self.factory_agent_url = Some(factory_agent_url);
        self
    }

    pub fn with_factory_agent_type(mut self, factory_agent_type: AgentType) -> Self {
        self.factory_agent_type = Some(factory_agent_type);
        self
    }

    pub fn with_factory_agent_domains(mut self, factory_agent_domains: AgentDomain) -> Self {
        self.factory_agent_domains = Some(factory_agent_domains);
        self
    }

    pub fn with_factory_agent_name(mut self, factory_agent_name: String) -> Self {
        self.factory_agent_name = Some(factory_agent_name);
        self
    }

    pub fn with_factory_agent_id(mut self, factory_agent_id: String) -> Self {
        self.factory_agent_id = Some(factory_agent_id);
        self
    }

    pub fn with_factory_agent_description(mut self, factory_agent_description: String) -> Self {
        self.factory_agent_description = Some(factory_agent_description);
        self
    }

    pub fn with_factory_agent_llm_provider_url(mut self, factory_agent_llm_provider_url: LlmProviderUrl) -> Self {
        self.factory_agent_llm_provider_url = Some(factory_agent_llm_provider_url);
        self
    }

    pub fn with_factory_agent_llm_provider_api_key(mut self, factory_agent_llm_provider_api_key: String) -> Self {
        self.factory_agent_llm_provider_api_key = Some(factory_agent_llm_provider_api_key);
        self
    }

    pub fn with_factory_agent_llm_model_id(mut self, factory_agent_llm_model_id: String) -> Self {
        self.factory_agent_llm_model_id = Some(factory_agent_llm_model_id);
        self
    }

    pub fn with_factory_agent_mcp_runtime_config(mut self, factory_agent_mcp_runtime_config: FactoryMcpRuntimeConfig) -> Self {
        self.factory_agent_mcp_runtime_config = Some(factory_agent_mcp_runtime_config);
        self
    }

    pub fn with_factory_agent_is_evaluated(mut self, factory_agent_is_evaluated: bool) -> Self {
        self.factory_agent_is_evaluated = factory_agent_is_evaluated;
        self
    }

    pub fn with_factory_agent_executor_url(mut self, factory_agent_executor_url: String) -> Self {
        self.factory_agent_executor_url = Some(factory_agent_executor_url);
        self
    }

    pub fn build(self) -> Result<FactoryAgentConfig, String> {
        let factory_agent_url = self.factory_agent_url.ok_or_else(|| "factory_agent_url is not set".to_string())?;
        let factory_agent_type = self.factory_agent_type.ok_or_else(|| "factory_agent_type is not set".to_string())?;
        let factory_agent_name = self.factory_agent_name.ok_or_else(|| "factory_agent_name is not set".to_string())?;
        let factory_agent_id = self.factory_agent_id.ok_or_else(|| "factory_agent_id is not set".to_string())?;
        let factory_agent_description = self.factory_agent_description.ok_or_else(|| "factory_agent_description is not set".to_string())?;
        let factory_agent_llm_provider_url = self.factory_agent_llm_provider_url.ok_or_else(|| "factory_agent_llm_provider_url is not set".to_string())?;
        let factory_agent_llm_provider_api_key = self.factory_agent_llm_provider_api_key.ok_or_else(|| "factory_agent_llm_provider_api_key is not set".to_string())?;
        let factory_agent_llm_model_id = self.factory_agent_llm_model_id.ok_or_else(|| "factory_agent_llm_model_id is not set".to_string())?;

        Ok(FactoryAgentConfig {
            factory_agent_url,
            factory_agent_type,
            factory_agent_domains: self.factory_agent_domains,
            factory_agent_name,
            factory_agent_id,
            factory_agent_description,
            factory_agent_llm_provider_url,
            factory_agent_llm_provider_api_key,
            factory_agent_llm_model_id,
            factory_agent_mcp_runtime_config : self.factory_agent_mcp_runtime_config,
            factory_agent_is_evaluated: self.factory_agent_is_evaluated,
            factory_agent_executor_url: self.factory_agent_executor_url,
        })
    }
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

/******************************************************************/
// Factory MCP Runtime Configuration
/******************************************************************/


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryMcpRuntimeConfig {
    pub factory_mcp_llm_provider_url: LlmProviderUrl,
    pub factory_mcp_llm_provider_api_key: String, // to be injected at runtime
    pub factory_mcp_llm_model_id: String,
    pub factory_mcp_server_url: String,
    pub factory_mcp_server_api_key: String,
}

impl FactoryMcpRuntimeConfig {
    pub fn builder() -> FactoryMcpRuntimeConfigBuilder {
        FactoryMcpRuntimeConfigBuilder::new()
    }
}

pub struct FactoryMcpRuntimeConfigBuilder {
    factory_mcp_llm_provider_url: Option<LlmProviderUrl>,
    factory_mcp_llm_provider_api_key: Option<String>,
    factory_mcp_llm_model_id: Option<String>,
    factory_mcp_server_url: Option<String>,
    factory_mcp_server_api_key: Option<String>,
}

impl FactoryMcpRuntimeConfigBuilder {
    pub fn new() -> Self {
        FactoryMcpRuntimeConfigBuilder {
            factory_mcp_llm_provider_url: None,
            factory_mcp_llm_provider_api_key: None,
            factory_mcp_llm_model_id: None,
            factory_mcp_server_url: None,
            factory_mcp_server_api_key: None,
        }
    }

    pub fn with_factory_mcp_llm_provider_url(mut self, factory_mcp_llm_provider_url: LlmProviderUrl) -> Self {
        self.factory_mcp_llm_provider_url = Some(factory_mcp_llm_provider_url);
        self
    }

    pub fn with_factory_mcp_llm_provider_api_key(mut self, factory_mcp_llm_provider_api_key: String) -> Self {
        self.factory_mcp_llm_provider_api_key = Some(factory_mcp_llm_provider_api_key);
        self
    }

    pub fn with_factory_mcp_llm_model_id(mut self, factory_mcp_llm_model_id: String) -> Self {
        self.factory_mcp_llm_model_id = Some(factory_mcp_llm_model_id);
        self
    }

    pub fn with_factory_mcp_server_url(mut self, factory_mcp_server_url: String) -> Self {
        self.factory_mcp_server_url = Some(factory_mcp_server_url);
        self
    }

    pub fn with_factory_mcp_server_api_key(mut self, factory_mcp_server_api_key: String) -> Self {
        self.factory_mcp_server_api_key = Some(factory_mcp_server_api_key);
        self
    }

    pub fn build(self) -> Result<FactoryMcpRuntimeConfig, String> {
        let factory_mcp_llm_provider_url = self.factory_mcp_llm_provider_url.ok_or_else(|| "factory_mcp_llm_provider_url is not set".to_string())?;
        let factory_mcp_llm_provider_api_key = self.factory_mcp_llm_provider_api_key.ok_or_else(|| "factory_mcp_llm_provider_api_key is not set".to_string())?;
        let factory_mcp_llm_model_id = self.factory_mcp_llm_model_id.ok_or_else(|| "factory_mcp_llm_model_id is not set".to_string())?;
        let factory_mcp_server_url = self.factory_mcp_server_url.ok_or_else(|| "factory_mcp_server_url is not set".to_string())?;
        let factory_mcp_server_api_key = self.factory_mcp_server_api_key.ok_or_else(|| "factory_mcp_server_api_key is not set".to_string())?;

        Ok(FactoryMcpRuntimeConfig {
            factory_mcp_llm_provider_url,
            factory_mcp_llm_provider_api_key,
            factory_mcp_llm_model_id,
            factory_mcp_server_url,
            factory_mcp_server_api_key,
        })
    }
}
