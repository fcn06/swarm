use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
//use agent_evaluation_service::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
use agent_models::evaluation::evaluation_models::{AgentEvaluationLogData,JudgeEvaluation};


use agent_memory_service::memory_service_client::agent_memory_client::AgentMemoryServiceClient;
use agent_memory_service::models::Role;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
//use agent_discovery_service::model::models::{AgentDefinition, TaskDefinition, ToolDefinition};
use agent_models::registry::registry_models::{AgentDefinition, TaskDefinition, ToolDefinition};

/********************************************/
/* Service Adapter for Evaluation Service   */
/********************************************/

// Adapter for AgentEvaluationServiceClient
pub struct AgentEvaluationServiceAdapter {
    client: AgentEvaluationServiceClient,
}

impl AgentEvaluationServiceAdapter {
    pub fn new(url: &str) -> Self {
        let client = AgentEvaluationServiceClient::new(url.to_string());
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
    pub fn new(url: &str) -> Self {
        let client = AgentMemoryServiceClient::new(url.to_string());
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
    pub fn new(url: &str) -> Self {
        let client = AgentDiscoveryServiceClient::new(url);
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
                if !agent.skills.is_empty() {
                    Some(format!("agent://{}/", agent.id))
                } else {
                    None
                }
            }).flatten()
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
