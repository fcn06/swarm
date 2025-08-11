// main.rs
mod api;

use tracing::{info, error, Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use crate::api::endpoint::run_endpoint;
use configuration::AgentMcpConfig;
use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use clap::Parser;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_mcp_config.toml")]
    config_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
}


/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub mcp_agent: McpAgent,
    pub agent_mcp_config: AgentMcpConfig, // Use Arc for shared ownership
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (e.g., using tracing or env_logger)
     //tracing_subscriber::fmt::init(); // Example using tracing_subscriber

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
    /* Loading MCP Config File and launching        */
    /* MCP Agent                                    */
    /************************************************/ 

    info!("Starting MCP Agent...");

    // load mcp config file and initialize appropriateruntime
    let agent_mcp_config = match AgentMcpConfig::load_agent_config(&args.config_file) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "Failed to load agent configuration from {}: {}. Exiting.",
                &args.config_file, e
            );
            return Err(e.into());
        }
    };

    let mcp_agent = McpAgent::new(agent_mcp_config.clone()).await?;

    /************************************************/
    /* MCP Agent Launched                           */
    /************************************************/ 

    /************************************************/
    /* Launch REST endpoint to connect to agent     */
    /************************************************/ 

    // Create AppState
    let app_state = AppState {
        mcp_agent,
        agent_mcp_config, // Keep the original config
    };

    // Run the endpoint, passing the combined state
    info!("Starting API endpoint...");
    let _ = run_endpoint(app_state).await; // Pass AppState to run_endpoint

    /************************************************/
    /* Endpoint launched                            */
    /************************************************/ 


    info!("MCP Agent finished.");

    Ok(())
}
