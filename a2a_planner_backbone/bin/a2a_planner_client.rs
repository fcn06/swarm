use a2a_planner_backbone::a2a_agent_logic::planner_agent::PlannerAgent;
use a2a_planner_backbone::PlannerAgentDefinition;


use configuration::AgentPlannerConfig;

use a2a_rs::domain::{Message};

use dotenv::dotenv;

use clap::Parser;

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
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenv().ok();

    // Parse command-line arguments
    let args = Args::parse();

    // load a2a config file and initialize appropriateruntime
    let agent_planner_config = AgentPlannerConfig::load_agent_config(&args.config_file)
        .expect("No planner configuration file");

    // Initialize the Planner Agent
    let mut planner_agent = PlannerAgent::new(agent_planner_config).await?;

    // --- Test Case 1 ---
    let message_id_1 = uuid::Uuid::new_v4().to_string();
    let user_req_1 = Message::user_text(args.user_query, message_id_1);

    //println!("--- Sending User Request 1 ---");
    let result_1 = planner_agent.handle_user_request(user_req_1).await;
    
    //println!("--- Final Execution Result 1 ---");
    //println!("Request ID: {}", result_1.request_id);
    //println!("Success: {}", result_1.success);
    println!("Output:{:?}", result_1.output);

    Ok(())
}
