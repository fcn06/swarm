use rmcp::transport::sse_server::SseServer;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
mod common;
use common::customer_mcp_service::CustomerMcpService;
use common::general_mcp_service::GeneralMcpService;
use common::scrape_mcp_service::ScrapeMcpService;
use common::weather_mcp_service::WeatherMcpService;

use clap::{Parser, Subcommand};

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Host to bind the server to
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "8000")]
    port: String,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launches the server with the weather tool
    Weather,
    /// Launches the server with the customer tool
    Customer,
    /// Launches the server with the scrape tool
    Scrape,
    /// Launches the server with all tools
    All,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    let bind_address = format!("{}:{}", args.host, args.port);
    println!("Server listening on: {}", bind_address);

    let ct = match args.command {
        Commands::Weather => SseServer::serve(bind_address.parse()?)
            .await?
            .with_service(WeatherMcpService::new),
        Commands::Customer => SseServer::serve(bind_address.parse()?)
            .await?
            .with_service(CustomerMcpService::new),
        Commands::Scrape => SseServer::serve(bind_address.parse()?)
            .await?
            .with_service(ScrapeMcpService::new),
        Commands::All => SseServer::serve(bind_address.parse()?)
            .await?
            .with_service(GeneralMcpService::new),
    };

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}