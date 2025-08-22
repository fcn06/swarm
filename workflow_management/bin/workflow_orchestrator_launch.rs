use clap::Parser;
use std::sync::Arc;
use tracing::{error, info, warn};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use configuration::{setup_logging, AgentReference};

use workflow_management::agent_communication::{
   // a2a_agent_runner::A2AAgentRunner, 
    agent_registry::AgentRegistry,
};
use workflow_management::graph::config::load_graph_from_file;
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;

// use mcp_runtime::mcp_agent_logic::agent::McpAgent;
//use workflow_management::tools::mcp_tool_runner::McpToolRunner;


mod tasks;
mod tools;
mod agents;

use crate::tasks::example_tasks::{FarewellTask, GreetTask};
use crate::tools::example_tools::WeatherApiTool;
use crate::tools::mcp_tool_runner::McpToolRunner;
use crate::agents::a2a_agent_runner::A2AAgentRunner;


/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Workflow graph file path
    #[clap(long, default_value = "./workflow_management/example_workflow/multi_agent_workflow.json")]
    graph_file: String,
    /// Log level
    #[clap(long, default_value = "warn")]
    log_level: String,
    /// Discovery service URL
    #[clap(long, default_value = "http://localhost:5000")]
    discovery_url: String,
    /// MCP Config
    #[clap(long, default_value = "./configuration/mcp_runtime_config.toml")]
    mcp_config_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logging(&args.log_level);

    // TASKS
    // This is where you may inject a LLM management tasks
    // 1. Create and populate the TaskRegistry
    let mut task_registry = TaskRegistry::new();
    task_registry.register(Arc::new(GreetTask));
    task_registry.register(Arc::new(FarewellTask));
    let task_registry = Arc::new(task_registry);

    // TOOLS
    // This is where you may inject a MCP Runner Implementation
    // 2. Create and populate the ToolRegistry
    let mut tool_registry = ToolRegistry::new();
    // Injected tool
    tool_registry.register(Arc::new(WeatherApiTool));

    // Dynamically defined tools via MCP
    let mcp_agent = McpToolRunner::initialize_mcp_agent(args.mcp_config_path).await?;
    let mcp_tool_runner = McpToolRunner::new(mcp_agent.expect("No MCP Defined"), "general_api".to_string());
    tool_registry.register(Arc::new(mcp_tool_runner));

    let tool_registry = Arc::new(tool_registry);

    // AGENTS
    // This is where you may inject an A2A Agent Runner implementation
    // 3. Create and populate the AgentRegistry
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(args.discovery_url));
    let mut agent_registry = AgentRegistry::new();

     
    // Register a runner for "Basic_Agent"
    // can be extended to orchestration agent as well
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


    // todo : make it recursive to point execution of a workflow
    


    // LOAD AND EXECUTE
    // 4. Load workflow and execute
    let workflow_file = &args.graph_file;
    info!("Loading workflow from: {}", workflow_file);

    match load_graph_from_file(workflow_file) {
        Ok(graph) => {
            info!("Workflow loaded successfully. Plan: {}", graph.plan_name);
            //info!("Workflow loaded successfully. Plan: {:?}", graph);
            let mut executor =
                PlanExecutor::new(graph, task_registry, agent_registry, tool_registry);
            if let Err(e) = executor.execute_plan().await {
                warn!("Error executing plan: {}", e);
            }
        }
        Err(e) => {
            error!("Error loading workflow: {}", e);
        }
    }
    
    Ok(())
}
