use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_evaluation_service::evaluation_server::judge_agent::AgentLogData;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;
use agent_memory_service::models::Role;
use agent_core::business_logic::services::{EvaluationService, MemoryService};

// Adapter for AgentEvaluationServiceClient
pub struct AgentEvaluationServiceAdapter {
    client: AgentEvaluationServiceClient,
}

impl AgentEvaluationServiceAdapter {
    pub fn new(client: AgentEvaluationServiceClient) -> Self {
        AgentEvaluationServiceAdapter { client }
    }
}

#[async_trait]
impl EvaluationService for AgentEvaluationServiceAdapter {
    async fn log_evaluation(&self, data: AgentLogData) -> Result<()> {
        self.client.log_evaluation(data).await.map(|_| ())
    }
}

// Adapter for AgentMemoryServiceClient
pub struct AgentMemoryServiceAdapter {
    client: AgentMemoryServiceClient,
}

impl AgentMemoryServiceAdapter {
    pub fn new(client: AgentMemoryServiceClient) -> Self {
        AgentMemoryServiceAdapter { client }
    }
}

#[async_trait]
impl MemoryService for AgentMemoryServiceAdapter {
    async fn log(&self, conversation_id: String, role: Role, text: String, agent_name: Option<String>) -> Result<()> {
        self.client.log(conversation_id, role, text, agent_name).await.map(|_| ())
    }
}
