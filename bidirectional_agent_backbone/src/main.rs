use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AgentInfo {
    id: String,
    address: String,
    skills: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let agent_id = "agent_1".to_string();
    let agent_address = "http://127.0.0.1:8080".to_string(); // This agent's address
    let agent_skills = vec!["math".to_string(), "weather".to_string()];

    let agent_info = AgentInfo {
        id: agent_id.clone(),
        address: agent_address,
        skills: agent_skills,
    };

    info!("Attempting to register agent: {:?}", agent_info);

    // Register with discovery service
    let client = reqwest::Client::new();
    let discovery_service_url = "http://127.0.0.1:3030/register";
    match client.post(discovery_service_url).json(&agent_info).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("Agent {} successfully registered.", agent_id);
            } else {
                eprintln!("Failed to register agent {}: {:?}", agent_id, response.text().await);
            }
        }
        Err(e) => {
            eprintln!("Error registering agent {}: {}", agent_id, e);
        }
    }

    // Agent's main loop (placeholder)
    info!("Agent {} started and running...", agent_id);
    // In a real scenario, this would involve listening for incoming requests,
    // processing them, and potentially making requests to other agents.
    tokio::signal::ctrl_c().await?;
    info!("Agent {} received shutdown signal.", agent_id);

    Ok(())
}
