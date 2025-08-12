use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};
use clap::Parser;
use agent_evaluation_service::evaluation_server::server::EvaluationServer;

/// Command-line arguments for the evaluation server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "warn")]
    log_level: String,
    #[clap(long, default_value = "0.0.0.0:4001")]
    uri: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

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
            fmt::layer()
                .compact()
                .with_ansi(true)
                .with_filter(filter::LevelFilter::from_level(log_level))
        );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let evaluation_server = EvaluationServer::new(args.uri).await?;
    evaluation_server.start_http().await?;

    Ok(())
}