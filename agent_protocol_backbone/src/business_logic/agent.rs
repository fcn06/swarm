use async_trait::async_trait;
use crate::planning::plan_definition::{ExecutionResult};
use llm_api::chat::Message as LlmMessage;
use crate::config::agent_config::AgentConfig;


#[async_trait]
pub trait Agent: Send + Sync  + Clone + 'static {
    async fn new( agent_config: AgentConfig) -> anyhow::Result<Self>;
    async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
}

