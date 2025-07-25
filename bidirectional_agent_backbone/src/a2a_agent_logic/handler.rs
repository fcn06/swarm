use async_trait::async_trait;
use a2a_rs::port::message_handler::AsyncMessageHandler;
use a2a_rs::domain::{Message, TaskState, Part, Task};
use crate::a2a_agent_logic::planner_agent::PlannerAgent;
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;
use llm_api::chat::ChatLlmInteraction;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use a2a_rs::A2AError;
use std::pin::Pin;
use std::future::Future;

#[derive(Clone)]
pub struct BidirectionalAgentHandler {
    llm_interaction: ChatLlmInteraction,
    mcp_agent: Option<McpAgent>,
    planner_agent: Arc<Mutex<Option<PlannerAgent>>>,
}

impl BidirectionalAgentHandler {
    pub fn with_storage<S>(
        llm_interaction: ChatLlmInteraction,
        mcp_agent: Option<McpAgent>,
        planner_agent: Option<PlannerAgent>,
        _storage: S,
    ) -> Self
    where
        S: Send + Sync + 'static,
    {
        Self {
            llm_interaction,
            mcp_agent,
            planner_agent: Arc::new(Mutex::new(planner_agent)),
        }
    }
}

#[async_trait]
impl AsyncMessageHandler for BidirectionalAgentHandler {
    async fn process_message<'a>(
            &self,
            task_id: &'a str,
            message: &'a Message,
            session_id: Option<&'a str>,
        ) -> Result<Task, A2AError> {


        let user_query = message.parts.iter().filter_map(|part| {
            if let Part::Text { text, metadata: _ } = part {
                Some(text.clone())
            } else {
                None
            }
        }).collect::<Vec<String>>().join(" ");

        tracing::info!("BidirectionalAgentHandler received message: {}", user_query);

        let handler_clone = self.clone();
        let message_clone = message.clone();

        Box::pin(async move {
            // Try to handle with planner agent first
            if let Some(planner) = handler_clone.planner_agent.lock().await.as_mut() {
                tracing::info!("BidirectionalAgentHandler: Attempting to handle with PlannerAgent");
                let execution_result = planner.submit_user_text(user_query.clone()).await;
                tracing::info!("BidirectionalAgentHandler: PlannerAgent execution result success: {}", execution_result.success);
                if execution_result.success {
                    return Ok(Task::response(message_clone.message_id.clone(), TaskState::Completed, vec![Part::text(execution_result.output)]));
                } else {
                    // If planner failed, but returned some output, send it as response
                    if !execution_result.output.is_empty() {
                        return Ok(Task::response(message_clone.message_id.clone(), TaskState::Failed, vec![Part::text(format!("Planner failed: {}", execution_result.output))]));
                    }
                }
            }

            // If no planner or planner failed, try to handle with MCP agent
            if let Some(mcp) = &handler_clone.mcp_agent {
                tracing::info!("BidirectionalAgentHandler: Attempting to handle with McpAgent");
                match mcp.submit_user_text(user_query.clone()).await {
                    Ok(mcp_response) => {
                        tracing::info!("BidirectionalAgentHandler: McpAgent handled message successfully.");
                        return Ok(Task::response(message_clone.message_id.clone(), TaskState::Completed, vec![Part::text(mcp_response)]));
                    }
                    Err(e) => {
                        tracing::error!("BidirectionalAgentHandler: McpAgent failed to handle message: {:?}", e);
                        // Fallback to direct LLM if MCP fails
                    }
                }
            }

            // If neither planner nor MCP handled the message, use direct LLM interaction
            tracing::info!("BidirectionalAgentHandler: Falling back to direct LLM interaction.");
            match handler_clone.llm_interaction.call_api_simple_v2("user".to_string(), user_query.clone()).await {
                Ok(llm_response_option) => {
                    let llm_response = llm_response_option.unwrap_or_else(|| "No response from LLM.".to_string());
                    tracing::info!("BidirectionalAgentHandler: Direct LLM interaction response: {}", llm_response);
                    Ok(Task::response(message_clone.message_id.clone(), TaskState::Completed, vec![Part::text(llm_response)]))
                }
                Err(e) => {
                    tracing::error!("BidirectionalAgentHandler: Direct LLM interaction failed: {:?}", e);
                    Err(A2AError::Internal(format!("Failed to process message with LLM: {:?}", e)))
                }
            }
        })
    }
}
