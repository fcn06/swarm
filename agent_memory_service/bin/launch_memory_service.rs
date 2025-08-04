use std::sync::Arc;


use tracing::{info, error, Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use clap::Parser;

use serde::{Serialize,Deserialize};
use chrono::Utc;

use agent_memory_service::memory_server::server::MemoryServer;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "warn")]
    log_level: String,
    #[clap(long, default_value = "0.0.0.0:5000")]
    uri: String,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

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
    /* Launch Memory Server                         */
    /************************************************/ 
    let memory_server=MemoryServer::new(args.uri).await?;
    memory_server.start_http().await?;

    /************************************************/
    /* End Launch Memory Server                     */
    /************************************************/ 

        Ok(())

}

