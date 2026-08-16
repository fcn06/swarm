use anyhow::{Result, Context};
use llm_api::chat::Message;
use tracing::{info, error, warn, debug};
use std::env;
use serde_json::json;
use std::sync::Arc;
use crate::mcp_client::mcp_client::McpClient;
use llm_api::chat::{ChatLlmInteraction, ChatCompletionRequest, ChatCompletionResponse, Choice, ToolChoice};
use llm_api::tools::Tool;
use configuration::McpRuntimeConfig;
use crate::mcp_client::mcp_client::execute_tool_call_v2;
use crate::mcp_client::mcp_client::get_tools_list_v2;
use crate::mcp_client::mcp_client::initialize_mcp_client_v2;
use crate::mcp_tools::tools::define_all_tools;

#[derive(Clone, Debug)]
enum AgentState {
    Thinking,
    Executing(Choice),
    Evaluating(Choice, Vec<Message>),
    Correcting(String),
    Finished,
}

/// The `McpAgent` struct encapsulates the state and logic for the Multi-Controller Piper (MCP) agent.
#[derive(Clone)]
pub struct McpAgent {
    llm_interaction: ChatLlmInteraction,
    pub mcp_client: Arc<McpClient>,
    messages: Vec<Message>,
    llm_all_tool: Vec<Tool>,
    agent_mcp_config: McpRuntimeConfig,
    state: AgentState,
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
            state: AgentState::Thinking,
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

    async fn thinking_step(&mut self) -> anyhow::Result<AgentState> {
        info!("--- Thinking ---");
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

        let choice = response.choices[0].clone();

        if choice.finish_reason == self.agent_mcp_config.agent_mcp_finish_reason_tool_calls {
            Ok(AgentState::Executing(choice))
        } else {
            self.messages.push(Message {
                role: choice.message.role.clone(),
                content: choice.message.content.clone(),
                tool_calls: choice.message.tool_calls.clone(),
                tool_call_id: None,
            });
            Ok(AgentState::Finished)
        }
    }

    async fn executing_step(&mut self, choice: &Choice) -> anyhow::Result<AgentState> {
        info!("--- Executing ---");
        if let Some(tool_calls) = &choice.message.tool_calls {
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
            Ok(AgentState::Evaluating(choice.clone(), tool_results))
        } else {
            Ok(AgentState::Thinking)
        }
    }

    async fn evaluating_step(&mut self, choice: &Choice, tool_results: Vec<Message>) -> anyhow::Result<AgentState> {
        info!("--- Evaluating ---");
        let mut evaluation_messages = self.messages.clone();
        evaluation_messages.push(Message {
            role: choice.message.role.clone(),
            content: choice.message.content.clone(),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });
        evaluation_messages.extend(tool_results.clone());
        evaluation_messages.push(Message {
            role: "system".to_string(),
            content: Some(self.agent_mcp_config.agent_mcp_evaluation_prompt.clone()),
            tool_call_id: None,
            tool_calls: None,
        });

        let request_payload = ChatCompletionRequest {
            model: self.llm_interaction.model_id.clone(),
            messages: evaluation_messages,
            temperature: Some(0.0),
            max_tokens: Some(1024),
            top_p: Some(1.0),
            stop: None,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let response = self.call_api_v2(&request_payload).await?;
        
        if let Some(first_choice) = response.choices.get(0) {
            if let Some(content) = &first_choice.message.content {
                if content.contains("unsatisfactory") {
                    warn!("Tool execution unsatisfactory. Entering correction state.");
                    return Ok(AgentState::Correcting(content.clone()));
                }
            }
        }
        
        info!("Tool execution satisfactory.");
        self.messages.push(Message {
            role: choice.message.role.clone(),
            content: choice.message.content.clone(),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });
        self.messages.extend(tool_results);
        Ok(AgentState::Thinking)
    }

    async fn correcting_step(&mut self, issue: String) -> anyhow::Result<AgentState> {
        info!("--- Correcting ---");
        self.messages.push(Message {
            role: "system".to_string(),
            content: Some(format!("{}\n The issue was: {}", self.agent_mcp_config.agent_mcp_correction_prompt, issue)),
            tool_call_id: None,
            tool_calls: None,
        });
        Ok(AgentState::Thinking)
    }

    pub async fn execute_loop(&mut self) -> anyhow::Result<Option<Message>> {
        let mut final_message: Option<Message> = None;

        for loop_count in 0..self.agent_mcp_config.agent_mcp_max_loops {
            info!(
                "Agent Loop Iteration: {}/{} - State: {:?}",
                loop_count + 1,
                self.agent_mcp_config.agent_mcp_max_loops,
                self.state
            );

            let next_state = match self.state.clone() {
                AgentState::Thinking => self.thinking_step().await?,
                AgentState::Executing(choice) => self.executing_step(&choice).await?,
                AgentState::Evaluating(choice, tool_results) => self.evaluating_step(&choice, tool_results).await?,
                AgentState::Correcting(issue) => self.correcting_step(issue).await?,
                AgentState::Finished => break,
            };
            self.state = next_state;
        }
        
        if let Some(last_message) = self.messages.last() {
            if last_message.role == self.agent_mcp_config.agent_mcp_role_assistant {
                final_message = Some(last_message.clone());
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
        self.state = AgentState::Thinking;
        self.execute_loop().await
    }

    pub async fn submit_user_text(&self, user_text: String) -> Result<String> {
        info!("MCP Agent received user text: {}", user_text);
        Ok(format!("MCP agent processed: {}", user_text))
    }
}
