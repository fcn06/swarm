use async_trait::async_trait;
use crate::planning::plan_definition::{ExecutionResult};
use llm_api::chat::Message as LlmMessage;
use configuration::AgentConfig;

use std::sync::Arc;
use crate::business_logic::services::MemoryService;
use crate::business_logic::services::EvaluationService;

#[async_trait]
pub trait Agent: Send + Sync  + Clone + 'static {
    //async fn new( agent_config: AgentConfig) -> anyhow::Result<Self>;
    async fn new( agent_config: AgentConfig, evaluation_service: Option<Arc<dyn EvaluationService>>,memory_service: Option<Arc<dyn MemoryService>>) -> anyhow::Result<Self>;
    async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
}

