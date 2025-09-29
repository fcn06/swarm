use reqwest::Client;
use anyhow::Result;

//use crate::models::{LogEntry, LogPayload, Role};
use agent_models::memory::memory_models::{LogEntry, LogPayload, Role};


#[derive(Debug, Clone)]
pub struct AgentMemoryServiceClient {
    memory_service_url: String,
    client: Client,
}

impl AgentMemoryServiceClient {
    pub fn new(memory_service_url: String) -> Self {
        AgentMemoryServiceClient {
            memory_service_url,
            client: Client::new(),
        }
    }

    pub async fn log(&self, conversation_id: String, role: Role, content: String, agent_id: Option<String>) -> Result<Vec<LogEntry>> {
        let url = format!("{}/log", self.memory_service_url);
        let payload = LogPayload {
            conversation_id,
            role,
            content,
            agent_id,
        };

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;

        Ok(response.json::<Vec<LogEntry>>().await?)
    }

    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<Vec<LogEntry>>> {
        let url = format!("{}/conversation/{}", self.memory_service_url, conversation_id);
        let response = self.client.get(&url)
            .send()
            .await?;

        Ok(response.json::<Option<Vec<LogEntry>>>().await?)
    }
}