use clap::Parser;
use std::sync::Arc;
use tracing::{error, info, warn};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use configuration::{setup_logging, AgentReference};
use workflow_management::agent_communication::{
    a2a_agent_runner::A2AAgentRunner, agent_registry::AgentRegistry,
};
use workflow_management::graph::config::load_graph_from_file;
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::tasks::example_tasks::{FarewellTask, GreetTask};
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::example_tools::WeatherApiTool;
use workflow_management::tools::tool_registry::ToolRegistry;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Workflow graph file path
    #[clap(long, default_value = "./workflow_management/example_workflow/mix_workflow.json")]
    graph_file: String,
    /// Log level
    #[clap(long, default_value = "info")]
    log_level: String,
    /// Discovery service URL
    #[clap(long, default_value = "http://localhost:8000")]
    discovery_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logging(&args.log_level);

    // 1. Create and populate the TaskRegistry
    let mut task_registry = TaskRegistry::new();
    task_registry.register(Arc::new(GreetTask));
    task_registry.register(Arc::new(FarewellTask));
    let task_registry = Arc::new(task_registry);

    // 2. Create and populate the ToolRegistry
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(Arc::new(WeatherApiTool));
    let tool_registry = Arc::new(tool_registry);

    // 3. Setup Agent Communication
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(args.discovery_url));
    let mut agent_registry = AgentRegistry::new();

    // Register a runner for "Basic_Agent"
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

    // 4. Load workflow and execute
    let workflow_file = &args.graph_file;
    info!("Loading workflow from: {}", workflow_file);

    match load_graph_from_file(workflow_file) {
        Ok(graph) => {
            info!("Workflow loaded successfully. Plan: {}", graph.id);
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
