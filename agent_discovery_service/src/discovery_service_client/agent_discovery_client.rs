use reqwest::{Client, Error};
use anyhow::Result;
//use crate::model::models::{AgentDefinition, TaskDefinition, ToolDefinition};
use agent_models::registry::registry_models::{AgentDefinition, TaskDefinition, ToolDefinition};


/*
agent_discovery_client.rs 
is concerned with how to communicate with the discovery service over the network.

agent_discovery_client.rs: 
This file contains the concrete client implementation for interacting directly 
with the Agent Discovery Service's HTTP API. 
It handles the low-level details of making network requests (using reqwest) to register, deregister, 
list, and search for agents, tasks, and tools. It knows the specific endpoints and data formats of the discovery service.
*/

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

    // Agent Definition methods

    /// Registers an agent definition with the discovery service.
    pub async fn register_agent_definition(&self, agent_def: &AgentDefinition) -> Result<String, Error> {
        let url = format!("{}/agents/register", self.discovery_service_url);
        let response = self.client.post(&url).json(agent_def).send().await?;
        response.text().await
    }
    
    /// Deregisters an agent definition from the discovery service.
    pub async fn deregister_agent_definition(&self, agent_def: &AgentDefinition) -> Result<String, Error> {
        let url = format!("{}/agents/deregister", self.discovery_service_url);
        let response = self.client.post(&url).json(agent_def).send().await?;
        response.text().await
    }

    /// Lists all registered agent definitions.
    pub async fn list_agent_definitions(&self) -> Result<Vec<AgentDefinition>, Error> {
        let url = format!("{}/agents", self.discovery_service_url);
        let response = self.client.get(&url).send().await?;
        response.json::<Vec<AgentDefinition>>().await
    }

    /// Searches for agents that have a specific skill.
    /// The skill is provided as a query parameter.
    pub async fn search_agents_by_skill(&self, skill: &str) -> Result<Vec<AgentDefinition>, Error> {
        let url = format!("{}/agents/search", self.discovery_service_url);
        let response = self.client.get(&url).query(&[("skill", skill)]).send().await?;
        response.json::<Vec<AgentDefinition>>().await
    }

    /// Lists all agents except for the one with the specified ID.
    /// This is useful for preventing an agent from discovering itself.
    pub async fn list_other_agents_definitions(&self, agent_id_to_filter_out: &str) -> Result<Vec<AgentDefinition>, Error> {
        let all_agents = self.list_agent_definitions().await?;
        let filtered_agents = all_agents
            .into_iter()
            .filter(|agent| agent.id != agent_id_to_filter_out)
            .collect();
        Ok(filtered_agents)
    }

    // Task Definition methods

    /// Registers a task definition with the discovery service.
    pub async fn register_task_definition(&self, task_def: &TaskDefinition) -> Result<String, Error> {
        let url = format!("{}/tasks/register", self.discovery_service_url);
        let response = self.client.post(&url).json(task_def).send().await?;
        response.text().await
    }

    /// Lists all registered task definitions.
    pub async fn list_task_definitions(&self) -> Result<Vec<TaskDefinition>, Error> {
        let url = format!("{}/tasks", self.discovery_service_url);
        let response = self.client.get(&url).send().await?;
        response.json::<Vec<TaskDefinition>>().await
    }

    // Tool Definition methods

    /// Registers a tool definition with the discovery service.
    pub async fn register_tool_definition(&self, tool_def: &ToolDefinition) -> Result<String, Error> {
        let url = format!("{}/tools/register", self.discovery_service_url);
        let response = self.client.post(&url).json(tool_def).send().await?;
        response.text().await
    }

    /// Lists all registered tool definitions.
    pub async fn list_tool_definitions(&self) -> Result<Vec<ToolDefinition>, Error> {
        let url = format!("{}/tools", self.discovery_service_url);
        let response = self.client.get(&url).send().await?;
        response.json::<Vec<ToolDefinition>>().await
    }
}
