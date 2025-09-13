use anyhow::Result;
use llm_api::chat::Message;

//use serde_json::Value;

//use log::{debug, error, info,trace, warn};
use tracing::{info, error,warn,trace, debug};
use std::env;

use serde_json::json;
use std::sync::Arc;


use crate::mcp_client::mcp_client::McpClient;

use llm_api::chat::{ChatLlmInteraction, ChatCompletionRequest, ChatCompletionResponse, Choice, ToolChoice};

use llm_api::tools::Tool;

use configuration::AgentMcpConfig;

use crate::mcp_agent_logic::process_response::process_response;
use crate::mcp_client::mcp_client::execute_tool_call_v2;

use crate::mcp_client::mcp_client::get_tools_list_v2;
use crate::mcp_client::mcp_client::initialize_mcp_client_v2;
use crate::mcp_tools::tools::define_all_tools;



/// Holds the state and logic for the MCP Agent.
#[derive(Clone)]
pub struct McpAgent {
    llm_interaction: ChatLlmInteraction,
    //pub mcp_client: Arc<RunningService<RoleClient, InitializeRequestParam>>,
    pub mcp_client: Arc<McpClient>,
    messages: Vec<Message>,
    llm_all_tool: Vec<Tool>,
    agent_mcp_config: AgentMcpConfig,
}

impl McpAgent {
        /// Creates a new McpAgent instance.
        pub async fn new(
            agent_mcp_config: AgentMcpConfig,
                    ) -> anyhow::Result<Self> {
        // Set model to be used
        let model_id = agent_mcp_config.agent_mcp_model_id.clone();

        // Set model to be used
        let system_message = agent_mcp_config.agent_mcp_system_prompt.clone();

        // Set API key for LLM
        let llm_mcp_api_key = env::var("LLM_MCP_API_KEY").expect("LLM_MCP_API_KEY must be set");

        // Get client to MCP server
        let mcp_client = Arc::new(initialize_mcp_client_v2(agent_mcp_config.clone()).await?);

        // connect mcp_server and retrieve tools
        let list_tools = get_tools_list_v2(mcp_client.clone()).await?;

        // Hard coded tool for testing purpose
        let llm_all_tool = define_all_tools(list_tools)?;

        // Conversation history
        let init_messages = vec![Message {
            role: "system".to_string(),
            content: Some(system_message),
            tool_call_id: None,
            tool_calls:None

        }];


        Ok(
            Self {
                llm_interaction: ChatLlmInteraction::new(
                    agent_mcp_config.agent_mcp_llm_url.clone(),
                    model_id,
                    llm_mcp_api_key,
                ),
                mcp_client: mcp_client,
                messages:init_messages,
                llm_all_tool: llm_all_tool,
                agent_mcp_config: agent_mcp_config,
            }

        )
    }

    /// Returns a clone of the available tools.
    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.llm_all_tool.clone()
    }

    /// Push a new message to be processed by the mcp agent
    pub fn push_message(
        &mut self,
        user_message:Message,
    ) {
        self.messages.push(user_message);
    }

    /// Reset agent to initial system message
    pub fn reset_messages(
        &mut self,
    ) -> anyhow::Result<()> {

        // Set model to be used
        let system_message = self.agent_mcp_config.agent_mcp_system_prompt.clone();

        // Conversation history
        let init_messages = vec![Message {
            role: "system".to_string(),
            content: Some(system_message),
            tool_call_id: None,
            tool_calls:None

        }];

        self.messages = init_messages;

        Ok(())
    }

    /// Calls the external LLM API.
    async fn call_api_v2(
        &self,
        request_payload: &ChatCompletionRequest,
    ) -> anyhow::Result<ChatCompletionResponse> {
        debug!("Calling LLM API with payload: {:?}", request_payload);
        //trace!("Request Payload:{:#?}",request_payload.clone());

        // Serialize it to a JSON string.
        let payload_json = serde_json::to_string(&request_payload.clone()).unwrap();
        trace!("{}",payload_json);

        let response = self.llm_interaction.call_chat_completions_v2(request_payload).await?;
        debug!("LLM API Response: {:?}", response);
        Ok(response)
    }

    /// Handles the execution of tool calls requested by the LLM.
    async fn handle_tool_calls(
        &mut self,
        choice: &Choice,
    ) -> anyhow::Result<()>  {
        if let Some(tool_calls) = &choice.message.tool_calls {
            info!("--- Tool Calls Requested ---");
            let mut tool_results: Vec<Message> = Vec::new();

            for tool_call in tool_calls {
                info!("Executing tool call: {}", tool_call.id);
                
                match execute_tool_call_v2(self.mcp_client.clone(), tool_call.clone()).await {
                    Ok(result) => {
                        let result_content_str =
                            serde_json::to_string(&result.content)?;

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

    /// Executes the main agent logic loop.
    pub async fn execute_loop(&mut self) -> anyhow::Result<Option<Message>>  {
        let mut final_message: Option<Message> = None;

        for loop_count in 0..self.agent_mcp_config.agent_mcp_max_loops {
            info!(
                "Agent Loop Iteration: {}/{} ",
                loop_count + 1,
                self.agent_mcp_config.agent_mcp_max_loops
            );

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
            let choice = &response.choices[0];

            let agent_response = process_response(loop_count, choice, &mut self.messages);

            if agent_response.should_exit {
                final_message = agent_response.final_message;
                info!("Agent loop exiting based on process_response decision.");
                break;
            }

            match choice.finish_reason.as_str() {
                reason if reason == self.agent_mcp_config.agent_mcp_finish_reason_tool_calls => {
                    self.handle_tool_calls(choice).await?;
                }
                reason if reason == self.agent_mcp_config.agent_mcp_finish_reason_stop => {
                    info!("LLM indicated finish_reason 'stop'.");
                    if choice.message.role == self.agent_mcp_config.agent_mcp_role_assistant {
                        final_message = Some(Message {
                            role: self.agent_mcp_config.agent_mcp_role_assistant.clone(),
                            content: choice.message.content.clone(),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        warn!(
                            "Finish reason was 'stop'. Captured last assistant message."
                        );
                    } else {
                        warn!(
                            "Finish reason was 'stop', but last message role ('{}') wasn't assistant.",
                            choice.message.role
                        );
                    }
                    break;
                }
                other_reason => {
                    warn!("Unhandled finish reason '{}', exiting loop.", other_reason);
                    if let Some(content) = &choice.message.content {
                        if !content.trim().is_empty() {
                            final_message = Some(Message {
                                role: self.agent_mcp_config.agent_mcp_role_assistant.clone(),
                                content: Some(content.clone()),
                                tool_call_id: None,
                                tool_calls: None,
                            });
                            info!(
                                "Captured last assistant message due to unhandled finish reason '{}'",
                                other_reason
                            );
                        }
                    }
                    break;
                }
            }

            if loop_count == self.agent_mcp_config.agent_mcp_max_loops - 1 {
                info!("Maximum loop count reached.");
                error!(
                    "Maximum loop count ({}) reached, exiting.",
                    self.agent_mcp_config.agent_mcp_max_loops
                );
                anyhow::bail!(
                    "Agent exceeded maximum iterations ({})",
                    self.agent_mcp_config.agent_mcp_max_loops
                );
            }
        } // End of loop

        if final_message.is_none() {
            warn!("Agent finished without a definitive final message.");
        }

        Ok(final_message)
    }

    /// Initializes and runs the agent.
    pub async fn run_agent_internal(
        &mut self,
        user_message: Message,
    ) -> anyhow::Result<Option<Message>>  {
        self.reset_messages()?;
        self.push_message(user_message);
        self.execute_loop().await
    }

    pub async fn submit_user_text(&self, user_text: String) -> Result<String> {
        // This is a placeholder implementation.
        // In a real scenario, this would involve a more complex interaction
        // with the LLM and tool execution based on the user_text.
        info!("MCP Agent received user text: {}", user_text);
        Ok(format!("MCP agent processed: {}", user_text))
    }
}
