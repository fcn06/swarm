use anyhow::Result;
use async_trait::async_trait;
//use agent_evaluation_service::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
use agent_models::evaluation::evaluation_models::{AgentEvaluationLogData,JudgeEvaluation};

//use agent_memory_service::models::Role;
use agent_models::memory::memory_models::Role;

use std::any::Any;

//use agent_discovery_service::model::models::{AgentDefinition, TaskDefinition, ToolDefinition};
use agent_models::registry::registry_models::{AgentDefinition, TaskDefinition, ToolDefinition};

/// A trait that defines the interface for an evaluation service.
#[async_trait]
pub trait EvaluationService: Send + Sync {
    async fn log_evaluation(&self, data: AgentEvaluationLogData) -> Result<JudgeEvaluation>;
}

/// A trait that defines the interface for a memory service.
#[async_trait]
pub trait MemoryService: Send + Sync {
    async fn log(&self, conversation_id: String, role: Role, text: String, agent_name: Option<String>) -> Result<()>;
}

/// A trait that defines the interface for a discovery service.
#[async_trait]
pub trait DiscoveryService: Send + Sync {
    async fn register_agent(&self, agent_def: &AgentDefinition) -> Result<()>;
    async fn unregister_agent(&self, agent_def: &AgentDefinition) -> Result<()>;
    async fn get_agent_address(&self, agent_id: String) -> Result<Option<String>>;
    async fn discover_agents(&self) -> Result<Vec<AgentDefinition>>; 
    async fn register_task(&self, task_def: &TaskDefinition) -> Result<()>;
    async fn list_tasks(&self) -> Result<Vec<TaskDefinition>>; 
    async fn register_tool(&self, tool_def: &ToolDefinition) -> Result<()>;
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>>; 
    async fn list_available_resources(&self) -> Result<String>;
}

// New trait for workflow related services
#[async_trait]
pub trait WorkflowServiceApi: Send + Sync + 'static  + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    async fn refresh_agents(&self) -> anyhow::Result<()>; // Re-added refresh_agents
}