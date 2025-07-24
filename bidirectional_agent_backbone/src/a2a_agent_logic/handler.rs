use async_trait::async_trait;
use a2a_rs::port::message_handler::AsyncMessageHandler;
use a2a_rs::domain::{Message, TaskState, Part};
use crate::a2a_agent_logic::planner_agent::PlannerAgent;
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;
use llm_api::chat::ChatLlmInteraction;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

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
    async fn handle_message(&self, message: Message) -> Result<Message> {
        // Extract the user query from the message parts
        let user_query = message.parts.iter().filter_map(|part| {
            if let Part::Text { text, metadata: _ } = part {
                Some(text.clone())
            } else {
                None
            }
        }).collect::<Vec<String>>().join(" ");

        tracing::info!("BidirectionalAgentHandler received message: {}", user_query);

        // Try to handle with planner agent first
        if let Some(planner) = self.planner_agent.lock().await.as_mut() {
            tracing::info!("BidirectionalAgentHandler: Attempting to handle with PlannerAgent");
            match planner.submit_user_text(user_query.clone()).await {
                Ok(execution_result) => {
                    tracing::info!("BidirectionalAgentHandler: PlannerAgent execution result success: {}", execution_result.success);
                    if execution_result.success {
                        return Ok(Message::response(message.id, TaskState::Completed, vec![Part::text(execution_result.output)]));
                    } else {
                        // If planner failed, but returned some output, send it as response
                        if !execution_result.output.is_empty() {
                             return Ok(Message::response(message.id, TaskState::Failed, vec![Part::text(format!("Planner failed: {}", execution_result.output))]));
                        }
                       
                    }
                }
                Err(e) => {
                    tracing::error!("BidirectionalAgentHandler: PlannerAgent failed to handle message: {:?}", e);
                    // Fallback to MCP or direct LLM if planner completely fails
                }
            }
        }

        // If no planner or planner failed, try to handle with MCP agent
        if let Some(mcp) = &self.mcp_agent {
            tracing::info!("BidirectionalAgentHandler: Attempting to handle with McpAgent");
            match mcp.submit_user_text(user_query.clone()).await {
                Ok(mcp_response) => {
                    tracing::info!("BidirectionalAgentHandler: McpAgent handled message successfully.");
                    return Ok(Message::response(message.id, TaskState::Completed, vec![Part::text(mcp_response)]));
                }
                Err(e) => {
                    tracing::error!("BidirectionalAgentHandler: McpAgent failed to handle message: {:?}", e);
                    // Fallback to direct LLM if MCP fails
                }
            }
        }

        // If neither planner nor MCP handled the message, use direct LLM interaction
        tracing::info!("BidirectionalAgentHandler: Falling back to direct LLM interaction.");
        match self.llm_interaction.call_api_simple_v2("user".to_string(), user_query.clone()).await {
            Ok(llm_response_option) => {
                let llm_response = llm_response_option.unwrap_or_else(|| "No response from LLM.".to_string());
                tracing::info!("BidirectionalAgentHandler: Direct LLM interaction response: {}", llm_response);
                Ok(Message::response(message.id, TaskState::Completed, vec![Part::text(llm_response)]))
            }
            Err(e) => {
                tracing::error!("BidirectionalAgentHandler: Direct LLM interaction failed: {:?}", e);
                Err(anyhow::anyhow!("Failed to process message with LLM: {:?}", e))
            }
        }
    }
}
