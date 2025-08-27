use async_trait::async_trait;
use crate::planning::plan_definition::{ExecutionResult};
use llm_api::chat::Message as LlmMessage;
use configuration::AgentConfig;

use std::sync::Arc;
use crate::business_logic::services::MemoryService;
use crate::business_logic::services::EvaluationService;
use crate::business_logic::services::DiscoveryService;
use crate::business_logic::services::WorkflowServiceApi;

use serde_json::Map;
use serde_json::Value;

#[async_trait]
pub trait Agent: Send + Sync  + Clone + 'static {
    async fn new( 
        agent_config: AgentConfig, 
        evaluation_service: Option<Arc<dyn EvaluationService>>, 
        memory_service: Option<Arc<dyn MemoryService>>, 
        discovery_service: Option<Arc<dyn DiscoveryService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>
    ) -> anyhow::Result<Self>;
    //async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
    async fn handle_request(&self, request: LlmMessage, metadata:Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult>;
    
}


// Option<Map<String, Value>>