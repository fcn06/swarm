use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_server::judge_agent::AgentEvaluationLogData;
use agent_memory_service::models::Role;
use std::any::Any;
use a2a_rs::domain::AgentCard;

/// A trait that defines the interface for an evaluation service.
#[async_trait]
pub trait EvaluationService: Send + Sync {
    async fn log_evaluation(&self, data: AgentEvaluationLogData) -> Result<()>;
}

/// A trait that defines the interface for a memory service.
#[async_trait]
pub trait MemoryService: Send + Sync {
    async fn log(&self, conversation_id: String, role: Role, text: String, agent_name: Option<String>) -> Result<()>;
}

/// A trait that defines the interface for a discovery service.
#[async_trait]
pub trait DiscoveryService: Send + Sync {
    async fn register_agent(&self, agent_card: &AgentCard) -> Result<()>;
    async fn unregister_agent(&self, agent_card: &AgentCard) -> Result<()>;
    async fn get_agent_address(&self, agent_name: String) -> Result<Option<String>>;
}

// New trait for workflow related services
pub trait WorkflowServiceApi: Send + Sync + 'static  + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
