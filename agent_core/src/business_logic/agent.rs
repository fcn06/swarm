use async_trait::async_trait;
//use crate::execution::execution_result::{ExecutionResult};
use agent_models::execution::execution_result::{ExecutionResult};

use llm_api::chat::Message as LlmMessage;
use configuration::{AgentConfig, McpRuntimeConfig};

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
        agent_api_key:String,
        mcp_runtime_config: Option<McpRuntimeConfig>,
        mcp_runtime_api_key:Option<String>,
        evaluation_service: Option<Arc<dyn EvaluationService>>, 
        memory_service: Option<Arc<dyn MemoryService>>, 
        discovery_service: Option<Arc<dyn DiscoveryService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>
    ) -> anyhow::Result<Self>;
    async fn handle_request(&self, request: LlmMessage, metadata:Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult>;
    
}
