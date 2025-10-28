use anyhow::Result;
use configuration::{AgentConfig, AgentMcpConfig};
use agent_models::factory::config::{AgentDomain, AgentType, FactoryAgentConfig, FactoryConfig, LlmProviderUrl};
use tracing::{info, debug};


/*


async fn setup_evaluation_service(planner_agent_config: &AgentConfig) -> Option<Arc<dyn EvaluationService>> {
    if let Some(url) = planner_agent_config.agent_evaluation_service_url() {
        info!("Evaluation service configured at: {}", url);
        let adapter = AgentEvaluationServiceAdapter::new(&url);
        Some(Arc::new(adapter))
    } else {
        warn!("Evaluation service URL not configured. No evaluations will be logged.");
        None
    }
}

async fn setup_memory_service(planner_agent_config: &AgentConfig) -> Option<Arc<dyn MemoryService>> {
    if let Some(url) = planner_agent_config.agent_memory_service_url() {
        info!("Memory service configured at: {}", url);
        let adapter = AgentMemoryServiceAdapter::new(&url);
        Some(Arc::new(adapter))
    } else {
        warn!("Memory service URL not configured. No memory will be used.");
        None
    }
}

async fn setup_discovery_service(discovery_url: String) -> Option<Arc<dyn DiscoveryService>> {
    info!("Discovery service configured at: {}", discovery_url);
    let adapter = AgentDiscoveryServiceAdapter::new(&discovery_url);
    Some(Arc::new(adapter))
}

*/





pub struct AgentFactory {
    factory_config: FactoryConfig,
}

impl AgentFactory {
    pub fn new(factory_config: FactoryConfig) -> Self {
        AgentFactory {
            factory_config,
        }
    }

    pub fn create_agent_config(&self, factory_agent_config: &FactoryAgentConfig, host:String,http_port:String,ws_port:String) -> Result<AgentConfig> {
        info!("Creating AgentConfig for agent: {}", factory_agent_config.factory_agent_name);

        let mut builder = AgentConfig::builder()
            .agent_name(factory_agent_config.factory_agent_name.clone())
            .agent_description(factory_agent_config.factory_agent_description.clone())
            .agent_model_id(factory_agent_config.factory_agent_llm_model_id.clone());

        // Map LLM provider URL
        let llm_url = match &factory_agent_config.factory_agent_llm_provider_url {
            LlmProviderUrl::Groq => "https://api.groq.com/openai/v1/chat/completions".to_string(),
            LlmProviderUrl::Google => "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".to_string(),
            LlmProviderUrl::LlamaCpp => "http://localhost:2000/v1/chat/completions".to_string(),
        };
        builder = builder.agent_llm_url(llm_url);

        // Set common defaults or values from factory_config
        builder = builder.agent_host(host)
                         .agent_http_port(http_port)
                         .agent_ws_port(ws_port);
                        // .agent_discovery_url(self.factory_config.factory_discovery_url.clone().unwrap_or_default());

        // Set agent-type specific defaults or logic
        match factory_agent_config.factory_agent_type {
            AgentType::Specialist => {
                builder = builder.agent_system_prompt("You are a helpful assistant.".to_string())
                                 .agent_skill_id("generic_skill".to_string())
                                 .agent_skill_name("Generic Skill".to_string())
                                 .agent_skill_description("A generic skill for various tasks.".to_string())
                                 .agent_version("1.0.0".to_string())
                                 .agent_doc_url("/docs".to_string())
                                 .agent_tags(vec!["general".to_string()])
                                 .agent_examples(vec!["Hello".to_string()]);

                if let Some(domain) = &factory_agent_config.factory_agent_domains {
                    match domain {
                        AgentDomain::General => { /* already set */ },
                        AgentDomain::Finance => {
                            builder = builder.agent_system_prompt("You are a financial advisor.".to_string())
                                             .agent_skill_id("finance_skill".to_string())
                                             .agent_skill_name("Finance Advisor Skill".to_string())
                                             .agent_skill_description("Advises on financial matters.".to_string())
                                             .agent_tags(vec!["finance".to_string()]);
                        },
                        AgentDomain::Customer => {
                            builder = builder.agent_system_prompt("You are a customer support agent.".to_string())
                                             .agent_skill_id("customer_skill".to_string())
                                             .agent_skill_name("Customer Support Skill".to_string())
                                             .agent_skill_description("Assists customers with their queries.".to_string())
                                             .agent_tags(vec!["customer".to_string()]);
                        },
                        AgentDomain::Weather => {
                            builder = builder.agent_system_prompt("You are a weather forecaster.".to_string())
                                             .agent_skill_id("weather_skill".to_string())
                                             .agent_skill_name("Weather Forecasting Skill".to_string())
                                             .agent_skill_description("Provides weather updates.".to_string())
                                             .agent_tags(vec!["weather".to_string()]);
                        },
                    }
                }
            },
            AgentType::Planner => {
                builder = builder.agent_system_prompt("You are a planner agent, capable of creating detailed plans.".to_string())
                                 .agent_skill_id("planner_skill".to_string())
                                 .agent_skill_name("Planning Skill".to_string())
                                 .agent_skill_description("Creates multi-step plans for complex tasks.".to_string())
                                 .agent_version("1.0.0".to_string())
                                 .agent_doc_url("/docs".to_string())
                                 .agent_tags(vec!["plan".to_string(), "orchestration".to_string()]);
            },
            AgentType::Executor => {
                builder = builder.agent_system_prompt("You are an executor agent that executes precsiley workflow that you are delegated.".to_string())
                                 .agent_skill_id("workflow_execution".to_string())
                                 .agent_skill_name("Execute Strictly Defined Workflow".to_string())
                                 .agent_skill_description("Receives a workflow definition as an input and execute it".to_string())
                                 .agent_version("1.0.0".to_string())
                                 .agent_doc_url("/docs".to_string())
                                 .agent_tags(vec!["execute plan".to_string()]);
            },
        }

        // Additional defaults if not already set
        let final_config = builder.build()?;
        debug!("Created AgentConfig: {:?}", final_config);
        Ok(final_config)
    }

    pub fn create_mcp_config(&self) -> Result<AgentMcpConfig> {
        info!("Creating AgentMcpConfig from FactoryConfig");

        // Default values for AgentMcpConfig
        let mcp_role_tool = "tool".to_string();
        let mcp_role_assistant = "assistant".to_string();
        let mcp_tool_choice_auto = "auto".to_string();
        let mcp_finish_reason_tool_calls = "tool_calls".to_string();
        let mcp_finish_reason_stop = "stop".to_string();
        let mcp_max_loops = 5; // Sensible default
        let mcp_system_prompt = "You are a helpful assistant that can use tools.".to_string();

        // Values from FactoryConfig, with fallbacks to defaults
        let mcp_server_url = self.factory_config.factory_discovery_url.clone(); // Reusing discovery URL for MCP server as an example
        let mcp_server_api_key = None; // API key might be loaded from environment or a more secure source
        let mcp_model_id = "gemini-2.0-flash".to_string(); // Default MCP LLM model
        let mcp_llm_url = LlmProviderUrl::Google.to_string(); // Using the Display implementation
        let mcp_endpoint_url = None;

        Ok(AgentMcpConfig {
            agent_mcp_role_tool: mcp_role_tool,
            agent_mcp_role_assistant: mcp_role_assistant,
            agent_mcp_tool_choice_auto: mcp_tool_choice_auto,
            agent_mcp_finish_reason_tool_calls: mcp_finish_reason_tool_calls,
            agent_mcp_finish_reason_stop: mcp_finish_reason_stop,
            agent_mcp_max_loops: mcp_max_loops,
            agent_mcp_server_url: mcp_server_url,
            agent_mcp_server_api_key: mcp_server_api_key,
            agent_mcp_model_id: mcp_model_id,
            agent_mcp_llm_url: mcp_llm_url,
            agent_mcp_system_prompt: mcp_system_prompt,
            agent_mcp_endpoint_url: mcp_endpoint_url,
        })
    }

    pub fn get_factory_config(&self) -> &FactoryConfig {
        &self.factory_config
    }
}
