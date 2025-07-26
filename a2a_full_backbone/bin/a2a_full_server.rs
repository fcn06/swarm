use configuration::AgentFullConfig;
use a2a_full_backbone::a2a_full_server::full_server::FullAgentServer;

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
async fn main() ->  Result<(), Box<dyn std::error::Error>>  {

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
    /* Loading A2A Full Server Config File and      */
    /* launching A2A full agent server              */
    /************************************************/ 


    // load a2a config file and initialize appropriateruntime
    let agent_planner_config = AgentFullConfig::load_agent_config(&args.config_file);
    let server=FullAgentServer::new(agent_planner_config.expect("REASON")).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* A2A full agent server launched               */
    /* Responding to any A2A CLient                 */
    /************************************************/ 

    Ok(())
}