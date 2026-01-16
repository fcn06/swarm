use anyhow::{Result, Context};
use llm_api::chat::Message;
use tracing::{info, error, warn, trace, debug};
use std::env;
use serde_json::json;
use std::sync::Arc;
use crate::mcp_client::mcp_client::McpClient;
use llm_api::chat::{ChatLlmInteraction, ChatCompletionRequest, ChatCompletionResponse, Choice, ToolChoice};
use llm_api::tools::Tool;
use configuration::McpRuntimeConfig;
use crate::mcp_agent_logic::process_response::process_response;
use crate::mcp_client::mcp_client::execute_tool_call_v2;
use crate::mcp_client::mcp_client::get_tools_list_v2;
use crate::mcp_client::mcp_client::initialize_mcp_client_v2;
use crate::mcp_tools::tools::define_all_tools;

/// The `McpAgent` struct encapsulates the state and logic for the Multi-Controller Piper (MCP) agent.
#[derive(Clone)]
pub struct McpAgent {
    llm_interaction: ChatLlmInteraction,
    pub mcp_client: Arc<McpClient>,
    messages: Vec<Message>,
    llm_all_tool: Vec<Tool>,
    agent_mcp_config: McpRuntimeConfig,
}

impl McpAgent {
    pub async fn new(agent_mcp_config: McpRuntimeConfig, mcp_runtime_api_key: Option<String>) -> anyhow::Result<Self> {
        let model_id = agent_mcp_config.agent_mcp_model_id.clone();
        let system_message = agent_mcp_config.agent_mcp_system_prompt.clone();

        let llm_mcp_api_key = if let Some(api_key) = mcp_runtime_api_key {
            api_key
        } else if let Some(env_var_name) = &agent_mcp_config.agent_mcp_llm_api_key_env_var {
            env::var(env_var_name)
                .context(format!("Environment variable '{}' for LLM API key must be set", env_var_name))?
        } else {
            env::var("LLM_MCP_API_KEY")
                .context("LLM_MCP_API_KEY environment variable must be set")?
        };

        let mcp_client = Arc::new(initialize_mcp_client_v2(agent_mcp_config.clone())
            .await
            .context("Failed to initialize MCP client")?);

        let list_tools = get_tools_list_v2(mcp_client.clone())
            .await
            .context("Failed to retrieve tools list from MCP server")?;

        let llm_all_tool = define_all_tools(list_tools)
            .context("Failed to define tools from retrieved list")?;

        let init_messages = vec![Message {
            role: "system".to_string(),
            content: Some(system_message),
            tool_call_id: None,
            tool_calls: None,
        }];

        Ok(Self {
            llm_interaction: ChatLlmInteraction::new(
                agent_mcp_config.agent_mcp_llm_url.clone(),
                model_id,
                llm_mcp_api_key,
            ),
            mcp_client,
            messages: init_messages,
            llm_all_tool,
            agent_mcp_config,
        })
    }

    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.llm_all_tool.clone()
    }

    pub fn push_message(&mut self, user_message: Message) {
        self.messages.push(user_message);
    }

    pub fn reset_messages(&mut self) -> anyhow::Result<()> {
        let system_message = self.agent_mcp_config.agent_mcp_system_prompt.clone();
        self.messages = vec![Message {
            role: "system".to_string(),
            content: Some(system_message),
            tool_call_id: None,
            tool_calls: None,
        }];
        Ok(())
    }

    async fn call_api_v2(
        &self,
        request_payload: &ChatCompletionRequest,
    ) -> anyhow::Result<ChatCompletionResponse> {
        debug!("Calling LLM API with payload: {:?}", request_payload);
        let response = self.llm_interaction.call_chat_completions_v2(request_payload).await
            .context("LLM chat completion API call failed")?;
        debug!("LLM API Response: {:?}", response);
        Ok(response)
    }

    async fn handle_tool_calls(
        &mut self,
        choice: &Choice,
    ) -> anyhow::Result<()> {
        if let Some(tool_calls) = &choice.message.tool_calls {
            info!("--- Tool Calls Requested ---");
            let mut tool_results: Vec<Message> = Vec::new();

            for tool_call in tool_calls {
                info!("Executing tool call: {}", tool_call.id);
                
                match execute_tool_call_v2(self.mcp_client.clone(), tool_call.clone()).await {
                    Ok(result) => {
                        let result_content_str = serde_json::to_string(&result.content)
                            .context(format!("Failed to serialize tool result for tool_call_id: {}", tool_call.id))?;

                        tool_results.push(Message {
                            role: self.agent_mcp_config.agent_mcp_role_tool.clone(),
                            content: Some(format!("Response from Tool: {}", result_content_str)),
                            tool_call_id: Some(tool_call.id.clone()),
                            tool_calls: None,
                        });
                    }
                    Err(e) => {
                        error!("Error executing tool {}: {}", tool_call.id, e);
                        let error_content = json!({
                            "error": format!("Error executing tool '{}': {}", tool_call.id, e),
                            "tool_call_id": tool_call.id
                        });
                        tool_results.push(Message {
                            role: self.agent_mcp_config.agent_mcp_role_tool.clone(),
                            content: Some(error_content.to_string()),
                            tool_call_id: Some(tool_call.id.clone()),
                            tool_calls: None,
                        });
                    }
                }
            }
            self.messages.extend(tool_results);
        }
        Ok(())
    }

    pub async fn execute_loop(&mut self) -> anyhow::Result<Option<Message>> {
        let mut final_message: Option<Message> = None;

        for loop_count in 0..self.agent_mcp_config.agent_mcp_max_loops {
            info!(
                "Agent Loop Iteration: {}/{}",
                loop_count + 1,
                self.agent_mcp_config.agent_mcp_max_loops
            );

            // Create a clean version of messages for the LLM (without reasoning tags if preferred)
            // Actually, we should clean the message we just received before putting it back in history.
            
            let request_payload = ChatCompletionRequest {
                model: self.llm_interaction.model_id.clone(),
                messages: self.messages.clone(),
                temperature: Some(0.0),
                max_tokens: Some(1024),
                top_p: Some(1.0),
                stop: None,
                stream: Some(false),
                tools: Some(self.llm_all_tool.clone()),
                tool_choice: Some(ToolChoice::String(
                    self.agent_mcp_config.agent_mcp_tool_choice_auto.clone(),
                )),
            };

            let response = self.call_api_v2(&request_payload).await?;

            if response.choices.is_empty() {
                error!("LLM response contained no choices.");
                anyhow::bail!("LLM response contained no choices.");
            }
            
            // Extract the first choice
            let mut choice = response.choices[0].clone();
            
            // Clean content of the message in the choice (remove <think> tags)
            if let Some(content) = choice.message.content.as_mut() {
                *content = self.llm_interaction.remove_think_tags(content.clone()).await?;
            }

            // Process the response to update history and check if we should exit
            let agent_response = process_response(loop_count, &choice, &mut self.messages);

            if agent_response.should_exit {
                final_message = agent_response.final_message;
                info!("Agent loop exiting based on process_response decision (Reason: {}).", choice.finish_reason);
                break;
            }

            // If we are here, it means finish_reason was likely "tool_calls"
            if choice.finish_reason == self.agent_mcp_config.agent_mcp_finish_reason_tool_calls {
                self.handle_tool_calls(&choice).await?;
            } else {
                warn!("Expected tool_calls finish_reason but got '{}'. Exiting loop.", choice.finish_reason);
                final_message = agent_response.final_message;
                break;
            }

            if loop_count == self.agent_mcp_config.agent_mcp_max_loops - 1 {
                error!("Maximum loop count ({}) reached, exiting.", self.agent_mcp_config.agent_mcp_max_loops);
                anyhow::bail!("Agent exceeded maximum iterations ({})", self.agent_mcp_config.agent_mcp_max_loops);
            }
        }

        if final_message.is_none() {
            warn!("Agent finished without a definitive final message.");
        }

        Ok(final_message)
    }

    pub async fn run_agent_internal(
        &mut self,
        user_message: Message,
    ) -> anyhow::Result<Option<Message>> {
        self.reset_messages()?;
        self.push_message(user_message);
        self.execute_loop().await
    }

    pub async fn submit_user_text(&self, user_text: String) -> Result<String> {
        info!("MCP Agent received user text: {}", user_text);
        Ok(format!("MCP agent processed: {}", user_text))
    }
}
