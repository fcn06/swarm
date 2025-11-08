use anyhow::Result;
use configuration::{AgentConfig};
use agent_models::factory::config::{AgentDomain, AgentType, FactoryAgentConfig, FactoryConfig, LlmProviderUrl};
use tracing::{info, debug};
use std::sync::Arc;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;

use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};
use basic_agent::business_logic::basic_agent::BasicAgent;
use planner_agent::business_logic::planner_agent::PlannerAgent;
use executor_agent::business_logic::executor_agent::ExecutorAgent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::agent::Agent;
use configuration::McpRuntimeConfig;
use agent_models::factory::config::FactoryMcpRuntimeConfig;

const GROQ_CHAT_COMPLETION_ENDPOINT: &'static str = "https://api.groq.com/openai/v1/chat/completions";
const GOOGLE_CHAT_COMPLETION_ENDPOINT: &'static str = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";
const LLAMACPP_CHAT_COMPLETION_ENDPOINT: &'static str = "http://localhost:2000/v1/chat/completions";

// launch services
// Create and launch agent with factory agentbuilder

pub struct AgentFactory {
    pub factory_config: FactoryConfig,
    pub factory_discovery_service: Arc<dyn DiscoveryService>,
    pub factory_memory_service: Option<Arc<dyn MemoryService>>,
    pub factory_evaluation_service: Option<Arc<dyn EvaluationService>>,
}

impl AgentFactory {
    pub fn new(factory_config: FactoryConfig) -> Self {
        AgentFactory {
            factory_config:factory_config.clone(),
            factory_discovery_service: Arc::new(AgentDiscoveryServiceAdapter::new(&factory_config.factory_discovery_url.expect("Factory Discovery URL not set"))),
            factory_memory_service: Some(Arc::new(AgentMemoryServiceAdapter::new(&factory_config.factory_memory_service_url.expect("Factory Memory Service URL not set")))),
            factory_evaluation_service: Some(Arc::new(AgentEvaluationServiceAdapter::new(&factory_config.factory_evaluation_service_url.expect("Factory Evaluation Service URL not set")))),
        }
    }

    pub fn create_agent_config(&self, factory_agent_config: &FactoryAgentConfig, agent_http_endpoint:String) -> Result<AgentConfig> {
        info!("Creating AgentConfig for agent: {}", factory_agent_config.factory_agent_name);

        let mut builder = AgentConfig::builder()
            .agent_name(factory_agent_config.factory_agent_name.clone())
            .agent_id(factory_agent_config.factory_agent_id.clone())
            .agent_description(factory_agent_config.factory_agent_description.clone())
            .agent_model_id(factory_agent_config.factory_agent_llm_model_id.clone());

        // Map LLM provider URL
        let llm_url = match &factory_agent_config.factory_agent_llm_provider_url {
            LlmProviderUrl::Groq => GROQ_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::Google => GOOGLE_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::LlamaCpp => LLAMACPP_CHAT_COMPLETION_ENDPOINT.to_string(),
        };
        builder = builder.agent_llm_url(llm_url);

        // Set common defaults or values from factory_config
        builder = builder.agent_http_endpoint(agent_http_endpoint)
                         .agent_ws_endpoint("ws://127.0.0.1:9000".to_string());

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

        // todo:add mcp config

        // Additional defaults if not already set
        let final_config = builder.build()?;
        debug!("Created AgentConfig: {:?}", final_config);
        Ok(final_config)
    }


    pub fn create_mcp_config(&self,factory_mcp_runtime_config:&FactoryMcpRuntimeConfig) -> Result<McpRuntimeConfig> {


        let mcp_runtime_system_prompt=r#"You are a helpful assistant that answers user requests. If you can answer a question using your general knowledge, do so. Otherwise, you can use one or more tools to find the answer. When you receive a message with a role called \"tool\", you must use the response from tools in order to build a final answer."#;

        let llm_mcp_url = match &factory_mcp_runtime_config.factory_mcp_llm_provider_url {
            LlmProviderUrl::Groq => GROQ_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::Google => GOOGLE_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::LlamaCpp => LLAMACPP_CHAT_COMPLETION_ENDPOINT.to_string(),
        };


        Ok(
            McpRuntimeConfig {
                agent_mcp_role_tool: "tool".to_string(),
                agent_mcp_role_assistant: "assistant".to_string(),
                agent_mcp_tool_choice_auto: "auto".to_string(),
                agent_mcp_finish_reason_tool_calls:"tool_calls".to_string(),
                agent_mcp_finish_reason_stop: "stop".to_string(),
                agent_mcp_max_loops: 5, // Use appropriate type
                agent_mcp_server_url: Some(factory_mcp_runtime_config.factory_mcp_server_url.clone()),
                agent_mcp_server_api_key:Some(factory_mcp_runtime_config.factory_mcp_server_api_key.clone()),
                agent_mcp_model_id: factory_mcp_runtime_config.factory_mcp_llm_model_id.clone(),
                agent_mcp_llm_url: llm_mcp_url, 
                agent_mcp_llm_api_key_env_var: Some(factory_mcp_runtime_config.factory_mcp_llm_provider_api_key.clone()), 
                agent_mcp_system_prompt: mcp_runtime_system_prompt.to_string(),
                agent_mcp_endpoint_url: None, 
            }
        )

    }



    pub fn get_factory_config(&self) -> &FactoryConfig {
        &self.factory_config
    }


    
    pub async fn launch_agent(&self, factory_agent_config: &FactoryAgentConfig, agent_type:AgentType,agent_http_endpoint: String) -> Result<()> {
        
        let agent_config = self.create_agent_config(factory_agent_config, agent_http_endpoint).expect("Error Creating Agent Config from Factory");
        
        match agent_type {
            AgentType::Specialist => {
                let agent = BasicAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(),None, None, None, None, None).await?;
                let server = AgentServer::<BasicAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
            AgentType::Planner => {
                let agent = PlannerAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(),None ,None, None, None, None).await?;
                let server = AgentServer::<PlannerAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
            AgentType::Executor => {
                let agent = ExecutorAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(),None, None, None, None, None).await?;
                let server = AgentServer::<ExecutorAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
        }

        Ok(())
    }

    pub async fn launch_agent_with_mcp(&self, factory_agent_config: &FactoryAgentConfig, 
        factory_mcp_runtime_config:&FactoryMcpRuntimeConfig, agent_type:AgentType,agent_http_endpoint: String) -> Result<()> {
        
        let agent_config = self.create_agent_config(factory_agent_config, agent_http_endpoint).expect("Error Creating Agent Config from Factory");
        let mcp_config=self.create_mcp_config(factory_mcp_runtime_config).expect("Error Creating MCP Config from Factory");

        let mcp_runtime_details=McpRuntimeDetails{
            config:mcp_config.clone(),
            api_key:factory_mcp_runtime_config.factory_mcp_llm_provider_api_key.clone(),
        };

        debug!("Agent Config: {:?}", agent_config);
        debug!("MCP Config: {:?}", mcp_config);


        match agent_type {
            AgentType::Specialist => {
                //let agent = BasicAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(), None, None, None, None).await?;
                let agent = BasicAgent::new(agent_config.clone(), 
                    factory_agent_config.factory_agent_llm_provider_api_key.clone(), 
                        Some(mcp_runtime_details.clone()),
                            None, None, None, None).await?;
                
                let server = AgentServer::<BasicAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
            AgentType::Planner => {
                let agent = PlannerAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(),None, None, None, None, None).await?;
                let server = AgentServer::<PlannerAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
            AgentType::Executor => {
                let agent = ExecutorAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(),None, None, None, None, None).await?;
                let server = AgentServer::<ExecutorAgent>::new(agent_config, agent, None).await?;
                server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            },
        }

        Ok(())
    }


}
