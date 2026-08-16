use async_trait::async_trait;
use uuid::Uuid;
use configuration::AgentConfig;

use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;

use llm_api::chat::ChatLlmInteraction;
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::debug;

use serde_json::Map;
use serde_json::Value;

use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use llm_api::chat::Message as LlmMessage;
use llm_api::google_interactions::{GeminiInteractionRequest, Part};

use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
use agent_models::execution::execution_result::ExecutionResult;
use agent_core::business_logic::services::WorkflowServiceApi;

/// Modern A2A server setup 
#[derive(Clone)]
pub struct BasicAgent {
    llm_interaction: ChatLlmInteraction,
    mcp_agent: Option<Arc<Mutex<McpAgent>>>,
}

#[async_trait]
impl Agent for BasicAgent {
    /// Creation of a new simple a2a agent
    async fn new(
        agent_config: AgentConfig,
        agent_api_key: String,
        mcp_runtime_details: Option<McpRuntimeDetails>,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            agent_api_key,
        );

        let mcp_agent = if let Some(details) = mcp_runtime_details {
            let mcp_agent = McpAgent::new(details.config, Some(details.api_key)).await?;
            Some(Arc::new(Mutex::new(mcp_agent)))
        } else {
            None
        };

        Ok(Self {
            llm_interaction,
            mcp_agent,
        })
    }

    /// Business logic for handling user request
    async fn handle_request(
        &self,
        request: GeminiInteractionRequest,
        _metadata: Option<Map<String, Value>>,
    ) -> anyhow::Result<ExecutionResult> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();

        let user_query = request
            .contents
            .iter()
            .flat_map(|c| c.parts.iter())
            .filter_map(|p| match p {
                Part::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        let llm_msg = LlmMessage {
            role: "user".to_string(),
            content: Some(user_query.clone()),
            tool_call_id: None,
            tool_calls: None,
        };

        // Use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
        let response = if self.mcp_agent.is_none() {
            self.llm_interaction
                .call_api_simple("user".to_string(), user_query)
                .await
                .unwrap()
        } else {
            let mut locked_mcp_agent = self.mcp_agent.as_ref().unwrap().lock().await;
            locked_mcp_agent
                .run_agent_internal(llm_msg)
                .await
                .unwrap()
        };

        let llm_content = response
            .expect("No Return from LLM")
            .content
            .expect("Empty result from Llm");

        let output_value = match serde_json::from_str::<Value>(&llm_content) {
            Ok(json_val) => json_val,
            Err(_) => Value::String(llm_content),
        };

        debug!("Output Value from Basic Agent: {:?}", output_value);

        Ok(ExecutionResult {
            request_id,
            conversation_id,
            success: true,
            output: output_value,
        })
    }
}
