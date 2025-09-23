
use std::sync::Arc;
use tracing::{info, error};

use agent_core::server::agent_server::AgentServer;
use agent_core::business_logic::agent::Agent;

use planner_agent::business_logic::planner_agent::PlannerAgent;
use configuration::{setup_logging, AgentReference,AgentConfig};

//use agent_core::business_logic::services::BusinessServices;

#[tokio::main]
async fn main() -> anyhow::Result<()> {


    /************************************************/
    /* Loading A2A Config File and launching        */
    /* A2A agent server                             */
    /************************************************/ 
    // load a2a config file and initialize appropriateruntime
    let workflow_agent_config = AgentConfig::load_agent_config("configuration/agent_workflow_config.toml").expect("Incorrect WorkFlow Agent config file");


    /* 
    // TODO: Initialize the services properly for the planner agent
    let services = BusinessServices::new(
        None,
        None,
        None,
        None,
    ).await?;
    
    let agent = PlannerAgent::new(
        agent_config.clone(),
        Some(Arc::new(services.evaluation_service)),
        None,
        Some(Arc::new(services.discovery_service)),
        None,
    ).await?;

    let agent_server = AgentServer::new(
        agent_config.agent_name.clone(),
        agent_config.agent_host.clone(),
        agent_config.agent_port,
    );

    info!("Starting planner agent server at http://{}:{}", agent_config.agent_host, agent_config.agent_port);
    
    if let Err(e) = agent_server.run(agent).await {
        error!("Server error: {}", e);
    }
    */
    
    Ok(())
}
