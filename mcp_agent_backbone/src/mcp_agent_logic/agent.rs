//use log::{debug, error, info,trace, warn};
use tracing::{info, error,warn,trace, Level, debug};

use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::sync::Arc;

use rmcp::RoleClient;
use rmcp::model::InitializeRequestParam;
use rmcp::service::RunningService;

use llm_api::chat::{call_chat_completions_v2};
use llm_api::chat::{ChatCompletionRequest, ChatCompletionResponse, Choice, Message, ToolChoice};
use llm_api::tools::Tool;

use crate::mcp_initialization::mcp_agent_config::RuntimeMcpConfigProject;
use configuration::AgentMcpConfig;

use crate::mcp_agent_logic::process_response::process_response;
use crate::mcp_client::mcp_client::execute_tool_call_v2;

/// Holds the state required for the agent execution loop.
struct AgentState {
    http_client: Client,
    mcp_client: Arc<RunningService<RoleClient, InitializeRequestParam>>,
    messages: Vec<Message>,
    llm_all_tool: Vec<Tool>,
    model_id: String,
    config: AgentMcpConfig, // Keep config struct here
}


/// Calls the external LLM API.
async fn call_api_v2(
    client: &Client,
    request_payload: &ChatCompletionRequest,
    api_url:String,
) -> Result<ChatCompletionResponse, reqwest::Error> {
    debug!("Calling LLM API with payload: {:?}", request_payload);
    //trace!("Request Payload:{:#?}",request_payload.clone());

    // Serialize it to a JSON string.
    let payload_json = serde_json::to_string(&request_payload.clone()).unwrap();
    trace!("{}",payload_json);

    let response = call_chat_completions_v2(client, request_payload,api_url).await;
    debug!("LLM API Response: {:?}", response);
    response
}

/// Handles the execution of tool calls requested by the LLM.
async fn handle_tool_calls(
    mcp_client: Arc<RunningService<RoleClient, InitializeRequestParam>>,
    choice: &Choice,
    messages: &mut Vec<Message>,
    config: &AgentMcpConfig, // Pass config reference
) -> Result<(), Box<dyn Error>> {
    if let Some(tool_calls) = &choice.message.tool_calls {
        info!("--- Tool Calls Requested ---");
        let mut tool_results: Vec<Message> = Vec::new();

        for tool_call in tool_calls {
            info!("Executing tool call: {}", tool_call.id);
            
            match execute_tool_call_v2(mcp_client.clone(), tool_call.clone()).await {
                Ok(result) => {
                    let result_content_str =
                        serde_json::to_string(&result.content).map_err(|e| {
                            error!("Failed to serialize tool result content: {}", e);
                            Box::new(e) as Box<dyn Error>
                        })?;

                    tool_results.push(Message {
                        role: config.agent_mcp_role_tool.clone(),
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
                        role: config.agent_mcp_role_tool.clone(),
                        content: Some(error_content.to_string()),
                        tool_call_id: Some(tool_call.id.clone()),
                        tool_calls: None,
                    });
                }
            }
        }
        messages.extend(tool_results);
    }
    Ok(())
}

/// Executes the main agent logic loop.
async fn execute_loop(state: &mut AgentState) -> Result<Option<Message>, Box<dyn Error>> {
    let mut final_message: Option<Message> = None;

    for loop_count in 0..state.config.agent_mcp_max_loops {
        info!(
            "Agent Loop Iteration: {}/{} ",
            loop_count + 1,
            state.config.agent_mcp_max_loops
        );

        let request_payload = ChatCompletionRequest {
            model: state.model_id.clone(),
            messages: state.messages.clone(),
            temperature: Some(0.0),
            max_tokens: Some(1024),
            top_p: Some(1.0),
            stop: None,
            stream: Some(false),
            tools: Some(state.llm_all_tool.clone()),
            tool_choice: Some(ToolChoice::String(
                state.config.agent_mcp_tool_choice_auto.clone(),
            )),
        };

        let response = call_api_v2(&state.http_client, &request_payload,state.config.agent_mcp_llm_url.clone()).await?;

        if response.choices.is_empty() {
            error!("LLM response contained no choices.");
            return Err("LLM response contained no choices.".into());
        }
        let choice = &response.choices[0];

        // Pass relevant config parts if needed (e.g., state.config.role_assistant)
        let agent_response = process_response(loop_count, choice, &mut state.messages);


        if agent_response.should_exit {
            final_message = agent_response.final_message;
            info!("Agent loop exiting based on process_response decision.");
            break;
        }

        match choice.finish_reason.as_str() {
            reason if reason == state.config.agent_mcp_finish_reason_tool_calls => {
                handle_tool_calls(
                    state.mcp_client.clone(),
                    choice,
                    &mut state.messages,
                    &state.config,
                )
                .await?;
                // For some LLM , once you have used the tools, it makes sense to remove the tools from the request
                // It is a bad practice, and for some LLM it is not necesary
                //error!("Deleting Tools List : ({:#?})", state.llm_all_tool);
                //state.llm_all_tool.clear();
            }
            reason if reason == state.config.agent_mcp_finish_reason_stop => {
                info!("LLM indicated finish_reason 'stop'.");
                if final_message.is_none() {
                    // Optionally capture the last message from choice.message if needed, using config roles
                    if choice.message.role == state.config.agent_mcp_role_assistant {
                        final_message = Some(Message {
                            role: state.config.agent_mcp_role_assistant.clone(),
                            content: choice.message.content.clone(),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        warn!(
                            "Finish reason was 'stop', but no final message captured by process_response. Captured last assistant message."
                        );
                    } else {
                        warn!(
                            "Finish reason was 'stop', but no final message captured and last message role ('{}') wasn't assistant.",
                            choice.message.role
                        );
                    }
                }
                break;
            }
            other_reason => {
                warn!("Unhandled finish reason '{}', exiting loop.", other_reason);
                if let Some(content) = &choice.message.content {
                    if !content.trim().is_empty() && final_message.is_none() {
                        // Use configured assistant role when capturing fallback message
                        final_message = Some(Message {
                            role: state.config.agent_mcp_role_assistant.clone(),
                            content: Some(content.clone()),
                            tool_call_id: None, // Assuming no tool call ID for this fallback
                            tool_calls: None,
                        });
                        info!(
                            "Captured last assistant message due to unhandled finish reason '{}'",
                            other_reason
                        );
                    }
                } else {
                    final_message = Some(Message {
                        role: state.config.agent_mcp_role_assistant.clone(),
                        content: None,
                        tool_call_id: None,
                        tool_calls: None,
                    });
                    info!(
                        "Captured empty assistant message due to unhandled finish reason '{}'",
                        other_reason
                    );
                }
                break;
            }
        }

        if loop_count == state.config.agent_mcp_max_loops - 1 {
            info!("Maximum loop count reached.");
            error!(
                "Maximum loop count ({}) reached, exiting.",
                state.config.agent_mcp_max_loops
            );
            return Err(format!(
                "Agent exceeded maximum iterations ({})",
                state.config.agent_mcp_max_loops
            )
            .into());
        }
    } // End of loop

    if final_message.is_none() {
        warn!("Agent finished without a definitive final message.");
    }

    Ok(final_message)
}

/// Initializes and runs the agent.
// Now accepts AgentConfig as a parameter
pub async fn run_agent(
    config_project: RuntimeMcpConfigProject,
    agent_config: AgentMcpConfig, // Accept AgentConfig directly
    user_message: Message,
) -> Result<Option<Message>, Box<dyn Error>> {
    let mut messages = config_project.init_messages;
    messages.push(user_message);

    // Create agent state using the passed-in config
    let mut agent_state = AgentState {
        http_client: config_project.http_client,
        mcp_client: config_project.mcp_client,
        messages,
        llm_all_tool: config_project.llm_all_tool,
        model_id: config_project.model_id,
        config: agent_config, // Use the passed-in agent_config
    };

    // Run the loop
    execute_loop(&mut agent_state).await
}
