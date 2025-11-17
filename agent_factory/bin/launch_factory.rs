use std::env;
use std::sync::Arc;

use clap::Parser;

use tracing::{info,error};
use futures::future::join_all;
use serde_json::json;

use configuration::{setup_logging};

use agent_factory::agent_factory::AgentFactory;

use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService,WorkflowServiceApi};

// Registration via discovery service
use agent_models::registry::registry_models::{TaskDefinition,ToolDefinition};

use agent_models::factory::config::FactoryConfig;
use agent_models::factory::config::LlmProviderUrl;
use agent_models::factory::config::AgentDomain;
use agent_models::factory::config::AgentType;
use agent_models::factory::config::FactoryAgentConfig;
use agent_models::factory::config::FactoryMcpRuntimeConfig;

// Invokers
use resource_invoker::McpRuntimeToolInvoker;
use resource_invoker::GreetTask;
use resource_invoker::A2AAgentInvoker;
use resource_invoker::McpRuntimeToolInvoker as McpRuntimeTools;

use workflow_management::agent_communication::agent_invoker::AgentInvoker;
use workflow_management::tasks::task_invoker::TaskInvoker;
use workflow_management::tools::tool_invoker::ToolInvoker;
use executor_agent::business_logic::executor_agent::WorkFlowInvokers;

use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};



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

/***********************************************************************************/
// Initialization of evaluation, memory, discovery services
/***********************************************************************************/

async fn setup_evaluation_service(evaluation_service_url:&String) -> Option<Arc<dyn EvaluationService>> {
    info!("Evaluation service configured at: {}", evaluation_service_url);
    let adapter = AgentEvaluationServiceAdapter::new(&evaluation_service_url);
    Some(Arc::new(adapter))

}

async fn setup_memory_service(memory_service_url:&String) -> Option<Arc<dyn MemoryService>> {
    info!("Memory service configured at: {}", memory_service_url);
    let adapter = AgentMemoryServiceAdapter::new(&memory_service_url);
    Some(Arc::new(adapter))
}

async fn setup_discovery_service(discovery_service_url: &String) -> Arc<dyn DiscoveryService> {
info!("Discovery service configured at: {}", discovery_service_url);
let adapter = AgentDiscoveryServiceAdapter::new(&discovery_service_url);
Arc::new(adapter)
}

/***********************************************************************************/
// End of Services Initialization
/***********************************************************************************/

/***********************************************************************************/
// Initialization of Invoker Services
/***********************************************************************************/


async fn setup_task_invoker() -> anyhow::Result<Arc<dyn TaskInvoker>> {
    let greet_task_invoker = GreetTask::new()?;
    let greet_task_invoker = Arc::new(greet_task_invoker);

    Ok(greet_task_invoker)
}

async fn setup_tool_invoker(mcp_config_path: String) -> anyhow::Result<Arc<dyn ToolInvoker>> {
    let mcp_tool_invoker = McpRuntimeToolInvoker::new(mcp_config_path).await?;
    let mcp_tool_invoker = Arc::new(mcp_tool_invoker);

    Ok(mcp_tool_invoker)
}


async fn setup_agent_invoker_v2( discovery_service_adapter: Arc<dyn DiscoveryService>) -> anyhow::Result<Arc<dyn AgentInvoker>> {
    let a2a_agent_invoker = A2AAgentInvoker::new_with_discovery(None, None, discovery_service_adapter).await?;
    let a2a_agent_invoker = Arc::new(a2a_agent_invoker);

    Ok(a2a_agent_invoker)
}

/***********************************************************************************/
// End of Initialization of Invoker Services
/***********************************************************************************/


/***********************************************************************************/
// Registration Tasks and tools
/***********************************************************************************/

 
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


/// Register Tools in Discovery Service
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

/***********************************************************************************/
// End of Registration Tasks and Tools
/***********************************************************************************/


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

    /************************************************/
    /* Loading Factory Config File                  */
    /* Creating Agent Factory                       */
    /************************************************/ 
    let factory_config = FactoryConfig::load_factory_config(&args.config_file).expect("Incorrect Factory Config File");

    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(&factory_config.factory_evaluation_service_url.clone().expect("Factory Evaluation Service URL not set")).await;
    let memory_service = setup_memory_service(&factory_config.factory_memory_service_url.clone().expect("Factory Memory Service URL not set")).await;
    let discovery_service = setup_discovery_service(&factory_config.factory_discovery_url).await;
   
    /************************************************/
    /* Set Up Registrations via discovery service   */
    /* Only Tasks and Tools need to be registered   */
    /* Agents Self Register at Launch               */
    /************************************************/ 
    register_tasks(discovery_service.clone()).await?;
    register_tools(args.mcp_config_path.clone(),discovery_service.clone()).await?;

    /************************************************/
    /* Launch Agents from Factory                   */
    /************************************************/ 

    /************************************************/
    /* Set Up Invokers                               */
    /************************************************/ 
    let task_invoker= setup_task_invoker().await?;
    let tool_invoker = setup_tool_invoker(args.mcp_config_path.clone()).await?;
    let agent_invoker= setup_agent_invoker_v2(discovery_service.clone()).await?;

    /************************************************/
    /* Get a Workflow Invokers Instance           */
    /************************************************/ 
    let workflow_invokers = WorkFlowInvokers::init(
        task_invoker.clone(),
        agent_invoker.clone(),
        tool_invoker.clone(),
    ).await?;

    let workflow_invokers: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_invokers));

    /************************************************/
    /* Launch Agent Factory                         */
    /************************************************/ 

    // Launch Agent Factory
    let agent_factory=AgentFactory::new(factory_config.clone(),
                    discovery_service.clone(),
                            memory_service,
                                evaluation_service,
                                    workflow_invokers);

    /************************************************/
    /* Set Up Registrations via discovery service           */
    /************************************************/ 
    //register_tasks(agent_factory.factory_discovery_service.clone()).await?;
    //register_tools(args.mcp_config_path.clone(),agent_factory.factory_discovery_service.clone()).await?;

    /************************************************/
    /* Launch Planner and Executor Agents from Factory             */
    /************************************************/ 

    let mut handles = vec![];
    
    let agent_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");

        /************************************************/
        /* Launch Domain Agent with MCP                 */
        /************************************************/ 


    let factory_mcp_runtime_config = FactoryMcpRuntimeConfig::builder()
        .with_factory_mcp_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_mcp_llm_provider_api_key(agent_api_key.clone())
        .with_factory_mcp_llm_model_id("openai/gpt-oss-20b".to_string())
        .with_factory_mcp_server_url("http://localhost:8000/sse".to_string())
        .with_factory_mcp_server_api_key("".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryMcpRuntimeConfig: {}", e))?;
    
    
    // Config for Specialist Agent
    let factory_agent_config = FactoryAgentConfig::builder()
        .with_factory_agent_url("http://127.0.0.1:8080".to_string())
        .with_factory_agent_type(AgentType::Specialist)
        .with_factory_agent_domains(AgentDomain::General)
        .with_factory_agent_name("Basic_Agent".to_string())
        .with_factory_agent_id("Basic_Agent".to_string())
        .with_factory_agent_description("An Agent that answer Basic Questions".to_string())
        .with_factory_agent_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_agent_llm_provider_api_key(agent_api_key.clone())
        .with_factory_agent_llm_model_id("openai/gpt-oss-20b".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryAgentConfig: {}", e))?;



    match agent_factory.launch_agent(&factory_agent_config, Some(&factory_mcp_runtime_config), AgentType::Specialist).await {
        Ok(handle) => {
            info!("Successfully launched Basic Agent");
            handles.push(handle);
        },
        Err(e) => error!("Failed to launch Basic Agent: {:?}", e),
    }
    
        /************************************************/
        /* Launch Planner and Executor                  */
        /************************************************/ 


    let agent_planner_api_key = env::var("LLM_PLANNER_API_KEY").expect("LLM_PLANNER_API_KEY must be set");

    // Config for Executor Agent
    let factory_agent_config_executor = FactoryAgentConfig::builder()
        .with_factory_agent_url("http://127.0.0.1:9580".to_string())
        .with_factory_agent_type(AgentType::Executor)
        .with_factory_agent_domains(AgentDomain::General)
        .with_factory_agent_name("Executor_Agent".to_string())
        .with_factory_agent_id("Executor_Agent".to_string())
        .with_factory_agent_description("An Agent that executes workflows".to_string())
        .with_factory_agent_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_agent_llm_provider_api_key(agent_api_key.clone())
        .with_factory_agent_llm_model_id("openai/gpt-oss-20b".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryAgentConfig for Executor: {}", e))?;

    // Config for Planner Agent
    let factory_agent_config_planner = FactoryAgentConfig::builder()
        .with_factory_agent_url("http://127.0.0.1:9590".to_string())
        .with_factory_agent_type(AgentType::Planner)
        .with_factory_agent_domains(AgentDomain::General)
        .with_factory_agent_name("Planner_Agent".to_string())
        .with_factory_agent_id("Planner_Agent".to_string())
        .with_factory_agent_description("An Agent that plans workflows".to_string())
        .with_factory_agent_llm_provider_url(LlmProviderUrl::Groq)
        .with_factory_agent_llm_provider_api_key(agent_planner_api_key)
        .with_factory_agent_llm_model_id("openai/gpt-oss-20b".to_string())
        .with_factory_agent_executor_url("http://127.0.0.1:9580".to_string())
        .build().map_err(|e| anyhow::anyhow!("Failed to build FactoryAgentConfig for Planner: {}", e))?;

    // Launch Executor Agent
    match agent_factory.launch_agent(&factory_agent_config_executor, None, AgentType::Executor).await {
        Ok(handle) => {
            info!("Successfully launched Executor Agent");
            handles.push(handle);
        },
        Err(e) => error!("Failed to launch Executor Agent: {:?}", e),
    }

    println!("\n");

    // Launch Planner Agent
    match agent_factory.launch_agent(&factory_agent_config_planner, None, AgentType::Planner).await {
        Ok(handle) => {
            info!("Successfully launched Planner Agent");
            handles.push(handle);
        },
        Err(e) => error!("Failed to launch Planner Agent: {:?}", e),
    }

    println!("\n");

        /************************************************/
        /* Join Handles                                 */
        /************************************************/ 

    
    // Wait for all agents to complete
    let results = join_all(handles).await;

    for result in results {
        match result {
            Ok(Ok(_)) => (), // Agent finished successfully
            Ok(Err(e)) => error!("Agent task failed: {:?}", e),
            Err(e) => error!("Agent task panicked: {:?}", e),
        }
    }

    /************************************************/
    /* End Launch Agents from Factory               */
    /************************************************/ 


    Ok(())
}
