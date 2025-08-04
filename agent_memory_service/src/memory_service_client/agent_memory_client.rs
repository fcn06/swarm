
use reqwest::{Client, Error};
use serde::{Serialize, Deserialize};

use crate::{AgentData, MemoryEntry};

#[derive(Debug)]
pub struct AgentServiceClient {
    memory_service_url: String,
    client: Client,
}

impl AgentServiceClient {
    pub fn new(memory_service_url: String) -> Self {
        AgentServiceClient {
            memory_service_url,
            client: Client::new(),
        }
    }

    pub async fn push_content_to_memory_service(&self, agent_data: AgentData) -> Result<MemoryEntry, Error> {
        let url = format!("{}/memory", self.memory_service_url);
        let response = self.client.post(&url)
            .json(&agent_data)
            .send()
            .await?;

        response.json::<MemoryEntry>().await
    }

    pub async fn retrieve_content_from_memory_service(&self) -> Result<Vec<MemoryEntry>, Error> {
        let url = format!("{}/memory", self.memory_service_url);
        let response = self.client.get(&url)
            .send()
            .await?;

        response.json::<Vec<MemoryEntry>>().await
    }
}