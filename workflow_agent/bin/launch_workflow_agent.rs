//mod tasks;
//mod tools;
//mod agents;

//use crate::tasks::tasks_invoker::{GreetTask};
//use crate::tools::mcp_runtime_tool_invoker::McpRuntimeToolInvoker;
//use crate::agents::a2a_agent_invoker::A2AAgentInvoker;

use resource_invoker::McpRuntimeToolInvoker;
use resource_invoker::GreetTask;
use resource_invoker::A2AAgentInvoker;

use clap::Parser;
use std::sync::Arc;
use tracing::{ info,warn};

use configuration::{setup_logging, AgentReference,AgentConfig};

use serde_json::json;



use agent_models::registry::registry_models::{TaskDefinition,AgentDefinition,ToolDefinition};

use workflow_management::agent_communication::agent_invoker::AgentInvoker;
use workflow_management::tasks::task_invoker::TaskInvoker;
use workflow_management::tools::tool_invoker::ToolInvoker;


use agent_core::business_logic::agent::Agent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService,WorkflowServiceApi};


use workflow_agent::business_logic::workflow_agent::WorkFlowAgent;
use workflow_agent::business_logic::workflow_invokers::WorkFlowInvokers;

use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};



/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_workflow_config.toml")]
    config_file: String,
    /// Log level
    #[clap(long, default_value = "warn")]
    log_level: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
}

/********************************************************************/
// Instantiation of Memory, Evaluation, Discovery Services
/********************************************************************/

async fn setup_evaluation_service(workflow_agent_config: &AgentConfig) -> Option<Arc<dyn EvaluationService>> {
    if let Some(url) = workflow_agent_config.agent_evaluation_service_url() {
        info!("Evaluation service configured at: {}", url);
        let adapter = AgentEvaluationServiceAdapter::new(&url);
        Some(Arc::new(adapter))
    } else {
        warn!("Evaluation service URL not configured. No evaluations will be logged.");
        None
    }
}

async fn setup_memory_service(workflow_agent_config: &AgentConfig) -> Option<Arc<dyn MemoryService>> {
    if let Some(url) = workflow_agent_config.agent_memory_service_url() {
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


/********************************************************************/
// Registration via discovery Service of the resources 
// that we will make available
/********************************************************************/

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

    let agent_definition=AgentDefinition {
        id: "Basic_Agent".to_string(),
        name: "Basic Agent for weather requests, customer requests and other general topics".to_string(),
        description: "Retrieve Weather in a Location, Get customer details and other General Requests".to_string(),
        skills: Vec::new(),
    };

    discovery_service.register_agent(&agent_definition).await?;
    Ok(())
}

/// Register Agents in Discovery Service
async fn register_tools(mcp_config_path: String,discovery_service: Arc<dyn DiscoveryService>) -> anyhow::Result<()> {

    let mcp_tools = McpRuntimeToolInvoker::new(mcp_config_path).await?;
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

/********************************************************************/
// Invokers
/********************************************************************/

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

async fn setup_agent_invoker(executor_agent_config: &AgentConfig) -> anyhow::Result<Arc<dyn AgentInvoker>> {
    let discovery_service_adapter = Arc::new(AgentDiscoveryServiceAdapter::new(
        &executor_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")
    ));

    let a2a_agent_invoker = A2AAgentInvoker::new(vec![AgentReference {
        name: "Basic_Agent".to_string(),
        url: "http://127.0.0.1:8080".to_string(),
        is_default: Some(true),
    }], None, None, discovery_service_adapter.clone()).await?;
    let a2a_agent_invoker = Arc::new(a2a_agent_invoker);

    Ok(a2a_agent_invoker)
}

/********************************************************************/
// Main
/********************************************************************/

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 
    // load a2a config file and initialize appropriateruntime
    let workflow_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect WorkFlow Agent config file");


    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(&workflow_agent_config).await;
    let memory_service = setup_memory_service(&workflow_agent_config).await;
    let discovery_service = setup_discovery_service(workflow_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")).await;


    /************************************************/
    /* Set Up Registrations via discovery service           */
    /************************************************/ 
    register_tasks(discovery_service.clone().unwrap()).await?;
    register_agents(discovery_service.clone().unwrap()).await?;
    register_tools(args.mcp_config_path.clone(),discovery_service.clone().unwrap()).await?;

    
    /************************************************/
    /* Set Up Invokers                               */
    /************************************************/ 
    let task_invoker= setup_task_invoker().await?;
    let tool_invoker = setup_tool_invoker(args.mcp_config_path).await?;
    let agent_invoker= setup_agent_invoker(&workflow_agent_config).await?;

    /************************************************/
    /* Get a Workflow Invokers Instance           */
    /************************************************/ 
    let workflow_invokers = WorkFlowInvokers::init(
        task_invoker.clone(),
        agent_invoker.clone(),
        tool_invoker.clone(),
    ).await?;

    //debug!("{}",workflow_invokers.list_available_resources());

    let workflow_invokers: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_invokers));

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
     let agent = WorkFlowAgent::new(workflow_agent_config.clone(), evaluation_service, memory_service, discovery_service.clone(), workflow_invokers).await?;

    /************************************************/
    /* Launch Workflow Agent Server                 */
    /************************************************/ 
    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<WorkFlowAgent>::new(workflow_agent_config, agent, discovery_service).await?;
   
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* Agent server launched                        */
    /* Responding to any A2A CLient                 */
    /************************************************/ 
    
    Ok(())

}