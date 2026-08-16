// main.rs
mod api;

use tracing::{info,  Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use crate::api::endpoint::run_endpoint;

use clap::Parser;

use a2a_rs::{
    HttpClient,
};

use std::sync::Arc;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_basic_config.toml")]
    config_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
    #[clap(long, default_value = "0.0.0.0:3000")]
    uri: String,
    #[clap(long, default_value = "http://localhost:8080")]
    a2a_uri: String,
}


/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub a2a_client: Arc<HttpClient>,
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
    /* Connecting to the A2A Agent                  */
    /************************************************/ 

    let a2a_uri=args.a2a_uri;
    info!("A2A URI: {}", a2a_uri);
    let a2a_client = HttpClient::new(a2a_uri);

    /************************************************/
    /* Connected to A2A agent                       */
    /************************************************/ 

    /************************************************/
    /* Launch REST endpoint to connect to agent     */
    /************************************************/ 

    // Create AppState
    let app_state = AppState {
        a2a_client:Arc::new(a2a_client),
    };


    //let uri="0.0.0.0:3000".to_string();
    let uri=args.uri;
    info!("URI: {}", uri);

    // Run the endpoint, passing the combined state
    info!("Starting API endpoint...");
    let _ = run_endpoint(app_state,uri).await; // Pass AppState to run_endpoint

    /************************************************/
    /* Endpoint launched                            */
    /************************************************/ 


    info!("A2A Agent finished.");

    Ok(())
}
