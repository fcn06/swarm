
use resource_invoker::McpRuntimeToolInvoker;
use resource_invoker::GreetTask;
use resource_invoker::A2AAgentInvoker;


use clap::Parser;
use std::sync::Arc;
use tracing::{ info};

use configuration::{setup_logging,AgentConfig};

// not needed with v2
#[allow(unused_imports)]
use configuration::{AgentReference};



use executor_agent::business_logic::executor_agent::{ExecutorAgent, WorkFlowInvokers};

use workflow_management::agent_communication::agent_invoker::AgentInvoker;
use workflow_management::tasks::task_invoker::TaskInvoker;
use workflow_management::tools::tool_invoker::ToolInvoker;



use agent_core::business_logic::agent::Agent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService,WorkflowServiceApi};


use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_executor_config.toml")]
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

async fn setup_evaluation_service(evaluation_service_url:String) -> Option<Arc<dyn EvaluationService>> {
    info!("Evaluation service configured at: {}", evaluation_service_url);
    let adapter = AgentEvaluationServiceAdapter::new(&evaluation_service_url);
    Some(Arc::new(adapter))

}

async fn setup_memory_service(memory_service_url:String) -> Option<Arc<dyn MemoryService>> {
    info!("Memory service configured at: {}", memory_service_url);
    let adapter = AgentMemoryServiceAdapter::new(&memory_service_url);
    Some(Arc::new(adapter))
}

async fn setup_discovery_service(discovery_service_url: String) -> Option<Arc<dyn DiscoveryService>> {
info!("Discovery service configured at: {}", discovery_service_url);
let adapter = AgentDiscoveryServiceAdapter::new(&discovery_service_url);
Some(Arc::new(adapter))
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let args = Args::parse();
    setup_logging(&args.log_level);

    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 
    // load a2a config file and initialize appropriateruntime
    let executor_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect Executor Agent config file");
    let agent_api_key="".to_string();

    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(args.evaluation_service_url).await;
    let memory_service = setup_memory_service(args.memory_service_url).await;
    let discovery_service = setup_discovery_service(args.discovery_service_url).await;


    /************************************************/
    /* Set Up Invokers                               */
    /************************************************/ 
    let task_invoker= setup_task_invoker().await?;
    let tool_invoker = setup_tool_invoker(args.mcp_config_path).await?;
    let agent_invoker= setup_agent_invoker_v2(discovery_service.clone().expect("No Discovery Service")).await?;

    /************************************************/
    /* Get a Workflow Invokers Instance           */
    /************************************************/ 
    let workflow_invokers = WorkFlowInvokers::init(
        task_invoker.clone(),
        agent_invoker.clone(),
        tool_invoker.clone(),
    ).await?;

   // debug!("{}",workflow_invokers.list_available_resources());

    let workflow_invokers: Option<Arc<dyn WorkflowServiceApi>> = Some(Arc::new(workflow_invokers));

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
    let agent = ExecutorAgent::new(executor_agent_config.clone(),agent_api_key,None,None, evaluation_service, memory_service, discovery_service.clone(), workflow_invokers).await?;


    /************************************************/
    /* Launch Workflow Agent Server                 */
    /************************************************/ 
    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<ExecutorAgent>::new(executor_agent_config, agent, discovery_service).await?;
   
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* Agent server launched                        */
    /* Responding to any A2A CLient                 */
    /************************************************/ 


    Ok(())
}
