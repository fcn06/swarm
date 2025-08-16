use clap::Parser;
use std::sync::Arc;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{prelude::*, fmt, layer::Layer, Registry, filter};
use workflow_management::graph::config::load_graph_from_file;
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tasks::example_tasks::{GreetTask, FarewellTask};

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "./workflow_management/example_workflow/conditional_execution_example.json")]
    graph_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let log_level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::WARN,
    };

    let subscriber = Registry::default().with(
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(log_level)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    // 1. Create and populate the TaskRegistry
    let mut registry = TaskRegistry::new();
    registry.register(Arc::new(GreetTask));
    registry.register(Arc::new(FarewellTask));
    let registry = Arc::new(registry);

    let workflow_file = &args.graph_file;
    info!("Loading workflow from: {}", workflow_file);

    match load_graph_from_file(workflow_file) {
        Ok(graph) => {
            info!("Workflow loaded successfully. Plan: {}", graph.id);
            // 2. Inject the registry into the PlanExecutor
            let mut executor = PlanExecutor::new(graph, registry);
            if let Err(e) = executor.execute_plan().await {
                warn!("Error executing plan: {}", e);
            }
        }
        Err(e) => {
            error!("Error loading workflow: {}", e);
        }
    }
}
