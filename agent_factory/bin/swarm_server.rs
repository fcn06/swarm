use std::sync::Arc;
use clap::Parser;
use tracing::info;

use agent_core::server::gateway_server::{
    GatewayBackend, GatewayConfigFile, GatewayServer, MultiModelGatewayBackend,
};
use agent_core::session::SessionStore;
use configuration::setup_logging;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Swarm Open Responses & Stateful Gateway Server")]
struct Args {
    /// Path to gateway configuration file (e.g. gateway_kickstart/config_files/gateway_config.toml)
    #[clap(long, short = 'c')]
    config_file: Option<String>,

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

    let mut bind_address = args.bind_address.clone();
    let mut log_level = args.log_level.clone();

    let backend: Arc<dyn GatewayBackend> = if let Some(config_path) = &args.config_file {
        match std::fs::read_to_string(config_path) {
            Ok(content) => match toml::from_str::<GatewayConfigFile>(&content) {
                Ok(config) => {
                    if let Some(server) = &config.server {
                        if let Some(addr) = &server.bind_address {
                            if args.bind_address == "0.0.0.0:8080" {
                                bind_address = addr.clone();
                            }
                        }
                        if let Some(level) = &server.log_level {
                            if args.log_level == "info" {
                                log_level = level.clone();
                            }
                        }
                    }
                    println!("✔ Loaded Gateway Configuration from: {}", config_path);
                    Arc::new(MultiModelGatewayBackend::from_config(&config))
                }
                Err(err) => {
                    eprintln!("⚠️ Failed to parse config file {}: {}. Using env defaults.", config_path, err);
                    Arc::new(MultiModelGatewayBackend::from_env())
                }
            },
            Err(err) => {
                eprintln!("⚠️ Failed to read config file {}: {}. Using env defaults.", config_path, err);
                Arc::new(MultiModelGatewayBackend::from_env())
            }
        }
    } else {
        Arc::new(MultiModelGatewayBackend::from_env())
    };

    setup_logging(&log_level);
    info!("Starting Swarm Gateway Server on {}", bind_address);

    let session_store = Arc::new(SessionStore::new());
    let gateway_server = GatewayServer::new(session_store, backend);

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       🌐 fcn06/swarm Multi-Agent Gateway Server Running       ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ • Open Responses Route:      POST http://{}/v1/responses      ║", bind_address);
    println!("║ • Chat Completions Route:    POST http://{}/v1/chat/completions║", bind_address);
    println!("║ • In-Memory Session Storage: Active                            ║");
    if let Some(cfg) = &args.config_file {
        println!("║ • Config File:               {:<33} ║", cfg);
    }
    println!("╚════════════════════════════════════════════════════════════════╝");

    gateway_server.start(&bind_address).await?;

    Ok(())
}
