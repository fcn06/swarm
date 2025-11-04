use resource_invoker::McpRuntimeToolInvoker as McpRuntimeTools;
use std::env;

use clap::Parser;
use std::sync::Arc;

use configuration::{setup_logging};

use serde_json::json;
use agent_factory::agent_factory::AgentFactory;


use agent_core::business_logic::services::{ DiscoveryService};

// Registration via discovery service
use agent_models::registry::registry_models::{TaskDefinition,AgentDefinition,ToolDefinition};
use agent_models::factory::config::FactoryConfig;

use agent_models::factory::config::LlmProviderUrl;
use agent_models::factory::config::AgentDomain;
use agent_models::factory::config::AgentType;
use agent_models::factory::config::FactoryAgentConfig;



/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/factory_config.toml")]
    config_file: String,
    /// Log level
    #[clap(long, default_value = "warn")]
    log_level: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:4000")]
    discovery_service_url: String,
    #[clap(long, default_value = "http://127.0.0.1:5000")]
    memory_service_url: String,
    #[clap(long, default_value = "http://127.0.0.1:7000")]
    evaluation_service_url: String,
}

 
/// Register Tasks in Discovery Service
async fn register_tasks(discovery_service: Arc<dyn DiscoveryService>) -> anyhow::Result<()> {

    let task_definition=TaskDefinition {
        id: "greeting".to_string(),
        name: "Say Hello".to_string(),
        description: "Say hello to somebody".to_string(),
        input_schema: json!({}),
        output_schema: json!({}),
    };
    discovery_service.register_task(&task_definition).await?;
    Ok(())
}

/// Register Agents in Discovery Service
async fn register_agents(discovery_service: Arc<dyn DiscoveryService>) -> anyhow::Result<()> {

    // todo:harmonize agent_id and agent_name
    let agent_definition=AgentDefinition {
        id: "Basic_Agent".to_string(),
        name: "Basic Agent for weather requests, customer requests and other general topics".to_string(),
        description: "Retrieve Weather in a Location, Get customer details and other General Requests".to_string(),
        agent_endpoint: "http://localhost:8080".to_string(),
        skills: Vec::new(),
    };

    discovery_service.register_agent(&agent_definition).await?;
    Ok(())
}

/// Register Agents in Discovery Service
async fn register_tools(mcp_config_path: String,discovery_service: Arc<dyn DiscoveryService>) -> anyhow::Result<()> {

    let mcp_tools = McpRuntimeTools::new(mcp_config_path).await?;
    let mcp_tools = Arc::new(mcp_tools);

    // Register tools
        let list_tools= mcp_tools.get_tools_list_v2().await?;
        for tool in list_tools {
            let tool_definition=ToolDefinition {
                id:tool.function.name.clone(),
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                input_schema: serde_json::to_value(&tool.function.parameters).unwrap_or_else(|_| json!({})),
                output_schema: json!({}),
            };
            discovery_service.register_tool(&tool_definition).await?;
        }
  
    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

    /************************************************/
    /* Loading Factory Config File                  */
    /* Creating Agent Factory                       */
    /************************************************/ 
    let factory_config = FactoryConfig::load_factory_config(&args.config_file).expect("Incorrect Factory Config File");
    let agent_factory=AgentFactory::new(factory_config);

    /************************************************/
    /* Set Up Registrations via discovery service           */
    /************************************************/ 
    register_tasks(agent_factory.factory_discovery_service.clone()).await?;
    register_agents(agent_factory.factory_discovery_service.clone()).await?;
    register_tools(args.mcp_config_path.clone(),agent_factory.factory_discovery_service.clone()).await?;

    /************************************************/
    /* Launch Agents from Factor                    */
    /************************************************/ 

    let agent_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");


    let factory_agent_config = FactoryAgentConfig::builder()
        .with_factory_agent_url("http://127.0.0.1:8080".to_string())
        .with_factory_agent_type(AgentType::Specialist)
        .with_factory_agent_domains(AgentDomain::General)
        .with_factory_agent_name("Basic_Agent".to_string())
        .with_factory_agent_description("An Agent that answer Basic Questions".to_string())
        .with_factory_agent_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_agent_llm_provider_api_key(agent_api_key)
        .with_factory_agent_llm_model_id("openai/gpt-oss-20b".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryAgentConfig: {}", e))?;

        //todo: add mcp_config to factory_agent_config


    agent_factory.launch_agent(&factory_agent_config,AgentType::Specialist,"http://127.0.0.1:8080".to_string()).await?;

    /************************************************/
    /* Agent  launched                              */
    /************************************************/ 


    Ok(())
}
