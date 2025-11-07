use resource_invoker::McpRuntimeToolInvoker as McpRuntimeTools;
use std::env;

use clap::Parser;
use std::sync::Arc;
use tracing::{ info};

use configuration::{setup_logging, AgentConfig};

use serde_json::json;

use planner_agent::business_logic::planner_agent::PlannerAgent;

// Registration via discovery service
use agent_models::registry::registry_models::{TaskDefinition,AgentDefinition,ToolDefinition};

use agent_core::business_logic::agent::Agent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};

use agent_service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter,AgentDiscoveryServiceAdapter};

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
    #[clap(long, default_value = "http://127.0.0.1:4000")]
    discovery_service_url: String,
    #[clap(long, default_value = "http://127.0.0.1:5000")]
    memory_service_url: String,
    #[clap(long, default_value = "http://127.0.0.1:7000")]
    evaluation_service_url: String,
}

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
        name: "Basic Agent".to_string(),
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
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 
    // load a2a config file and initialize appropriateruntime
    let planner_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect WorkFlow Agent config file");
    let agent_api_key = env::var("LLM_PLANNER_API_KEY").expect("LLM_PLANNER_API_KEY must be set");

    /************************************************/
    /* Instantiate Memory, Evaluation and Discovery Services  */
    /************************************************/ 
    let evaluation_service = setup_evaluation_service(args.evaluation_service_url).await;
    let memory_service = setup_memory_service(args.memory_service_url).await;
    let discovery_service = setup_discovery_service(args.discovery_service_url).await;


    /************************************************/
    /* Set Up Registrations via discovery service           */
    /************************************************/ 
    register_tasks(discovery_service.clone().unwrap()).await?;
    register_agents(discovery_service.clone().unwrap()).await?;
    register_tools(args.mcp_config_path.clone(),discovery_service.clone().unwrap()).await?;

    /************************************************/
    /* Launch Workflow Agent                        */
    /************************************************/ 
    let agent = PlannerAgent::new(planner_agent_config.clone(),agent_api_key,None,None, evaluation_service, memory_service, discovery_service.clone(), None).await?;
    
    /************************************************/
    /* Launch Workflow Agent Server                 */
    /************************************************/ 
    // Create the modern server, and pass the runtime elements
    // todo:enable a way to avoid evaluation if not needed
    let server = AgentServer::<PlannerAgent>::new(planner_agent_config, agent, discovery_service).await?;
   
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* Agent server launched                        */
    /* Responding to any A2A CLient                 */
    /************************************************/ 


    Ok(())
}
