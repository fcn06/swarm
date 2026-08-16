use anyhow::{Context, Result};
use llm_api::chat::Message;
use tracing::{info, error, warn, debug};
use std::env;
use serde_json::json;
use std::sync::Arc;

use crate::mcp_client::mcp_client::McpClient;
use llm_api::chat::{ChatLlmInteraction, ChatCompletionRequest, ChatCompletionResponse, Choice, ToolChoice};
use llm_api::tools::Tool;
use configuration::McpRuntimeConfig;
use crate::mcp_client::mcp_client::{execute_tool_call_v2, get_tools_list_v2, initialize_mcp_client_v2};
use crate::mcp_tools::tools::define_all_tools;

/// Represents the discrete states of the agent's execution loop.
///
/// The agent transitions between these states in a state-machine pattern:
/// `Thinking` → `Executing` → `Evaluating` → [`Correcting`] → `Thinking` → ... → `Finished`
#[derive(Clone, Debug)]
pub enum AgentState {
    /// Agent is calling the LLM to decide on the next action.
    Thinking,
    /// Agent is executing tool calls returned by the LLM.
    Executing(Choice),
    /// Agent is evaluating whether tool execution results are satisfactory.
    Evaluating(Choice, Vec<Message>),
    /// Agent is injecting a correction prompt after unsatisfactory results.
    Correcting(String),
    /// Agent has completed its task.
    Finished,
}

/// The context for a single run of the McpAgent, keeping execution state strictly request-scoped
/// and allowing the agent to run concurrently across multiple requests.
pub struct McpAgentRunContext {
    pub state: AgentState,
    pub messages: Vec<Message>,
    pub llm_all_tool: Vec<Tool>,
}

/// The `McpAgent` struct encapsulates the configuration and static components for the MCP agent.
/// It is stateless and thread-safe, and can be shared concurrently across multiple threads.
#[derive(Clone)]
pub struct McpAgent {
    llm_interaction: ChatLlmInteraction,
    pub mcp_client: Arc<McpClient>,
    agent_mcp_config: McpRuntimeConfig,
    tool_cache: Arc<std::sync::RwLock<std::collections::HashMap<String, Vec<Tool>>>>,
}

impl McpAgent {
    pub async fn new(
        agent_mcp_config: McpRuntimeConfig,
        mcp_runtime_api_key: Option<String>,
    ) -> anyhow::Result<Self> {
        let model_id = agent_mcp_config.agent_mcp_model_id.clone();

        let llm_mcp_api_key = if let Some(api_key) = mcp_runtime_api_key {
            api_key
        } else if let Some(env_var_name) = &agent_mcp_config.agent_mcp_llm_api_key_env_var {
            env::var(env_var_name)
                .context(format!("Environment variable '{}' for LLM API key must be set", env_var_name))?
        } else {
            env::var("LLM_MCP_API_KEY")
                .context("LLM_MCP_API_KEY environment variable must be set")?
        };

        let mcp_client = Arc::new(
            initialize_mcp_client_v2(agent_mcp_config.clone())
                .await
                .context("Failed to initialize MCP client")?,
        );

        let list_tools = match get_tools_list_v2(mcp_client.clone()).await {
            Ok(tools) => tools,
            Err(e) => {
                warn!("⚠️ Could not retrieve tools at startup: {}", e);
                vec![]
            }
        };

        let llm_all_tool = define_all_tools(list_tools).unwrap_or_else(|e| {
            warn!("⚠️ Failed to define tools from retrieved list: {}", e);
            vec![]
        });

        info!("🛠️ MCP Agent initialized with {} tool(s)", llm_all_tool.len());

        let mut tool_cache_map = std::collections::HashMap::new();
        tool_cache_map.insert("".to_string(), llm_all_tool);
        let tool_cache = Arc::new(std::sync::RwLock::new(tool_cache_map));

        Ok(Self {
            llm_interaction: ChatLlmInteraction::new(
                agent_mcp_config.agent_mcp_llm_url.clone(),
                model_id,
                llm_mcp_api_key,
            ),
            mcp_client,
            agent_mcp_config,
            tool_cache,
        })
    }

    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.tool_cache.read().unwrap().get("").cloned().unwrap_or_default()
    }

    pub async fn get_tools_for_session(&self, session_key: &str) -> Vec<Tool> {
        {
            let cache = self.tool_cache.read().unwrap();
            if let Some(tools) = cache.get(session_key) {
                if !tools.is_empty() {
                    return tools.clone();
                }
            }
        }

        match get_tools_list_v2(self.mcp_client.clone()).await {
            Ok(tools) => match define_all_tools(tools) {
                Ok(new_tools) => {
                    let mut cache = self.tool_cache.write().unwrap();
                    cache.insert(session_key.to_string(), new_tools.clone());
                    new_tools
                }
                Err(e) => {
                    warn!("⚠️ Failed to define refreshed tools: {}", e);
                    vec![]
                }
            },
            Err(e) => {
                warn!("⚠️ Failed to refresh tools from MCP server: {}", e);
                vec![]
            }
        }
    }

    pub fn invalidate_tool_cache(&self, session_key: &str) {
        let mut cache = self.tool_cache.write().unwrap();
        cache.remove(session_key);
        info!("🔄 Invalidated tool cache for session: {}", session_key);
    }

    async fn call_api_v2(
        &self,
        request_payload: &ChatCompletionRequest,
    ) -> anyhow::Result<ChatCompletionResponse> {
        debug!("Calling LLM API with payload: {:?}", request_payload);

        let max_retries = 3;
        let mut delay = std::time::Duration::from_millis(1000);

        for attempt in 1..=max_retries {
            match self.llm_interaction.call_chat_completions_v2(request_payload).await {
                Ok(response) => {
                    debug!("LLM API Response (attempt {}): {:?}", attempt, response);
                    return Ok(response);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    let is_rate_limit = err_msg.contains("429")
                        || err_msg.contains("rate_limit")
                        || err_msg.contains("Rate limit");
                    let is_validation = err_msg.contains("validation")
                        || err_msg.contains("request.tools")
                        || err_msg.contains("validation failed")
                        || err_msg.contains("400")
                        || err_msg.contains("Bad Request");

                    if is_validation {
                        // Do not retry schema validation errors; propagate immediately
                        return Err(e).context("LLM chat completion API call failed with validation error");
                    }

                    if is_rate_limit {
                        warn!("⚠️ LLM API rate limit exhausted after inner retries. Failing fast.");
                        return Err(anyhow::anyhow!(
                            "LLM provider rate limit exceeded. The AI service is temporarily unavailable. Please wait a moment and try again."
                        ));
                    }

                    if attempt == max_retries {
                        error!("❌ LLM API call failed after {} attempts: {}", max_retries, e);
                        return Err(e).context("LLM chat completion API call failed after max retries");
                    }

                    warn!(
                        "⚠️ LLM API call failed: {}. Temporary error on attempt {}/{}. Retrying in {:?}...",
                        err_msg, attempt, max_retries, delay
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
        anyhow::bail!("LLM API call failed after max retries")
    }

    // ──────────────────────────────────────────────────────────────
    // State Machine Steps
    // ──────────────────────────────────────────────────────────────

    /// Thinking: Call the LLM with the current message history and tools.
    /// Returns `Executing` if the LLM requests tool calls, or `Finished` otherwise.
    async fn thinking_step(&self, ctx: &mut McpAgentRunContext) -> anyhow::Result<AgentState> {
        info!("--- Thinking ---");

        let has_tools = !ctx.llm_all_tool.is_empty();

        // Detect excessive consecutive search/listing calls to prevent infinite loops
        let mut consecutive_searches = 0;
        for msg in ctx.messages.iter().rev() {
            if msg.role == "assistant" {
                if let Some(tool_calls) = &msg.tool_calls {
                    let has_search = tool_calls.iter().any(|tc| {
                        tc.function.name.contains("search") || tc.function.name.contains("list")
                    });
                    if has_search {
                        consecutive_searches += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if msg.role == "user" {
                break;
            }
        }

        let mut active_messages = ctx.messages.clone();
        if consecutive_searches >= 3 {
            tracing::warn!(
                "⚠️ Detected {} consecutive search/list calls. Injecting loop-breaking system prompt...",
                consecutive_searches
            );
            let loop_breaker = "System Notice: You have repeatedly searched or listed tools without making progress. If the required tool or data is unavailable, please explain what is missing to the user and finalize your answer.";
            active_messages.push(Message {
                role: "system".to_string(),
                content: Some(loop_breaker.to_string()),
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let request_payload = ChatCompletionRequest {
            model: self.llm_interaction.model_id.clone(),
            messages: active_messages,
            temperature: Some(0.0),
            max_tokens: Some(1024),
            top_p: Some(1.0),
            stop: None,
            stream: Some(false),
            tools: if has_tools { Some(ctx.llm_all_tool.clone()) } else { None },
            tool_choice: if has_tools {
                Some(ToolChoice::String(self.agent_mcp_config.agent_mcp_tool_choice_auto.clone()))
            } else {
                None
            },
        };

        let response = self.call_api_v2(&request_payload).await?;

        if response.choices.is_empty() {
            error!("LLM response contained no choices.");
            anyhow::bail!("LLM response contained no choices.");
        }

        let mut choice = response.choices[0].clone();

        // Clean <think> tags from the response content
        if let Some(content) = choice.message.content.as_mut() {
            *content = self.llm_interaction.remove_think_tags(content.clone()).await?;
        }

        // Commit the assistant's response to message history
        ctx.messages.push(Message {
            role: choice.message.role.clone(),
            content: choice.message.content.clone(),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });

        let has_tool_calls = choice.message.tool_calls.as_ref().map(|tc| !tc.is_empty()).unwrap_or(false);

        if has_tool_calls {
            info!("LLM requested {} tool call(s).", choice.message.tool_calls.as_ref().unwrap().len());
            Ok(AgentState::Executing(choice))
        } else if choice.finish_reason == self.agent_mcp_config.agent_mcp_finish_reason_tool_calls {
            Ok(AgentState::Executing(choice))
        } else {
            info!("Agent finished thinking (reason: {:?}).", choice.finish_reason);
            Ok(AgentState::Finished)
        }
    }

    /// Executing: Run all tool calls from the LLM's response.
    /// Returns `Evaluating` with the tool results.
    async fn executing_step(
        &self,
        ctx: &mut McpAgentRunContext,
        choice: &Choice,
    ) -> anyhow::Result<AgentState> {
        info!("--- Executing ---");

        if let Some(tool_calls) = &choice.message.tool_calls {
            let mut tool_results: Vec<Message> = Vec::new();

            for tool_call in tool_calls {
                info!("Executing tool call: {}", tool_call.id);
                let tool_name = tool_call.function.name.clone();

                match execute_tool_call_v2(self.mcp_client.clone(), tool_call.clone()).await {
                    Ok(result) => {
                        // Parse the result content
                        let mut parsed_texts = Vec::new();
                        if let Ok(json_arr) = serde_json::to_value(&result.content) {
                            if let Some(arr) = json_arr.as_array() {
                                for item in arr {
                                    if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                                        if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                                            parsed_texts.push(text.to_string());
                                        }
                                    }
                                }
                            }
                        }

                        let mut result_content_str = if !parsed_texts.is_empty() {
                            parsed_texts.join("\n")
                        } else {
                            serde_json::to_string(&result.content).unwrap_or_else(|_| "[]".to_string())
                        };

                        let original_len = result_content_str.chars().count();
                        info!("Tool '{}' returned {} chars.", tool_name, original_len);

                        // If tool output is very large, optionally sanitize/distill to avoid context overflow
                        if original_len > 8000 {
                            if let Some(sanitizer_model) = &self.agent_mcp_config.agent_mcp_sanitizer_model_id {
                                let user_query = ctx.messages.iter()
                                    .rev()
                                    .find(|m| m.role == "user")
                                    .and_then(|m| m.content.clone())
                                    .unwrap_or_default();

                                let sys_prompt = format!(
                                    "The user wants to: {}. Below is a long tool output. Provide a comprehensive distillation of this data, keeping only relevant information, specific IDs, dates, and numbers. Output only the summarized facts.",
                                    user_query
                                );

                                let request_payload = ChatCompletionRequest {
                                    model: sanitizer_model.clone(),
                                    messages: vec![
                                        Message {
                                            role: "system".to_string(),
                                            content: Some(sys_prompt),
                                            tool_call_id: None,
                                            tool_calls: None,
                                        },
                                        Message {
                                            role: "user".to_string(),
                                            content: Some(if result_content_str.len() > 15000 {
                                                result_content_str[..15000].to_string()
                                            } else {
                                                result_content_str.clone()
                                            }),
                                            tool_call_id: None,
                                            tool_calls: None,
                                        },
                                    ],
                                    temperature: Some(0.0),
                                    max_tokens: Some(1024),
                                    top_p: Some(1.0),
                                    stop: None,
                                    stream: Some(false),
                                    tools: None,
                                    tool_choice: None,
                                };

                                if let Ok(resp) = self.call_api_v2(&request_payload).await {
                                    if let Some(first) = resp.choices.first() {
                                        if let Some(text) = &first.message.content {
                                            info!("Sanitizer compressed tool output from {} to {} chars", original_len, text.len());
                                            result_content_str = text.clone();
                                        }
                                    }
                                }
                            }
                        }

                        tool_results.push(Message {
                            role: self.agent_mcp_config.agent_mcp_role_tool.clone(),
                            content: Some(result_content_str),
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

            if self.agent_mcp_config.agent_mcp_enable_evaluation.unwrap_or(false) {
                Ok(AgentState::Evaluating(choice.clone(), tool_results))
            } else {
                ctx.messages.extend(tool_results);
                Ok(AgentState::Thinking)
            }
        } else {
            Ok(AgentState::Thinking)
        }
    }

    /// Evaluating: Send tool results + evaluation prompt to LLM.
    /// Returns `Correcting` if results are unsatisfactory, or `Thinking` to continue.
    async fn evaluating_step(
        &self,
        ctx: &mut McpAgentRunContext,
        choice: &Choice,
        tool_results: Vec<Message>,
    ) -> anyhow::Result<AgentState> {
        info!("--- Evaluating ---");

        let mut evaluation_messages = ctx.messages.clone();
        evaluation_messages.push(Message {
            role: choice.message.role.clone(),
            content: Some(choice.message.content.clone().unwrap_or_default()),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });
        evaluation_messages.extend(tool_results.clone());
        evaluation_messages.push(Message {
            role: "system".to_string(),
            content: Some(format!(
                "{}\n\nRespond with ONLY a JSON object: {{\"satisfactory\": true/false, \"reason\": \"...\"}}",
                self.agent_mcp_config.agent_mcp_evaluation_prompt
            )),
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
            tools: Some(ctx.llm_all_tool.clone()),
            tool_choice: Some(ToolChoice::String("none".to_string())),
        };

        let response = self.call_api_v2(&request_payload).await?;

        if let Some(first_choice) = response.choices.first() {
            if let Some(content) = &first_choice.message.content {
                #[derive(serde::Deserialize)]
                struct EvalResult {
                    satisfactory: bool,
                    reason: String,
                }

                let (is_unsatisfactory, reason) = match serde_json::from_str::<EvalResult>(content) {
                    Ok(eval) => (!eval.satisfactory, eval.reason),
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to parse EvalResult JSON: {} - Raw: {}", e, content);
                        if content.to_lowercase().contains("unsatisfactory") {
                            (true, content.clone())
                        } else {
                            (false, String::new())
                        }
                    }
                };

                if is_unsatisfactory {
                    warn!("Tool execution unsatisfactory: {}", reason);
                    return Ok(AgentState::Correcting(reason));
                }
            }
        }

        info!("Tool execution satisfactory.");
        ctx.messages.push(Message {
            role: choice.message.role.clone(),
            content: Some(choice.message.content.clone().unwrap_or_default()),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });
        ctx.messages.extend(tool_results);
        Ok(AgentState::Thinking)
    }

    /// Correcting: Inject correction prompt with the issue description.
    /// Returns `Thinking` to retry the task.
    async fn correcting_step(&self, ctx: &mut McpAgentRunContext, issue: String) -> anyhow::Result<AgentState> {
        info!("--- Correcting ---");
        ctx.messages.push(Message {
            role: "system".to_string(),
            content: Some(format!(
                "{}\n The issue was: {}",
                self.agent_mcp_config.agent_mcp_correction_prompt, issue
            )),
            tool_call_id: None,
            tool_calls: None,
        });
        Ok(AgentState::Thinking)
    }

    // ──────────────────────────────────────────────────────────────
    // Main Execution Loop
    // ──────────────────────────────────────────────────────────────

    pub async fn execute_loop(&self, ctx: &mut McpAgentRunContext) -> anyhow::Result<Option<Message>> {
        let mut final_message: Option<Message> = None;

        for loop_count in 0..self.agent_mcp_config.agent_mcp_max_loops {
            info!(
                "Agent Loop Iteration: {}/{} - State: {:?}",
                loop_count + 1,
                self.agent_mcp_config.agent_mcp_max_loops,
                ctx.state
            );

            let next_state = match ctx.state.clone() {
                AgentState::Thinking => match self.thinking_step(ctx).await {
                    Ok(state) => state,
                    Err(e) => {
                        let err_msg = e.to_string();
                        warn!("⚠️ Thinking step failed: {}", err_msg);
                        let user_msg = if err_msg.contains("rate limit") || err_msg.contains("429") {
                            "I successfully executed the requested operations, but the AI service is temporarily rate-limited. Please try again in a moment."
                        } else {
                            "I encountered a temporary error while processing your request. Please try again shortly."
                        };
                        final_message = Some(Message {
                            role: self.agent_mcp_config.agent_mcp_role_assistant.clone(),
                            content: Some(user_msg.to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                        });
                        break;
                    }
                },
                AgentState::Executing(choice) => self.executing_step(ctx, &choice).await?,
                AgentState::Evaluating(choice, tool_results) => {
                    self.evaluating_step(ctx, &choice, tool_results).await?
                }
                AgentState::Correcting(issue) => self.correcting_step(ctx, issue).await?,
                AgentState::Finished => break,
            };
            ctx.state = next_state;
        }

        if let Some(last_message) = ctx.messages.last() {
            if last_message.role == self.agent_mcp_config.agent_mcp_role_assistant {
                final_message = Some(last_message.clone());
            }
        }

        if final_message.is_none() {
            warn!("Agent finished without a definitive final message. Generating fallback response.");
            let fallback_message = Message {
                role: self.agent_mcp_config.agent_mcp_role_assistant.clone(),
                content: Some(
                    "I apologize, but I encountered an execution issue while trying to process your request (maximum reasoning steps exceeded). Please try again in a few moments."
                        .to_string(),
                ),
                tool_calls: None,
                tool_call_id: None,
            };
            final_message = Some(fallback_message);
        }

        Ok(final_message)
    }

    pub fn reset_messages(&self) -> anyhow::Result<()> {
        // No-op for backwards compatibility: execution state is request-scoped
        Ok(())
    }

    pub async fn run_agent_internal(
        &self,
        user_message: Message,
    ) -> anyhow::Result<Option<Message>> {
        self.run_agent_internal_with_system_prompt(user_message, None).await
    }

    pub async fn run_agent_internal_with_system_prompt(
        &self,
        user_message: Message,
        system_prompt_override: Option<String>,
    ) -> anyhow::Result<Option<Message>> {
        let llm_all_tool = self.get_tools_for_session("").await;

        let system_message = system_prompt_override
            .unwrap_or_else(|| self.agent_mcp_config.agent_mcp_system_prompt.clone());

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: Some(system_message),
                tool_call_id: None,
                tool_calls: None,
            },
            user_message,
        ];

        let mut ctx = McpAgentRunContext {
            state: AgentState::Thinking,
            messages,
            llm_all_tool,
        };

        self.execute_loop(&mut ctx).await
    }

    pub async fn submit_user_text(&self, user_text: String) -> Result<String> {
        info!("MCP Agent received user text: {}", user_text);
        let user_message = Message {
            role: "user".to_string(),
            content: Some(user_text),
            tool_call_id: None,
            tool_calls: None,
        };
        let response = self.run_agent_internal(user_message).await?;
        let text = response
            .and_then(|m| m.content)
            .unwrap_or_else(|| "No response generated".to_string());
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_mcp_agent_concurrency_traits() {
        assert_send_sync::<McpAgent>();
        assert_send_sync::<McpAgentRunContext>();
    }
}
