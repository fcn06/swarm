//use log::{debug, error, info, warn};

use llm_api::chat::{Choice, Message};

#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub should_exit: bool,
    pub nb_loop: u32,
    pub final_message: Option<Message>,
}

/// Process Response from LLM, whether it is final, or must be iterative
pub fn process_response(
    loop_number: u32,
    choice: &Choice,
   // messages: &Vec<Message>,
    messages: &mut Vec<Message>,
) -> AgentResponse {
    //let mut new_messages = messages.clone();

    match choice.finish_reason.as_str() {
        "stop" => {
            // Case 1: Model generated text response
            if let Some(content) = &choice.message.content {
                let final_message = Message {
                    role: "assistant".to_string(),
                    content: Some(content.clone()),
                    tool_call_id: None,
                    tool_calls: None,
                };
                //new_messages.push(final_message.clone());
               
                AgentResponse {
                    should_exit: true,
                    nb_loop: loop_number,
                    final_message: Some(final_message),
                }
            } else {
                AgentResponse {
                    should_exit: true,
                    nb_loop: loop_number,
                    final_message: None,
                }
            }
        }
        "tool_calls" => {
            // Case 2: Model requested tool calls
            // todo:Bug to be fixed : This message is not processed in message history
            if let Some(tool_calls) = &choice.message.tool_calls {
                let tool_call_message = Message {
                    role: "assistant".to_string(),
                    content: None, // Content is None for tool_calls messages
                    tool_call_id: None, // todo : register the right tool_call_id
                    tool_calls: Some(tool_calls.clone()),
                };
                //new_messages.push(tool_call_message.clone());
                messages.extend(vec!(tool_call_message.clone()));

                AgentResponse {
                    should_exit: false,
                    nb_loop: loop_number,
                    final_message: Some(tool_call_message),
                }
            } else {
                AgentResponse {
                    should_exit: true,
                    nb_loop: loop_number,
                    final_message: None,
                }
            }


           
        }
        _ => {
            // Handle other finish reasons
            eprintln!("Unhandled finish reason: {}", choice.finish_reason);
            if let Some(content) = &choice.message.content {
                println!("Assistant Message (Partial/Error?): {}", content);
            }
            AgentResponse {
                should_exit: true,
                nb_loop: loop_number,
                final_message: None,
            }
        }
    }

    
}
