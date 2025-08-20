use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio;


use mcp_runtime::mcp_client::McpClient;
use workflow_management::agents::agent_registry::AgentRegistry;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;

use agent_core::business_logic::agent::Agent;

use agent_core::server::agent_server::AgentServer;
use std::sync::Arc;

use clap::Parser;

use tracing::{ Level, info, warn};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use agent_core::business_logic::services::{EvaluationService, MemoryService};
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;

use configuration::{setup_logging, AgentConfig,AgentReference};

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
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
async fn main() -> anyhow::Result<()> {

    let args = Args::parse();
    setup_logging(&args.log_level);

    let mut task_registry = TaskRegistry::new();
    let mut tool_registry = ToolRegistry::new();
    let mut agent_registry = AgentRegistry::new();

    


}