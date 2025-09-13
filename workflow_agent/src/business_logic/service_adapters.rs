use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_evaluation_service::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;
use agent_memory_service::models::Role;
use agent_core::business_logic::services::{EvaluationService, MemoryService};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use agent_core::business_logic::services::DiscoveryService;
use a2a_rs::domain::AgentCard;


/********************************************/
/* Service Adapter for Evaluation Service   */
/********************************************/

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
    async fn log_evaluation(&self, data: AgentEvaluationLogData) -> Result<JudgeEvaluation> {
        self.client.log_evaluation(data).await
    }
}

/********************************************/
/* Service Adapter for Memory Service       */
/********************************************/

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

/********************************************/
/* Service Adapter for Discovery Service    */
/********************************************/

pub struct AgentDiscoveryServiceAdapter {
    client: AgentDiscoveryServiceClient,
}

impl AgentDiscoveryServiceAdapter {
    pub fn new(client: AgentDiscoveryServiceClient) -> Self {
        AgentDiscoveryServiceAdapter { client }
    }
}

#[async_trait]
impl DiscoveryService for AgentDiscoveryServiceAdapter {
    async fn register_agent(&self, agent_card: &AgentCard) -> Result<()> {
        self.client.register(agent_card).await?;
        Ok(())
    }

    async fn unregister_agent(&self, agent_card: &AgentCard) -> Result<()> {
                self.client.deregister(agent_card).await?;
        Ok(())
    }

    async fn get_agent_address(&self, agent_name: String) -> Result<Option<String>> {
        let all_agents = self.client.list_agents().await?;
        Ok(all_agents
            .into_iter()
            .find(|agent| agent.name == agent_name)
            .map(|agent| agent.url))
    }
}

