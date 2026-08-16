use std::sync::Arc;
use clap::Parser;
use tracing::info;

use agent_core::server::gateway_server::{GatewayBackend, GatewayServer, MultiModelGatewayBackend};
use agent_core::session::SessionStore;
use configuration::setup_logging;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Swarm Open Responses & Stateful Gateway Server")]
struct Args {
    /// Bind address (e.g. 0.0.0.0:8080)
    #[clap(long, default_value = "0.0.0.0:8080")]
    bind_address: String,

    /// Log level
    #[clap(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging(&args.log_level);

    info!("Starting Swarm Gateway Server on {}", args.bind_address);

    let session_store = Arc::new(SessionStore::new());
    let backend: Arc<dyn GatewayBackend> = Arc::new(MultiModelGatewayBackend::from_env());

    let gateway_server = GatewayServer::new(session_store, backend);

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       🌐 fcn06/swarm Multi-Agent Gateway Server Running       ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ • Open Responses Route:      POST http://{}/v1/responses      ║", args.bind_address);
    println!("║ • Chat Completions Route:    POST http://{}/v1/chat/completions║", args.bind_address);
    println!("║ • In-Memory Session Storage: Active                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    gateway_server.start(&args.bind_address).await?;

    Ok(())
}
