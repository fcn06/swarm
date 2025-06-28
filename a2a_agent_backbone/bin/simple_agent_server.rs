use a2a_agent_backbone::a2a_agent_logic::config::AuthConfig;
use a2a_agent_backbone::a2a_agent_logic::config::ServerConfig;
use a2a_agent_backbone::a2a_agent_logic::config::StorageConfig;
use a2a_agent_backbone::a2a_agent_logic::server::SimpleAgentServer;

use a2a_agent_backbone::a2a_agent_initialization::a2a_agent_config::setup_project_a2a;
use configuration::AgentA2aConfig;

use clap::Parser;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_a2a_config.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with better formatting
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    // load a2a config file and initialize appropriateruntime
    let a2a_agent_config = AgentA2aConfig::load_agent_config(&args.config_file);
    let a2a_runtime_config_project =
        setup_project_a2a(a2a_agent_config.expect("Could not load A2a Config file")).await?;

    // Create the modern server, and pass the runtime elements
    // runtime config is passed from server then to task handler
    let server = SimpleAgentServer::new(a2a_runtime_config_project);
    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    Ok(())
}
