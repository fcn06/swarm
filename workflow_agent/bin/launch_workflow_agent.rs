mod tasks;
mod tools;
mod agents;

use clap::Parser;
use std::sync::Arc;
use tracing::{ info, debug,warn};

use configuration::{setup_logging, AgentReference,AgentConfig};

use serde_json::json;

use crate::tasks::tasks_invoker::{GreetTask};
use crate::tools::mcp_runtime_tool_invoker::McpRuntimeToolInvoker;
use crate::agents::a2a_agent_invoker::A2AAgentInvoker;


use workflow_management::agent_communication::agent_runner::AgentRunner;
use workflow_management::agent_communication::agent_registry::AgentDefinition;
use workflow_management::tasks::task_runner::TaskRunner;
use workflow_management::tasks::task_registry::TaskDefinition;
use workflow_management::tools::tool_runner::ToolRunner;
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


use workflow_agent::business_logic::workflow_agent::WorkFlowAgent;
use workflow_agent::business_logic::workflow_runners::WorkFlowRunners;

use workflow_agent::business_logic::service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter};
use workflow_agent::business_logic::service_adapters::AgentDiscoveryServiceAdapter;



/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_workflow_config_v2.toml")]
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
    let client = AgentDiscoveryServiceClient::new(discovery_url.clone());
    let adapter = AgentDiscoveryServiceAdapter::new(client);
    Some(Arc::new(adapter))
}

async fn setup_task_runner() -> anyhow::Result<Arc<TaskRunner>> {
    let greet_task_invoker = GreetTask::new()?;
    let greet_task_invoker = Arc::new(greet_task_invoker);

    let mut task_registry = TaskRegistry::new();
    task_registry.register_task(TaskDefinition {
        id: "greeting".to_string(),
        name: "Say Hello".to_string(),
        description: "Say hello to somebody".to_string(),
        input_schema: json!({}),
        output_schema: json!({}),
    });
    let task_registry = Arc::new(task_registry);

    Ok(Arc::new(TaskRunner::new(task_registry, greet_task_invoker)))
}

async fn setup_tool_runner(mcp_config_path: String) -> anyhow::Result<Arc<ToolRunner>> {
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
        let tool_registry = Arc::new(tool_registry);
    // End tools Registration

    Ok(Arc::new(ToolRunner::new(tool_registry, mcp_tool_runner_invoker)))
}

async fn setup_agent_runner(workflow_agent_config: &AgentConfig) -> anyhow::Result<Arc<AgentRunner>> {
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(
        workflow_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")
    ));
    let a2a_agent_invoker = A2AAgentInvoker::new(vec![AgentReference {
        name: "Basic_Agent".to_string(),
        url: "http://127.0.0.1:8080".to_string(),
        is_default: Some(true),
    }], None, None, discovery_client.clone()).await?;
    let a2a_agent_invoker = Arc::new(a2a_agent_invoker);

    let mut agent_registry = AgentRegistry::new();
    agent_registry.register_agent(AgentDefinition {
        id: "Basic_Agent".to_string(),
        name: "Basic Agent for weather requests, customer requests and other general topics".to_string(),
        description: "Retrieve Weather in a Location, Get customer details and other General Requests".to_string(),
        skills: Vec::new(),
    });
    let agent_registry = Arc::new(agent_registry);

    Ok(Arc::new(AgentRunner::new(agent_registry, a2a_agent_invoker)))
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
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(&workflow_agent_config).await;
    let memory_service = setup_memory_service(&workflow_agent_config).await;
    let discovery_service = setup_discovery_service(workflow_agent_config.agent_discovery_url.clone().expect("Discovery URL not configured")).await;


    /************************************************/
    /* Set Up Runners                               */
    /************************************************/ 
    let task_runner = setup_task_runner().await?;
    let tool_runner = setup_tool_runner(args.mcp_config_path).await?;
    let agent_runner = setup_agent_runner(&workflow_agent_config).await?;

    /************************************************/
    /* Get a Workflow Registries Instance           */
    /************************************************/ 
    let workflow_runners = WorkFlowRunners::init(
        task_runner.clone(),
        agent_runner.clone(),
        tool_runner.clone(),
    ).await?;

    debug!("{}",workflow_runners.list_available_resources());

    let workflow_runners: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_runners));

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
     let agent = WorkFlowAgent::new(workflow_agent_config.clone(), evaluation_service, memory_service, discovery_service.clone(), workflow_runners).await?;

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