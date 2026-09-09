use async_trait::async_trait;
use uuid::Uuid;
use configuration::AgentConfig;
use std::collections::HashMap;

use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;

use llm_api::chat::ChatLlmInteraction;
use std::sync::Arc;

use tracing::debug;

use serde_json::Value;

use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use mcp_runtime::mcp_agent_logic::context_providers::{
    HistoryContextProvider, IdentityContextProvider, SemanticMemoryContextProvider,
};
use llm_api::chat::Message as LlmMessage;
use agent_models::agent_request::AgentRequest;
use agent_models::memory::memory_models::{IdentityContext, Role as MemoryRole};

use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
use agent_models::execution::execution_result::ExecutionResult;
use agent_core::business_logic::services::WorkflowServiceApi;

/// Modern A2A server setup 
#[derive(Clone)]
pub struct BasicAgent {
    llm_interaction: ChatLlmInteraction,
    mcp_agent: Option<Arc<McpAgent>>,
    memory_service: Option<Arc<dyn MemoryService>>,
}

#[async_trait]
impl Agent for BasicAgent {
    /// Creation of a new simple a2a agent
    async fn new(
        agent_config: AgentConfig,
        agent_api_key: String,
        mcp_runtime_details: Option<McpRuntimeDetails>,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            agent_api_key,
        );

        let mcp_agent = if let Some(details) = mcp_runtime_details {
            let mut agent = McpAgent::new(details.config.clone(), Some(details.api_key)).await?;

            if let Some(ref mem) = memory_service {
                agent = agent.with_memory_service(mem.clone());

                if let Some(history_len) = details.config.agent_mcp_history_length {
                    agent = agent.with_context_provider(Arc::new(
                        HistoryContextProvider::new(mem.clone(), history_len),
                    ));
                }

                if details.config.agent_mcp_enable_memory_recall == Some(true) {
                    agent = agent.with_context_provider(Arc::new(
                        SemanticMemoryContextProvider::new(mem.clone(), 5, None),
                    ));
                }
            }

            if details.config.agent_mcp_enable_identity_context == Some(true) {
                let identity = IdentityContext {
                    role: agent_config.agent_description.clone(),
                    objectives: agent_config.agent_tags.clone(),
                    constraints: vec![],
                    capabilities: vec![agent_config.agent_skill_name.clone()],
                };
                agent = agent.with_context_provider(Arc::new(
                    IdentityContextProvider::new(identity),
                ));
            }

            Some(Arc::new(agent))
        } else {
            None
        };

        Ok(Self {
            llm_interaction,
            mcp_agent,
            memory_service,
        })
    }

    /// Business logic for handling user request
    async fn handle_request(
        &self,
        request: AgentRequest,
    ) -> anyhow::Result<ExecutionResult> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let conversation_id = request
            .session_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let user_query = request.user_query();

        let llm_msg = LlmMessage {
            role: "user".to_string(),
            content: Some(user_query.clone()),
            tool_call_id: None,
            tool_calls: None,
        };

        let metadata: Option<HashMap<String, String>> = request.metadata.as_ref().map(|meta_map| {
            meta_map
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()),
                    )
                })
                .collect()
        });

        // Use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
        let response = if let Some(ref agent) = self.mcp_agent {
            agent
                .run_agent_with_context(
                    llm_msg,
                    Some(conversation_id.clone()),
                    metadata,
                    None,
                )
                .await?
        } else {
            self.llm_interaction
                .call_api_simple("user".to_string(), user_query.clone())
                .await?
        };

        let llm_content = response
            .and_then(|m| m.content)
            .unwrap_or_else(|| "Empty result from LLM".to_string());

        // Log turn to memory service if available
        if let Some(ref mem) = self.memory_service {
            let _ = mem
                .log(
                    conversation_id.clone(),
                    MemoryRole::User,
                    user_query,
                    None,
                )
                .await;
            let _ = mem
                .log(
                    conversation_id.clone(),
                    MemoryRole::Agent,
                    llm_content.clone(),
                    None,
                )
                .await;
        }

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
