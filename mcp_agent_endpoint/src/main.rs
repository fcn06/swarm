// main.rs

mod api;

use log::{error, info}; // Use log for logging

use crate::api::endpoint::run_endpoint;
use configuration::AgentMcpConfig;
use mcp_agent_backbone::mcp_initialization::mcp_agent_config::RuntimeMcpConfigProject;

/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub project_config: RuntimeMcpConfigProject,
    pub mcp_agent_config: AgentMcpConfig, // Use Arc for shared ownership
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (e.g., using tracing or env_logger)
     tracing_subscriber::fmt::init(); // Example using tracing_subscriber
    //env_logger::init(); // Example using env_logger

    info!("Starting MCP Agent...");

    // Load Agent Configuration from config file
    let config_path = "configuration/agent_mcp_config.toml"; // to be inserted via command line
    info!("Loading agent configuration from {}...", config_path);
    let mcp_agent_config = match AgentMcpConfig::load_agent_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "Failed to load agent configuration from {}: {}. Exiting.",
                config_path, e
            );
            return Err(e.into());
        }
    };

    info!("Agent configuration loaded successfully.");

    // Setup Runtime Project Configuration
    let project_config =
        match mcp_agent_backbone::mcp_initialization::mcp_agent_config::setup_project_mcp(
            mcp_agent_config.clone(),
        )
        .await
        {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to setup project configuration: {}. Exiting.", e);
                return Err(e.into());
            }
        };
    info!("Project configuration setup successfully.");

    // Create AppState
    let app_state = AppState {
        project_config,
        mcp_agent_config, // Move the Arc<AgentConfig> into state
    };

    // Run the endpoint, passing the combined state
    info!("Starting API endpoint...");
    let _ = run_endpoint(app_state).await; // Pass AppState to run_endpoint

    info!("MCP Agent finished.");

    Ok(())
}
