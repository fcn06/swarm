use configuration::AgentConfig;
use basic_agent::business_logic::basic_agent::BasicAgent;
use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::agent::Agent;


use clap::Parser;
use std::env;

use configuration::setup_logging;

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_basic_config.toml")]
    config_file: String,
    #[clap(long, default_value = "warn")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Parse command-line arguments
    let args = Args::parse();

    /************************************************/
    /* Setting proper log level                     */
    /************************************************/ 
    setup_logging(&args.log_level);

    /************************************************/
    /* End of Setting proper log level              */
    /************************************************/ 

    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 

    // load a2a config file and initialize appropriateruntime
    let basic_agent_config = AgentConfig::load_agent_config(&args.config_file).expect("Incorrect Basic Agent config file");
  
    let agent_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");

    let agent = BasicAgent::new(basic_agent_config.clone(),agent_api_key, None,None, None,None,None).await?;

    // Create the modern server, and pass the runtime elements
    let server = AgentServer::<BasicAgent>::new(basic_agent_config, agent,None).await?;

    println!("🌐 Starting HTTP server only...");
    server.start_http().await?;

    /************************************************/
    /* A2A agent server launched                    */
    /* Responding to any A2A CLient                 */
    /************************************************/ 

    Ok(())
}
