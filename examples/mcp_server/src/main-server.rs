use rmcp::transport::sse_server::SseServer;
use tracing_subscriber::{
    layer::SubscriberExt,
    {self},
};

use tracing::{Level};
use tracing_subscriber::{
    fmt,
    layer::Layer,
    Registry, filter
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
    #[clap(long, default_value = "warn")]
    log_level: String,
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
    /* Defining on each port to listen to           */
    /************************************************/ 

    let bind_address = format!("{}:{}", args.host, args.port);
    println!("MCP Server listening on: {} with command: {:?}", bind_address, args.command);

    /************************************************/
    /*  Defining which tools to enable , and launch */
    /************************************************/ 

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
    
    /************************************************/
    /*  Server launched                             */
    /************************************************/ 

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}