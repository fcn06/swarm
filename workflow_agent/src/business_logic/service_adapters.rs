use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_evaluation_service::evaluation_server::judge_agent::AgentLogData;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;
use agent_memory_service::models::Role;
use agent_core::business_logic::services::{EvaluationService, MemoryService};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use agent_core::business_logic::services::DiscoveryService;
use a2a_rs::domain::AgentCard;

// Added for AgentInvoker implementation
use workflow_management::agent_communication::agent_invoker::AgentInvoker;

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
    async fn log_evaluation(&self, data: AgentLogData) -> Result<()> {
        self.client.log_evaluation(data).await.map(|_| ())
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

/********************************************/
/* Placeholder for AgentInvoker Adapter     */
/********************************************/

// This adapter is added as a placeholder to demonstrate awareness
// of the new AgentInvoker trait signature.
// In the current architecture, WorkflowAgent consumes an AgentRegistry
// (from the workflow_management crate) which is expected to handle agent invocation.
// If WorkflowAgent itself were to act as an AgentInvoker (e.g., for local agents
// or in a test environment), this would be an appropriate place for its implementation.
pub struct LocalAgentInvoker;

#[async_trait]
impl AgentInvoker for LocalAgentInvoker {
    async fn interact(&self, agent_id: String, message: String, skill: String) -> anyhow::Result<serde_json::Value> {
        // This is a placeholder implementation.
        // In a real scenario, this would contain logic to communicate with other agents.
        println!("LocalAgentInvoker: Interacting with agent {} with message '{}' and skill '{}'", agent_id, message, skill);
        // For demonstration, return a dummy JSON value
        Ok(serde_json::json!({
            "status": "simulated_success",
            "agent_id": agent_id,
            "message_received": message,
            "skill_requested": skill,
            "response": "This is a simulated response from LocalAgentInvoker."
        }))
    }
}
