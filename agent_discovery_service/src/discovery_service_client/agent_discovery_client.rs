
use reqwest::{Client, Error};
use serde::{Serialize, Deserialize};


use a2a_rs::domain::AgentCard;

#[derive(Debug)]
pub struct AgentDiscoveryServiceClient {
    discovery_service_url: String,
    client: Client,
}

impl AgentDiscoveryServiceClient {
    pub fn new(memory_service_url: String) -> Self {
        AgentDiscoveryServiceClient {
            discovery_service_url,
            client: Client::new(),
        }
    }

    pub async fn push_content_to_discovery_service(&self, agent_card: AgentCard) -> Result<AgentCard, Error> {
        let url = format!("{}/register", self.discovery_service_url);
        let response = self.client.post(&url)
            .json(&agent_card)
            .send()
            .await?;

        response.json::<AgentCard>().await
    }

    pub async fn retrieve_content_from_discovery_service(&self) -> Result<Vec<AgentCard>, Error> {
        let url = format!("{}/agents", self.discovery_service_url);
        let response = self.client.get(&url)
            .send()
            .await?;

        response.json::<Vec<AgentCard>>().await
    }
}