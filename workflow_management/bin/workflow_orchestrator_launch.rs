use clap::Parser;
use std::sync::Arc;
use tracing::{error, info, warn};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use configuration::{setup_logging, AgentReference};


use serde_json::json;

mod tasks;
mod tools;
mod agents;


use crate::tasks::tasks_invoker::{GreetTask};
use crate::tools::mcp_runtime_tool_invoker::McpRuntimeToolInvoker;
//use crate::agents::a2a_agent_interaction::A2AAgentInteraction;
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

use workflow_management::graph::config::load_graph_from_file;
use workflow_management::graph::graph_orchestrator::PlanExecutor;


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
    /// User Query
    #[clap(long, default_value = "Prepare a personalized email wishing a good day for company with customer_id 12345. You will mention the weather in the email.")]
    user_query: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logging(&args.log_level);

    // TASKS
    let greet_task_invoker = GreetTask::new()?;
    let greet_task_invoker=Arc::new(greet_task_invoker);

    let mut task_registry = TaskRegistry::new();

    task_registry.register_task(TaskDefinition {
        id: "greeting".to_string(),
        name: "Say Hello".to_string(),
        description: "Say hello to somebody".to_string(),
        input_schema: json!({}),
        output_schema:json!({}),
    });
    let task_registry = Arc::new(task_registry);

    let task_runner=Arc::new(TaskRunner::new(task_registry,greet_task_invoker));


    // TOOLS   
    // invoker
    let mcp_tool_runner_invoker = McpRuntimeToolInvoker::new(args.mcp_config_path).await?;
    let mcp_tool_runner_invoker=Arc::new(mcp_tool_runner_invoker);

    // registry
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register_tool(ToolDefinition {
        id: "get_current_weather".to_string(),
        name: "Retrieve Weather in a Location".to_string(),
        description: "Retrieves weather in a specific location".to_string(),
        input_schema: json!({}),
        output_schema:json!({}),
    });
    let tool_registry = Arc::new(tool_registry);
    // runner
    let tool_runner=Arc::new(ToolRunner::new(tool_registry,mcp_tool_runner_invoker));
    

    // AGENTS
    // invoker
    // additional services ( discovery, evaluation, memory)
    let discovery_client = Arc::new(AgentDiscoveryServiceClient::new(args.discovery_url));

    let a2a_agent_invoker = A2AAgentInvoker::new(vec![AgentReference {
        name: "Basic_Agent".to_string(),
        url: "http://127.0.0.1:8080".to_string(),
        is_default: Some(true),
    }], None, None, discovery_client.clone()).await?;
    let a2a_agent_invoker=Arc::new(a2a_agent_invoker);

    // registry
    let mut agent_registry = AgentRegistry::new();

    agent_registry.register_agent(AgentDefinition {
        id: "Basic_Agent".to_string(),
        name: "Basic Agent for weather requests, customer requests and other general topics".to_string(),
        description: "Retrieve Weather in a Location, Get customer details and other General Requests".to_string(),
        skills:Vec::new(),
    });
    let agent_registry=Arc::new(agent_registry);

    // runner
    let agent_runner= Arc::new(AgentRunner::new(agent_registry,a2a_agent_invoker));
    // todo : make it recursive to point execution of a workflow




    // LOAD AND EXECUTE
    // 4. Load workflow and execute
    let workflow_file = &args.graph_file;
    info!("Loading workflow from: {}", workflow_file);

    match load_graph_from_file(workflow_file) {
        Ok(graph) => {
            info!("Workflow loaded successfully. Plan: {}", graph.plan_name);
            info!("Workflow loaded successfully. Plan: {:?}", graph);
            let mut executor =
                PlanExecutor::new(graph, task_runner, agent_runner, tool_runner, args.user_query);
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


    /* 
    let weather_query_params = serde_json::json!({
            "_location": "Boston"
        }
    );
    let response=tool_runner.run("get_current_weather".to_string(), weather_query_params).await?;
    println!("Here is the response : {}",response);
    */