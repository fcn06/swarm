mod tasks;
mod tools;
mod agents;

use std::sync::Arc;
use tokio;
use clap::Parser;
use tracing::{  info, warn};

use agent_core::business_logic::agent::Agent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService,WorkflowServiceApi};

use crate::tasks::example_tasks::{FarewellTask, GreetTask};
use crate::tools::example_tools::WeatherApiTool;
use crate::tools::mcp_tool_runner::McpToolRunner;
use crate::agents::a2a_agent_runner::A2AAgentRunner;

use configuration::{setup_logging, AgentReference,AgentConfig};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;

use workflow_management::agent_communication::{agent_registry::AgentRegistry};
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;

use workflow_agent::business_logic::workflow_agent::WorkFlowAgent;
use workflow_agent::business_logic::workflow_registries::WorkFlowRegistries;
use workflow_agent::business_logic::service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter};
use workflow_agent::business_logic::service_adapters::AgentDiscoveryServiceAdapter;


/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_workflow_config.toml")]
    config_file: String,
    /// Workflow graph file path
    #[clap(long, default_value = "./workflow_management/example_workflow/multi_agent_workflow.json")]
    graph_file: String,
    /// Log level
    #[clap(long, default_value = "warn")]
    log_level: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
}

async fn setup_task_registry() -> Arc<TaskRegistry> {
    let mut task_registry = TaskRegistry::new();
    task_registry.register(Arc::new(GreetTask));
    task_registry.register(Arc::new(FarewellTask));
    Arc::new(task_registry)
}

async fn setup_tool_registry(mcp_config_path: String) -> Result<Arc<ToolRegistry>, Box<dyn std::error::Error>> {
    let mut tool_registry = ToolRegistry::new();
    // Injected tool
    tool_registry.register(Arc::new(WeatherApiTool));

    // Dynamically defined tools via MCP
    let mcp_agent = McpToolRunner::initialize_mcp_agent(mcp_config_path).await?;
    let mcp_tool_runner = McpToolRunner::new(mcp_agent.expect("No MCP Defined"), "general_mcp_server".to_string());
    tool_registry.register(Arc::new(mcp_tool_runner));

    Ok(Arc::new(tool_registry))
}

async fn setup_agent_registry(discovery_url: String) -> Result<Arc<AgentRegistry>, Box<dyn std::error::Error>> {
    let mut agent_registry = AgentRegistry::new();
     
    // Register a runner for "Basic_Agent"
    // can be extended to orchestration agent as well
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(discovery_url.clone()));
    let basic_agent_runner = A2AAgentRunner::new(
        vec![AgentReference {
            name: "Basic_Agent".to_string(),
            url: "http://127.0.0.1:8080".to_string(),
            is_default: Some(true),
        }],
        None,
        None,
        discovery_client.clone(),
    )
    .await?;
    agent_registry.register_with_name("Basic_Agent".to_string(), Arc::new(basic_agent_runner));
    
    Ok(Arc::new(agent_registry))
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
    let client = AgentDiscoveryServiceClient::new(discovery_url.clone());
    let adapter = AgentDiscoveryServiceAdapter::new(client);
    Some(Arc::new(adapter))
}

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
    /* Set Up Registries                            */
    /************************************************/ 
    let task_registry = setup_task_registry().await;
    let tool_registry = setup_tool_registry(args.mcp_config_path).await?;
    let agent_registry = setup_agent_registry(workflow_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")).await?;


    /************************************************/
    /* Get a Workflow Registries Instance           */
    /************************************************/ 
    let workflow_registries = WorkFlowRegistries::init(
        task_registry.clone(),
        agent_registry.clone(),
        tool_registry.clone(),
    ).await?;

    let workflow_registries: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_registries));


    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(&workflow_agent_config).await;
    let memory_service = setup_memory_service(&workflow_agent_config).await;
    let discovery_service = setup_discovery_service(workflow_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")).await;

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
     let agent = WorkFlowAgent::new(workflow_agent_config.clone(), evaluation_service, memory_service, discovery_service.clone(), workflow_registries).await?;

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