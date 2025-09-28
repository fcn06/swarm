use anyhow::Result;
use async_trait::async_trait;


// Generic Definition of Services
use super::services::{EvaluationService, MemoryService, DiscoveryService};

// connection to client definition of generic traits
use agent_memory_service::models::Role;
use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;
use agent_evaluation_service::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_discovery_service::model::models::{AgentDefinition, TaskDefinition, ToolDefinition};
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;

/*
service_adapters.rs 
is concerned with providing a consistent interface for agents 
to use discovery (and other) services, abstracting away the communication details.

service_adapters.rs: 
This file provides adapters that implement generic service traits 
(e.g., DiscoveryService, EvaluationService, MemoryService). 
These adapters wrap the concrete service clients 
(like AgentDiscoveryServiceClient) and expose a standardized, 
abstract interface to the core business logic of the agents. 
This means that the core agent logic doesn't directly depend on the specific HTTP client 
or the exact implementation of the discovery service. 
Instead, it interacts with the DiscoveryService trait, 
and the AgentDiscoveryServiceAdapter translates these calls to the AgentDiscoveryServiceClient.
*/

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
    async fn register_agent(&self, agent_def: &AgentDefinition) -> Result<()> {
        self.client.register_agent_definition(agent_def).await?;
        Ok(())
    }

    async fn unregister_agent(&self, agent_def: &AgentDefinition) -> Result<()> {
        self.client.deregister_agent_definition(agent_def).await?;
        Ok(())
    }

    async fn get_agent_address(&self, agent_id: String) -> Result<Option<String>> {
        let all_agents = self.client.list_agent_definitions().await?;
        Ok(all_agents
            .into_iter()
            .find(|agent| agent.id == agent_id)
            .map(|agent| {
                // Find the skill that provides the endpoint, assuming it's the agent itself
                // This part needs to be refined based on how you model agent endpoints
                // For now, let's assume the agent's ID is its endpoint for discovery purposes
                // In a real scenario, AgentDefinition might have a direct `endpoint` field
                if !agent.skills.is_empty() {
                    // Assuming the first skill's output might contain the endpoint, or directly the agent's ID is the endpoint
                    // For simplicity, returning the agent's ID as the "address" for now.
                    // You would typically have a dedicated `endpoint` field in AgentDefinition.
                    Some(format!("agent://{}/", agent.id))
                } else {
                    None
                }
            }).flatten() // Use flatten to convert Option<Option<String>> to Option<String>
        )
    }

    async fn discover_agents(&self) -> Result<Vec<AgentDefinition>> {
        Ok(self.client.list_agent_definitions().await?)
    }

    async fn register_task(&self, task_def: &TaskDefinition) -> Result<()> {
        self.client.register_task_definition(task_def).await?;
        Ok(())
    }

    async fn list_tasks(&self) -> Result<Vec<TaskDefinition>> {
        Ok(self.client.list_task_definitions().await?)
    }

    async fn register_tool(&self, tool_def: &ToolDefinition) -> Result<()> {
        self.client.register_tool_definition(tool_def).await?;
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(self.client.list_tool_definitions().await?)
    }
}
