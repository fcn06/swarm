use agent_core::business_logic::agent::Agent;
use configuration::AgentConfig;
use orchestration_agent::business_logic::orchestration_agent::OrchestrationAgent;
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
use orchestration_agent::business_logic::service_adapters::{AgentEvaluationServiceAdapter, AgentMemoryServiceAdapter};


/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_orchestration_config.toml")]
    config_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Parse command-line arguments
    let args = Args::parse();

    /************************************************/
    /* Setting proper log level. Default is INFO    */
    /************************************************/ 
    let log_level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::WARN,
    };

    let subscriber = Registry::default()
    .with(
        // stdout layer, to view everything in the console
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(log_level))
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    /************************************************/
    /* End of Setting proper log level              */
    /************************************************/ 

    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 

    // load a2a config file and initialize appropriateruntime
    let orchestration_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect Orchestration Agent config file");
  
    // Initialize evaluation service client if configured
    let evaluation_service: Option<Arc<dyn EvaluationService>> = if let Some(url) = orchestration_agent_config.agent_evaluation_service_url() {
        info!("Evaluation service configured at: {}", url);
        let client = AgentEvaluationServiceClient::new(url);
        let adapter = AgentEvaluationServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Evaluation service URL not configured. No evaluations will be logged.");
        None
    };

    // Initialize memory service client if configured
    let memory_service: Option<Arc<dyn MemoryService>> = if let Some(url) = orchestration_agent_config.agent_memory_service_url() {
        info!("Memory service configured at: {}", url);
        let client = AgentMemoryServiceClient::new(url);
        let adapter = AgentMemoryServiceAdapter::new(client);
        Some(Arc::new(adapter))
    } else {
        warn!("Memory service URL not configured. No memory will be used.");
        None
    };

    let agent = OrchestrationAgent::new(orchestration_agent_config.clone(), evaluation_service, memory_service).await?;


    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<OrchestrationAgent>::new(orchestration_agent_config, agent).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* A2A agent server launched                    */
    /* Responding to any A2A CLient                 */
    /************************************************/ 

    Ok(())
}
