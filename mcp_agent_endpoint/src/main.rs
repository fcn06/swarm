// main.rs
mod api;

use tracing::{info, error, Level, debug};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use crate::api::endpoint::run_endpoint;
use configuration::AgentMcpConfig;
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;

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

     let subscriber = Registry::default()
        .with(
            // stdout layer, to view everything in the console
            fmt::layer()
                .compact()
                .with_ansi(true)
                //.with_filter(filter::LevelFilter::from_level(Level::TRACE))
                .with_filter(filter::LevelFilter::from_level(Level::ERROR))
        );
    
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Starting MCP Agent...");

    // Load Agent Configuration from config file
    let config_path = "configuration/agent_mcp_config.toml"; // to be inserted via command line
    info!("Loading agent configuration from {}...", config_path);
    let agent_mcp_config = match AgentMcpConfig::load_agent_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "Failed to load agent configuration from {}: {}. Exiting.",
                config_path, e
            );
            return Err(e.into());
        }
    };


    let mut mcp_agent = McpAgent::new(agent_mcp_config.clone()).await?;

    // Create AppState
    let app_state = AppState {
        mcp_agent,
        agent_mcp_config, // Keep the original config
    };

    // Run the endpoint, passing the combined state
    info!("Starting API endpoint...");
    let _ = run_endpoint(app_state).await; // Pass AppState to run_endpoint

    info!("MCP Agent finished.");

    Ok(())
}
