mod tasks;
mod tools;
mod agents;

use clap::Parser;
use std::sync::Arc;
use tracing::{ info, warn};

//use configuration::{setup_logging, AgentReference,AgentConfig};
use configuration::{setup_logging, AgentConfig};

use serde_json::json;

//use crate::tasks::tasks_invoker::{GreetTask};
use crate::tools::mcp_runtime_tool_invoker::McpRuntimeToolInvoker;
//use crate::agents::a2a_agent_invoker::A2AAgentInvoker;

use planner_agent::business_logic::planner_agent::PlannerAgent;
use planner_agent::business_logic::workflow_registry::WorkFlowRegistry;


//use workflow_management::agent_communication::agent_runner::AgentRunner;
use workflow_management::agent_communication::agent_registry::AgentDefinition;
//use workflow_management::tasks::task_runner::TaskRunner;
use workflow_management::tasks::task_registry::TaskDefinition;
//use workflow_management::tools::tool_runner::ToolRunner;
use workflow_management::tools::tool_registry::ToolDefinition;

use workflow_management::agent_communication::{agent_registry::AgentRegistry,};
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;


use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;


use agent_core::business_logic::agent::Agent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService,WorkflowServiceApi};




use agent_core::business_logic::service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_planner_config.toml")]
    config_file: String,
    /// Log level
    #[clap(long, default_value = "warn")]
    log_level: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
}

async fn setup_evaluation_service(workflow_agent_config: &AgentConfig) -> Option<Arc<dyn EvaluationService>> {
    if let Some(url) = workflow_agent_config.agent_evaluation_service_url() {
        info!("Evaluation service configured at: {}", url);
        let client = AgentEvaluationServiceClient::new(url);
        let adapter = AgentEvaluationServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Evaluation service URL not configured. No evaluations will be logged.");
        None
    }
}

async fn setup_memory_service(workflow_agent_config: &AgentConfig) -> Option<Arc<dyn MemoryService>> {
    if let Some(url) = workflow_agent_config.agent_memory_service_url() {
        info!("Memory service configured at: {}", url);
        let client = AgentMemoryServiceClient::new(url);
        let adapter = AgentMemoryServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Memory service URL not configured. No memory will be used.");
        None
    }
}

async fn setup_discovery_service(discovery_url: String) -> Option<Arc<dyn DiscoveryService>> {
    info!("Discovery service configured at: {}", discovery_url);
    let client = AgentDiscoveryServiceClient::new(&discovery_url.clone());
    let adapter = AgentDiscoveryServiceAdapter::new(client);
    Some(Arc::new(adapter))
}

async fn setup_task_registry() -> anyhow::Result<Arc<TaskRegistry>> {


    let mut task_registry = TaskRegistry::new();
    task_registry.register_task(TaskDefinition {
        id: "greeting".to_string(),
        name: "Say Hello".to_string(),
        description: "Say hello to somebody".to_string(),
        input_schema: json!({}),
        output_schema: json!({}),
    });

    Ok(Arc::new(task_registry))
}

async fn setup_tool_registry(mcp_config_path: String) -> anyhow::Result<Arc<ToolRegistry>> {
    let mcp_tool_runner_invoker = McpRuntimeToolInvoker::new(mcp_config_path).await?;
    let mcp_tool_runner_invoker = Arc::new(mcp_tool_runner_invoker);

    // Register tools
        let mut tool_registry = ToolRegistry::new();
        let list_tools= mcp_tool_runner_invoker.get_tools_list_v2().await?;
        for tool in list_tools {
            let tool_definition=ToolDefinition {
                id:tool.function.name.clone(),
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                input_schema: serde_json::Value::String(serde_json::to_string(&tool.function.parameters).unwrap_or_else(|_| "{}".to_string())),
                output_schema: json!({}),
        };    
        tool_registry.register_tool(tool_definition);
        }
    // End tools Registration

    Ok(Arc::new(tool_registry))
}

async fn setup_agent_registry(workflow_agent_config: &AgentConfig) -> anyhow::Result<Arc<AgentRegistry>> {

    let mut agent_registry = AgentRegistry::new();
    agent_registry.register_agent(AgentDefinition {
        id: "Basic_Agent".to_string(),
        name: "Basic Agent for weather requests, customer requests and other general topics".to_string(),
        description: "Retrieve Weather in a Location, Get customer details and other General Requests".to_string(),
        skills: Vec::new(),
    });

    Ok(Arc::new(agent_registry))
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 
    // load a2a config file and initialize appropriateruntime
    let planner_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect WorkFlow Agent config file");


    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(&planner_agent_config).await;
    let memory_service = setup_memory_service(&planner_agent_config).await;
    let discovery_service = setup_discovery_service(planner_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")).await;


    /************************************************/
    /* Set Up Runners                               */
    /************************************************/ 
    let task_registry = setup_task_registry().await?;
    let tool_registry = setup_tool_registry(args.mcp_config_path).await?;
    let agent_registry = setup_agent_registry(&planner_agent_config).await?;

    /************************************************/
    /* Get a Workflow Registries Instance           */
    /************************************************/ 
    let workflow_registry = WorkFlowRegistry::init(
        task_registry.clone(),
        agent_registry.clone(),
        tool_registry.clone(),
    ).await?;

   // debug!("{}",workflow_runners.list_available_resources());

    let workflow_registry: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_registry));

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
    let agent = PlannerAgent::new(planner_agent_config.clone(), evaluation_service, memory_service, discovery_service.clone(), workflow_registry).await?;

    /************************************************/
    /* Launch Workflow Agent Server                 */
    /************************************************/ 
    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<PlannerAgent>::new(planner_agent_config, agent, discovery_service).await?;
   
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* Agent server launched                        */
    /* Responding to any A2A CLient                 */
    /************************************************/ 


    Ok(())
}
