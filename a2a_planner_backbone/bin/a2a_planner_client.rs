use a2a_planner_backbone::a2a_agent_logic::planner_agent::PlannerAgent;

use configuration::AgentPlannerConfig;

//use a2a_rs::domain::{Message};

use clap::Parser;

use tracing::{ Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};


/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Host to bind the server to
    #[clap(long, default_value = "How are you today ?")]
    user_query: String,
    /// Configuration file path (TOML format)
    #[clap(long, default_value = "configuration/agent_planner_config.toml")]
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
    /* Loading Planner Config File and launching    */
    /* A planner agent                              */
    /************************************************/ 
 
    // load a2a config file and initialize appropriateruntime
    let agent_planner_config = AgentPlannerConfig::load_agent_config(&args.config_file)
        .expect("No planner configuration file");

    // Initialize the Planner Agent
    let mut planner_agent = PlannerAgent::new(agent_planner_config).await?;

    /************************************************/
    /* Planner agent launched                       */
    /************************************************/ 


    /************************************************/
    /* Using it to resolve query passed in cmd line */
    /************************************************/ 
    // --- Test Case 1 ---
    /* 
    let message_id_1 = uuid::Uuid::new_v4().to_string();
    let user_req_1 = Message::user_text(args.user_query, message_id_1);
    //println!("--- Sending User Request 1 ---");
    let result_1 = planner_agent.handle_user_request(user_req_1).await;
    */

    let result_1=planner_agent.submit_user_text(args.user_query.clone()).await;

    println!("Output:{:?}", result_1.output);

    Ok(())
}
