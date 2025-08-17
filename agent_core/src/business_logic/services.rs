use anyhow::Result;
use async_trait::async_trait;
use agent_evaluation_service::evaluation_server::judge_agent::AgentLogData;
use agent_memory_service::models::Role;

/// A trait that defines the interface for an evaluation service.
#[async_trait]
pub trait EvaluationService: Send + Sync {
    async fn log_evaluation(&self, data: AgentLogData) -> Result<()>;
}

/// A trait that defines the interface for a memory service.
#[async_trait]
pub trait MemoryService: Send + Sync {
    async fn log(&self, conversation_id: String, role: Role, text: String, agent_name: Option<String>) -> Result<()>;
}


// todo:Do the same thing for DiscoveryService