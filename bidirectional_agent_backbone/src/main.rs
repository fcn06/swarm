use anyhow::Result;
use configuration::AgentBidirectionalConfig;

use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use tracing_subscriber::FmtSubscriber;

mod a2a_agent_logic;
mod a2a_plan;

#[tokio::main]
async fn main() -> Result<()> {
    // set tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load agent configuration
    let agent_bidirectional_config = AgentBidirectionalConfig::load_agent_config("configuration/agent_bidirectional_config.toml")
        .expect("Error loading agent configuration");

    // Create and start the server
    let server = a2a_agent_logic::server::BidirectionalAgentServer::new(agent_bidirectional_config).await?;
    server.start_all().await?;

    Ok(())
}
