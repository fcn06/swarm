use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};
use clap::Parser;
use agent_evaluation_service::evaluation_server::server::EvaluationServer;
use configuration::AgentConfig;
use std::env;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "0.0.0.0:7000")]
    uri: String,
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_judge_config.toml")]
    config_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
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
    /* Launch Evaluation Server                         */
    /************************************************/ 
    let agent_config = AgentConfig::load_agent_config(&args.config_file).expect("REASON");
    let agent_api_key = env::var("LLM_JUDGE_API_KEY").expect("LLM_JUDGE_API_KEY must be set");

    let evaluation_server = EvaluationServer::new(args.uri,agent_config,agent_api_key).await?;
    evaluation_server.start_http().await?;

    Ok(())
}