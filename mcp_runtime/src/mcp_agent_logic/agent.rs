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
/// It is responsible for interacting with the Language Model (LLM), managing conversation history,
/// and executing tool calls based on the LLM's responses.
///
/// The agent operates in a loop, processing user messages, calling the LLM API, and handling tool
/// execution until a final response is generated or a termination condition is met.
///
/// # Fields
/// - `llm_interaction`: Handles communication with the LLM API.
/// - `mcp_client`: A client for interacting with the MCP server to execute tools.
/// - `messages`: A vector of `Message` structs representing the conversation history.
/// - `llm_all_tool`: A vector of `Tool` structs available for the LLM to use.
/// - `agent_mcp_config`: Configuration settings for the MCP agent.
#[derive(Clone)]
pub struct McpAgent {
    llm_interaction: ChatLlmInteraction,
    pub mcp_client: Arc<McpClient>,
    messages: Vec<Message>,
    llm_all_tool: Vec<Tool>,
    agent_mcp_config: McpRuntimeConfig,
}

impl McpAgent {
    /// Creates a new `McpAgent` instance.
    ///
    /// This asynchronous function initializes the agent by:
    /// 1. Retrieving necessary configuration values.
    /// 2. Initializing the LLM interaction client.
    /// 3. Establishing a connection to the MCP server and fetching available tools.
    /// 4. Setting up the initial conversation history with a system message.
    ///
    /// # Arguments
    ///
    /// * `agent_mcp_config` - Configuration settings for the MCP agent, including LLM details
    ///                        and MCP server information.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Self)`: A new `McpAgent` instance if all initializations are successful.
    /// - `Err(anyhow::Error)`: An error if any configuration is missing, environment variables
    ///                          are not set, or communication with the MCP server fails.
    pub async fn new(agent_mcp_config: McpRuntimeConfig) -> anyhow::Result<Self> {
        let model_id = agent_mcp_config.agent_mcp_model_id.clone();
        let system_message = agent_mcp_config.agent_mcp_system_prompt.clone();

        let llm_mcp_api_key = if let Some(env_var_name) = &agent_mcp_config.agent_mcp_llm_api_key_env_var {
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

    pub async fn new_v2(agent_mcp_config: McpRuntimeConfig, mcp_runtime_api_key:String,) -> anyhow::Result<Self> {
        let model_id = agent_mcp_config.agent_mcp_model_id.clone();
        let system_message = agent_mcp_config.agent_mcp_system_prompt.clone();

        let llm_mcp_api_key = mcp_runtime_api_key;

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

    /// Returns a clone of the available tools that the LLM can use.
    ///
    /// # Returns
    ///
    /// A `Vec<Tool>` containing all the tools configured for this agent.
    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.llm_all_tool.clone()
    }

    /// Appends a new user message to the agent's conversation history.
    ///
    /// This method is used to add incoming messages (e.g., from a user) to the
    /// internal message queue, which will be sent to the LLM in subsequent API calls.
    ///
    /// # Arguments
    ///
    /// * `user_message` - The `Message` struct representing the user's input.
    pub fn push_message(&mut self, user_message: Message) {
        self.messages.push(user_message);
    }

    /// Resets the agent's conversation history to only the initial system message.
    ///
    /// This is useful for starting a new conversation or clearing the context of a previous one.
    /// The system message is re-initialized from the agent's configuration.
    ///
    /// # Returns
    ///
    /// An `anyhow::Result` which is:
    /// - `Ok(())`: If the messages were successfully reset.
    /// - `Err(anyhow::Error)`: If there was an issue retrieving the system prompt (though
    ///                          this should ideally be handled during agent initialization).
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

    /// Calls the external LLM API with the provided chat completion request payload.
    ///
    /// This private helper function handles the actual communication with the LLM.
    /// It also logs the request and response for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `request_payload` - A reference to the `ChatCompletionRequest` containing the
    ///                         messages, model, and other parameters for the LLM call.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(ChatCompletionResponse)`: The successful response from the LLM.
    /// - `Err(anyhow::Error)`: An error if the LLM API call fails.
    async fn call_api_v2(
        &self,
        request_payload: &ChatCompletionRequest,
    ) -> anyhow::Result<ChatCompletionResponse> {
        debug!("Calling LLM API with payload: {:?}", request_payload);
        let payload_json = serde_json::to_string(&request_payload)
            .context("Failed to serialize LLM request payload")?;
        trace!("Request Payload: {}", payload_json);

        let response = self.llm_interaction.call_chat_completions_v2(request_payload).await
            .context("LLM chat completion API call failed")?;
        debug!("LLM API Response: {:?}", response);
        Ok(response)
    }

    /// Handles the execution of tool calls requested by the LLM.
    ///
    /// When the LLM's response indicates a `tool_calls` finish reason, this function
    /// iterates through the requested tool calls, executes them via the MCP client,
    /// and appends their results (or error messages) back to the conversation history.
    ///
    /// # Arguments
    ///
    /// * `choice` - A reference to the `Choice` struct from the LLM's response,
    ///              containing the tool call details.
    ///
    /// # Returns
    ///
    /// An `anyhow::Result` which is:
    /// - `Ok(())`: If all tool calls were processed (successfully or with an error message recorded).
    /// - `Err(anyhow::Error)`: If there was a critical error during tool execution or serialization
    ///                          that prevented adding a message to the history.
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

    /// Executes the main agent logic loop.
    ///
    /// This function drives the agent's core behavior. It repeatedly calls the LLM, processes
    /// its responses, handles tool calls, and updates the conversation history until the LLM
    /// indicates a stop, a maximum loop count is reached, or an unrecoverable error occurs.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Option<Message>)`: An `Option` containing the final `Message` from the assistant
    ///                            if the loop concludes successfully, or `None` if no definitive
    ///                            final message was captured.
    /// - `Err(anyhow::Error)`: An error if the LLM response is empty, the maximum loop count
    ///                          is exceeded, or a critical error occurs during API calls or
    ///                          tool execution.
    pub async fn execute_loop(&mut self) -> anyhow::Result<Option<Message>> {
        let mut final_message: Option<Message> = None;

        for loop_count in 0..self.agent_mcp_config.agent_mcp_max_loops {
            info!(
                "Agent Loop Iteration: {}/{}",
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
        }

        if final_message.is_none() {
            warn!("Agent finished without a definitive final message.");
        }

        Ok(final_message)
    }

    /// Initializes and runs the agent with a given user message.
    ///
    /// This function serves as the primary entry point for running the agent's logic.
    /// It first resets the agent's internal message history, then pushes the new
    /// `user_message`, and finally enters the main `execute_loop` to process it.
    ///
    /// # Arguments
    ///
    /// * `user_message` - The `Message` struct representing the initial user input
    ///                      for the agent to process.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Option<Message>)`: An `Option` containing the final `Message` from the assistant
    ///                            if the agent successfully processes the user message and
    ///                            the loop concludes, or `None` if no definitive final message
    ///                            was captured.
    /// - `Err(anyhow::Error)`: An error if any step in the process fails, such as resetting
    ///                          messages, LLM API calls, or tool execution.
    pub async fn run_agent_internal(
        &mut self,
        user_message: Message,
    ) -> anyhow::Result<Option<Message>> {
        self.reset_messages()?;
        self.push_message(user_message);
        self.execute_loop().await
    }

    /// Submits plain user text to the MCP Agent for processing.
    ///
    /// This is a placeholder implementation. In a real-world scenario, this method
    /// would likely convert the `user_text` into a `Message` and then call
    /// `run_agent_internal` to engage the full agent logic.
    ///
    /// # Arguments
    ///
    /// * `user_text` - A `String` containing the plain text input from the user.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(String)`: A confirmation string indicating that the text was processed.
    /// - `Err(anyhow::Error)`: An error if processing the user text (e.g., within
    ///                          a more complex `run_agent_internal` call) fails.
    pub async fn submit_user_text(&self, user_text: String) -> Result<String> {
        info!("MCP Agent received user text: {}", user_text);
        // Placeholder: In a real scenario, this would involve a more complex interaction
        // with the LLM and tool execution based on the user_text.
        Ok(format!("MCP agent processed: {}", user_text))
    }
}
