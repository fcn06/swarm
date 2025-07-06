use configuration::AgentPlannerConfig;
use a2a_planner_backbone::a2a_planner_server::planner_server::SimplePlannerAgentServer;

use clap::Parser;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_planner_config.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>>  {
    // Initialize logging with better formatting
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    // load a2a config file and initialize appropriateruntime
    let agent_planner_config = AgentPlannerConfig::load_agent_config(&args.config_file);
    let server=SimplePlannerAgentServer::new(agent_planner_config.expect("REASON")).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    Ok(())
}