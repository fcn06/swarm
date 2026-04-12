use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager};

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
use common::search_mcp_service::SearchMcpService;

use clap::{Parser, Subcommand};
use std::sync::Arc;

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
    /// Launches the server with the search tool
    Search,
    /// Launches the server with all tools
    All,
}

/// Helper: create an axum Router from a StreamableHttpService
fn make_app<F, S>(factory: F, session_manager: Arc<LocalSessionManager>, config: StreamableHttpServerConfig) -> axum::Router
where
    F: Fn() -> Result<S, std::io::Error> + Send + Sync + Clone + 'static,
    S: rmcp::ServerHandler + Send + 'static,
{
    let service = StreamableHttpService::new(factory, session_manager, config);
    axum::Router::new()
        .route("/mcp", axum::routing::any({
            let svc = service.clone();
            move |req: axum::extract::Request| {
                let svc = svc.clone();
                async move { svc.handle(req).await }
            }
        }))
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

    let sse_config = StreamableHttpServerConfig::default();
    let session_manager = Arc::new(LocalSessionManager::default());

    let app = match args.command {
        Commands::Weather => make_app(|| Ok(WeatherMcpService::new()), session_manager, sse_config),
        Commands::Customer => make_app(|| Ok(CustomerMcpService::new()), session_manager, sse_config),
        Commands::Scrape => make_app(|| Ok(ScrapeMcpService::new()), session_manager, sse_config),
        Commands::Search => make_app(|| Ok(SearchMcpService::new()), session_manager, sse_config),
        Commands::All => make_app(|| Ok(GeneralMcpService::new()), session_manager, sse_config),
    };

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    
    /************************************************/
    /*  Server launched                             */
    /************************************************/ 

    axum::serve(listener, app).await?;
    
    Ok(())
}