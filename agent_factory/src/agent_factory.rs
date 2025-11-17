use anyhow::Result;
use configuration::{AgentConfig, AgentConfigBuilder};
use agent_models::factory::config::{AgentDomain, AgentType, FactoryAgentConfig, FactoryConfig, LlmProviderUrl};
use tracing::{info, debug};
use std::sync::Arc;
use tokio::task::JoinHandle;

use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService, WorkflowServiceApi};
use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::agent::Agent;

use basic_agent::business_logic::basic_agent::BasicAgent;
use planner_agent::business_logic::planner_agent::PlannerAgent;
use executor_agent::business_logic::executor_agent::ExecutorAgent;

use configuration::McpRuntimeConfig;
use agent_models::factory::config::FactoryMcpRuntimeConfig;

use executor_agent::business_logic::executor_agent::WorkFlowInvokers;

// Constants for LLM API endpoints
const GROQ_CHAT_COMPLETION_ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";
const GOOGLE_CHAT_COMPLETION_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";
const LLAMACPP_CHAT_COMPLETION_ENDPOINT: &str = "http://localhost:2000/v1/chat/completions";

// System prompts for different agent types
const PLANNER_SYSTEM_PROMPT: &str = r#"
You are an expert workflow generation AI. Your mission is to create highly efficient, well-structured, and accurate JSON workflow definitions for multi-agent systems. You have access to a specific set of agents, tools, and tasks. Your outputs must strictly adhere to the provided JSON schema and the following rules.

1. Maximize Agent Autonomy 🤝:
When an agent (e.g., Basic_Agent) has the inherent capability and tools to perform a sequence of related operations, consolidate these steps into a single delegation_agent activity.
Empower agents to handle their own internal logic and tool orchestration. Don't break a single logical action into multiple steps if one agent can handle it.
For example, if an agent can extract a location from a user request and then use that location with a weather tool, define this as a single delegation_agent activity.

2. Use agent_context for Dynamic Data 🧩:
NEVER embed variable references ({{...}}) directly into the description field. description is for a static, human-readable summary.
All dynamic data, especially outputs from previous activities, MUST be passed to a delegation_agent via its agent_context field. The agent will handle the processing internally.

3. Ensure Clear Dependencies 🔗:
Always specify dependencies correctly to reflect the execution and data flow.
If an activity relies on the output of a preceding activity, that dependency must be explicitly listed.

4. Be Concise & Accurate 🎯:
Keep activity description fields brief and focused on the objective.
Ensure all tool and agent IDs are from the provided list.
"#;

const EXECUTOR_SYSTEM_PROMPT: &str = "You are an executor agent that executes precsiley workflow that you are delegated.";

const MCP_RUNTIME_SYSTEM_PROMPT: &str = r#"You are a helpful assistant that answers user requests. If you can answer a question using your general knowledge, do so. Otherwise, you can use one or more tools to find the answer. When you receive a message with a role called "tool", you must use the response from tools in order to build a final answer."#;

/********************************************************/
// Configurator
/********************************************************/

/// Trait for configuring an AgentConfigBuilder based on agent type and domain.
trait AgentConfigurator {
    fn configure_agent_defaults(&self, builder: AgentConfigBuilder, factory_agent_config: &FactoryAgentConfig) -> Result<AgentConfigBuilder>;
}

/// Configurator for Specialist Agents
struct SpecialistAgentConfigurator;

impl AgentConfigurator for SpecialistAgentConfigurator {
    fn configure_agent_defaults(&self, mut builder: AgentConfigBuilder, factory_agent_config: &FactoryAgentConfig) -> Result<AgentConfigBuilder> {
        builder = builder.agent_system_prompt("You are a helpful assistant.".to_string())
                         .agent_discoverable(true)
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
        Ok(builder)
    }
}

/// Configurator for Planner Agents
struct PlannerAgentConfigurator;

impl AgentConfigurator for PlannerAgentConfigurator {
    fn configure_agent_defaults(&self, mut builder: AgentConfigBuilder, factory_agent_config: &FactoryAgentConfig) -> Result<AgentConfigBuilder> {
        builder = builder.agent_system_prompt(PLANNER_SYSTEM_PROMPT.to_string())
                         .agent_executor_url(factory_agent_config.factory_agent_executor_url.clone().expect("Executor URL not set"))
                         .agent_skill_id("planner_skill".to_string())
                         .agent_skill_name("Planning Skill".to_string())
                         .agent_skill_description("Creates multi-step plans for complex tasks.".to_string())
                         .agent_version("1.0.0".to_string())
                         .agent_doc_url("/docs".to_string())
                         .agent_tags(vec!["plan".to_string(), "orchestration".to_string()]);
        Ok(builder)
    }
}

/// Configurator for Executor Agents
struct ExecutorAgentConfigurator;

impl AgentConfigurator for ExecutorAgentConfigurator {
    fn configure_agent_defaults(&self, mut builder: AgentConfigBuilder, _factory_agent_config: &FactoryAgentConfig) -> Result<AgentConfigBuilder> {
        builder = builder.agent_system_prompt(EXECUTOR_SYSTEM_PROMPT.to_string())
                         .agent_skill_id("workflow_execution".to_string())
                         .agent_skill_name("Execute Strictly Defined Workflow".to_string())
                         .agent_skill_description("Receives a workflow definition as an input and execute it".to_string())
                         .agent_version("1.0.0".to_string())
                         .agent_doc_url("/docs".to_string())
                         .agent_tags(vec!["execute plan".to_string()]);
        Ok(builder)
    }
}


/********************************************************/
// End Configurator
/********************************************************/


/********************************************************/
// Agent Factory
/********************************************************/


pub struct AgentFactory {
    pub factory_config: FactoryConfig,
    pub factory_discovery_service: Arc<dyn DiscoveryService>,
    pub factory_memory_service: Option<Arc<dyn MemoryService>>,
    pub factory_evaluation_service: Option<Arc<dyn EvaluationService>>,
    pub workflow_service: Option<Arc<dyn WorkflowServiceApi>>, // Reverted to WorkflowServiceApi
}

impl AgentFactory {
    pub fn new(
        factory_config: FactoryConfig,
        factory_discovery_service: Arc<dyn DiscoveryService>,
        factory_memory_service: Option<Arc<dyn MemoryService>>,
        factory_evaluation_service: Option<Arc<dyn EvaluationService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>, // Reverted to WorkflowServiceApi
    ) -> Self {
        AgentFactory {
            factory_config: factory_config.clone(),
            factory_discovery_service,
            factory_memory_service,
            factory_evaluation_service,
            workflow_service, // Stored directly
        }
    }

    pub fn create_agent_config(&self, factory_agent_config: &FactoryAgentConfig) -> Result<AgentConfig> {
        info!("Creating AgentConfig for agent: {}", factory_agent_config.factory_agent_name);

        let mut builder = AgentConfig::builder()
            .agent_name(factory_agent_config.factory_agent_name.clone())
            .agent_id(factory_agent_config.factory_agent_id.clone())
            .agent_description(factory_agent_config.factory_agent_description.clone())
            .agent_model_id(factory_agent_config.factory_agent_llm_model_id.clone());

        let llm_url = match &factory_agent_config.factory_agent_llm_provider_url {
            LlmProviderUrl::Groq => GROQ_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::Google => GOOGLE_CHAT_COMPLETION_ENDPOINT.to_string(),
            LlmProviderUrl::LlamaCpp => LLAMACPP_CHAT_COMPLETION_ENDPOINT.to_string(),
        };
        builder = builder.agent_llm_url(llm_url);

        builder = builder.agent_http_endpoint(factory_agent_config.factory_agent_url.clone())
                        .agent_ws_endpoint("ws://127.0.0.1:9000".to_string());

        // Use the appropriate configurator based on agent type
        let final_builder = match factory_agent_config.factory_agent_type {
            AgentType::Specialist => SpecialistAgentConfigurator.configure_agent_defaults(builder, factory_agent_config)?,
            AgentType::Planner => PlannerAgentConfigurator.configure_agent_defaults(builder, factory_agent_config)?,
            AgentType::Executor => ExecutorAgentConfigurator.configure_agent_defaults(builder, factory_agent_config)?,
        };
        
        let final_config = final_builder.build()?;
        debug!("Created AgentConfig: {:?}", final_config);
        Ok(final_config)
    }


    pub fn create_mcp_config(&self,factory_mcp_runtime_config:&FactoryMcpRuntimeConfig) -> Result<McpRuntimeConfig> {

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
                agent_mcp_system_prompt: MCP_RUNTIME_SYSTEM_PROMPT.to_string(),
                agent_mcp_endpoint_url: None, 
            }
        )

    }



    pub fn get_factory_config(&self) -> &FactoryConfig {
        &self.factory_config
    }

    
    async fn launch_agent_server<A: Agent + 'static + Sync + Send>(
        agent_config: AgentConfig,
        agent: A,
        discovery_service: Option<Arc<dyn DiscoveryService>>,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let server = AgentServer::<A>::new(agent_config, agent, discovery_service).await?;
            server.start_http().await.map_err(|e| anyhow::anyhow!("{}", e))
        })
    }


    pub async fn launch_agent(&self, 
        factory_agent_config: &FactoryAgentConfig, 
        mcp_runtime_config: Option<&FactoryMcpRuntimeConfig>, 
        agent_type:AgentType) -> Result<JoinHandle<Result<()>>> {
        
        let agent_config = self.create_agent_config(factory_agent_config).expect("Error Creating Agent Config from Factory");
        
        let mcp_runtime_details = if let Some(config) = mcp_runtime_config {
            let mcp_config = self.create_mcp_config(config).expect("Error Creating MCP Config from Factory");
            debug!("MCP Config: {:?}", mcp_config);
            Some(McpRuntimeDetails {
                config: mcp_config,
                api_key: config.factory_mcp_llm_provider_api_key.clone(),
            })
        } else {
            None
        };

        debug!("Agent Config: {:?}", agent_config);

        let handle = match agent_type {
            AgentType::Specialist => {
                let agent = BasicAgent::new(agent_config.clone(), 
                    factory_agent_config.factory_agent_llm_provider_api_key.clone(),
                        mcp_runtime_details.clone(), 
                                None, None, 
                                    Some(self.factory_discovery_service.clone()), 
                                        None).await?;
                Self::launch_agent_server(agent_config, agent, Some(self.factory_discovery_service.clone())).await
            },

            AgentType::Planner => {
                let evaluation_service = if factory_agent_config.factory_agent_is_evaluated {
                    self.factory_evaluation_service.clone()
                } else {
                    None
                };

                let agent = PlannerAgent::new(agent_config.clone(), 
                    factory_agent_config.factory_agent_llm_provider_api_key.clone(),
                        None ,
                            evaluation_service.clone(),  
                                None, 
                                    Some(self.factory_discovery_service.clone()), 
                                        None).await?;
                Self::launch_agent_server(agent_config, agent, None).await
            },
            
            AgentType::Executor => {
                let agent = ExecutorAgent::new(agent_config.clone(), 
                    factory_agent_config.factory_agent_llm_provider_api_key.clone(),
                        None, 
                            None, None, 
                            Some(self.factory_discovery_service.clone()),
                                self.workflow_service.clone()).await?;
                Self::launch_agent_server(agent_config, agent, None).await
            },
        };


        // Refresh Agents after each agent launched
        if let Some(ws_arc) = &self.workflow_service {
            // Downcast the Arc<dyn WorkflowServiceApi> to get a reference to WorkFlowInvokers
            let workflow_service_invoker = ws_arc.as_ref() // Get &dyn WorkflowServiceApi
                                      .as_any()   // Convert to &dyn Any
                                      .downcast_ref::<WorkFlowInvokers>() // Attempt to downcast to &WorkFlowInvokers
                                      .expect("WorkflowServiceApi is not a WorkFlowInvokers. Cannot refresh agents correctly.");
            workflow_service_invoker.refresh_agents().await?;
        }


        Ok(handle)
    }

/********************************************************/
// End Agent Factory
/********************************************************/


}
