use reqwest::{Client, Error};
use a2a_rs::domain::AgentCard;
use anyhow::Result;


/// A client for interacting with the Agent Discovery Service.
#[derive(Debug)]
pub struct AgentDiscoveryServiceClient {
    discovery_service_url: String,
    client: Client,
}

impl AgentDiscoveryServiceClient {
    /// Creates a new client for the given discovery service URL.
    pub fn new(discovery_service_url: &str) -> Self {
        AgentDiscoveryServiceClient {
            discovery_service_url: discovery_service_url.to_string(),
            client: Client::new(),
        }
    }

    /// Registers an agent with the discovery service.
    pub async fn register(&self, agent_card: &AgentCard) -> Result<String, Error> {
        let url = format!("{}/register", self.discovery_service_url);
        let response = self.client.post(&url).json(agent_card).send().await?;
        response.text().await
    }
    
    /// Deregisters an agent from the discovery service.
    pub async fn deregister(&self, agent_card: &AgentCard) -> Result<String, Error> {
        let url = format!("{}/deregister", self.discovery_service_url);
        let response = self.client.post(&url).json(agent_card).send().await?;
        response.text().await
    }

    /// Lists all registered agents.
    pub async fn list_agents(&self) -> Result<Vec<AgentCard>, Error> {
        let url = format!("{}/agents", self.discovery_service_url);
        let response = self.client.get(&url).send().await?;
        response.json::<Vec<AgentCard>>().await
    }

    /// Searches for agents that have a specific skill.
    pub async fn search_by_skill(&self, skill: &str) -> Result<Vec<AgentCard>, Error> {
        let url = format!("{}/agents/search", self.discovery_service_url);
        let response = self.client.get(&url).query(&[("skill", skill)]).send().await?;
        response.json::<Vec<AgentCard>>().await
    }

    /// Lists all agents except for the one with the specified name.
    /// This is useful for preventing an agent from discovering itself.
    pub async fn list_other_agents(&self, agent_name_to_filter_out: &str) -> Result<Vec<AgentCard>, Error> {
        let all_agents = self.list_agents().await?;
        let filtered_agents = all_agents
            .into_iter()
            .filter(|agent| agent.name != agent_name_to_filter_out)
            .collect();
        Ok(filtered_agents)
    }
}
