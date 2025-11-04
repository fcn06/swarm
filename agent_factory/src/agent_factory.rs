use anyhow::Result;
use configuration::{AgentConfig};
use agent_models::factory::config::{AgentDomain, AgentType, FactoryAgentConfig, FactoryConfig, LlmProviderUrl};
use tracing::{info, debug};
use std::sync::Arc;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};
use basic_agent::business_logic::basic_agent::BasicAgent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::agent::Agent;


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

    pub fn create_agent_config(&self, factory_agent_config: &FactoryAgentConfig, host:String,http_port:String) -> Result<AgentConfig> {
        info!("Creating AgentConfig for agent: {}", factory_agent_config.factory_agent_name);

        let mut builder = AgentConfig::builder()
            .agent_name(factory_agent_config.factory_agent_name.clone())
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
        builder = builder.agent_host(host)
                         .agent_http_port(http_port)
                         .agent_ws_port("9000".to_string());

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



    pub fn get_factory_config(&self) -> &FactoryConfig {
        &self.factory_config
    }

    pub async fn launch_agent(&self, factory_agent_config: &FactoryAgentConfig, agent_type:AgentType,agent_endpoint: String) -> Result<()> {
        
        let host="127.0.0.1".to_string();
        let http_port="8080".to_string();
        
        let agent_config = self.create_agent_config(factory_agent_config, host, http_port).expect("Error Creating Agent Config from Factory");
        
        let agent = BasicAgent::new(agent_config.clone(), factory_agent_config.factory_agent_llm_provider_api_key.clone(), None, None, None, None).await?;
        // Create the modern server, and pass the runtime elements
        let server = AgentServer::<BasicAgent>::new(agent_config, agent, None).await?;

        //println!("🌐 Starting HTTP server only...");
        server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    // todo: load agent config from file

}
