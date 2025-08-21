mod tasks;
mod tools;
mod agents;

use crate::tasks::example_tasks::{FarewellTask, GreetTask};
use crate::tools::example_tools::WeatherApiTool;
use crate::tools::mcp_tool_runner::McpToolRunner;

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use crate::agents::a2a_agent_runner::A2AAgentRunner;

use configuration::{setup_logging, AgentReference};


use workflow_management::agent_communication::{agent_registry::AgentRegistry};

use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;


use std::sync::Arc;
use tokio;

use workflow_agent::business_logic::workflow_agent::WorkFlowAgent;
use workflow_agent::business_logic::workflow_registries::WorkFlowRegistries;

use agent_core::business_logic::agent::Agent;

use agent_core::server::agent_server::AgentServer;

use clap::Parser;

use tracing::{  info, warn};

use agent_core::business_logic::services::{EvaluationService, MemoryService};
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;

use configuration::{ AgentConfig};
use workflow_agent::business_logic::service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter};

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
    #[clap(long, default_value = "info")]
    log_level: String,
    /// Discovery service URL
    #[clap(long, default_value = "http://localhost:5000")]
    discovery_url: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

     /************************************************/
    /* Set Up Registries                            */
    /************************************************/ 

    /************************************************/
    /* Task   Registries                            */
    /************************************************/ 
    let mut task_registry = TaskRegistry::new();
    task_registry.register(Arc::new(GreetTask));
    task_registry.register(Arc::new(FarewellTask));
    let task_registry = Arc::new(task_registry);

    /************************************************/
    /* Tool   Registries                            */
    /************************************************/ 
    let mut tool_registry = ToolRegistry::new();
    // Injected tool
    tool_registry.register(Arc::new(WeatherApiTool));

    // Dynamically defined tools via MCP
    let mcp_agent = McpToolRunner::initialize_mcp_agent(args.mcp_config_path).await?;
    let mcp_tool_runner = McpToolRunner::new(mcp_agent.expect("No MCP Defined"), "general_api".to_string());
    tool_registry.register(Arc::new(mcp_tool_runner));

    let tool_registry = Arc::new(tool_registry);

    /************************************************/
    /* Agent   Registries                            */
    /************************************************/ 

   let mut agent_registry = AgentRegistry::new();
     
    // Register a runner for "Basic_Agent"
    // can be extended to orchestration agent as well
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(args.discovery_url));
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
    
    let agent_registry = Arc::new(agent_registry);

     /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 

    // load a2a config file and initialize appropriateruntime
    let workflow_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect WorkFlow Agent config file");

    /************************************************/
    /* Instantiate Memory abnd Evaluation Services  */
    /************************************************/ 
 
    // Initialize evaluation service client if configured
    let evaluation_service: Option<Arc<dyn EvaluationService>> = if let Some(url) = workflow_agent_config.agent_evaluation_service_url() {
        info!("Evaluation service configured at: {}", url);
        let client = AgentEvaluationServiceClient::new(url);
        let adapter = AgentEvaluationServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Evaluation service URL not configured. No evaluations will be logged.");
        None
    };

    // Initialize memory service client if configured
    let memory_service: Option<Arc<dyn MemoryService>> = if let Some(url) = workflow_agent_config.agent_memory_service_url() {
        info!("Memory service configured at: {}", url);
        let client = AgentMemoryServiceClient::new(url);
        let adapter = AgentMemoryServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Memory service URL not configured. No memory will be used.");
        None
    };

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
    let workflow_registries = WorkFlowRegistries::init(
        task_registry.clone(),
        agent_registry.clone(),
        tool_registry.clone(),
    ).await?;

    let agent = WorkFlowAgent::new(workflow_agent_config.clone(), evaluation_service, memory_service,Some(Arc::new(workflow_registries))).await?;

    /************************************************/
    /* Launch Workflow Agent Server                 */
    /************************************************/ 
    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<WorkFlowAgent>::new(workflow_agent_config, agent).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* Agent server launched                        */
    /* Responding to any A2A CLient                 */
    /************************************************/ 
    
    Ok(())

}