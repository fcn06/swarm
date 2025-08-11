use orchestration_agent::config::orchestration_agent_config::OrchestrationAgentConfig;
use orchestration_agent::business_logic::orchestration_agent::OrchestrationAgent;
use agent_protocol_backbone::server::agent_server::AgentServer;

use clap::Parser;

use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_full_config.toml")]
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
    let orchestration_agent_config = OrchestrationAgentConfig::load_agent_config(&args.config_file);
  
    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<OrchestrationAgent,OrchestrationAgentConfig>::new(orchestration_agent_config.expect("Incorrect Orchestration Agent config file")).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* A2A agent server launched                    */
    /* Responding to any A2A CLient                 */
    /************************************************/ 

    Ok(())
}
