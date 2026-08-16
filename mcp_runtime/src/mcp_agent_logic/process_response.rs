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
                
                // this final message does not need to be logged
                //messages.push(final_message.clone());
               
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
            if let Some(tool_calls) = &choice.message.tool_calls {
                let tool_call_message = Message {
                    role: "assistant".to_string(),
                    content: choice.message.content.clone(), // Preserve content if any (e.g. reasoning or explanation)
                    tool_call_id: None,
                    tool_calls: Some(tool_calls.clone()),
                };
                
                // this assistant message requesting to make a call for tools needs to be recorded in message history
                messages.push(tool_call_message.clone());

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
            // Handle other finish reasons: capture content if available
            eprintln!("Unhandled finish reason: {}", choice.finish_reason);
            let final_msg = if let Some(content) = &choice.message.content {
                println!("Assistant Message (Partial/Error?): {}", content);
                Some(Message {
                    role: "assistant".to_string(),
                    content: Some(content.clone()),
                    tool_call_id: None,
                    tool_calls: None,
                })
            } else {
                None
            };
            AgentResponse {
                should_exit: true,
                nb_loop: loop_number,
                final_message: final_msg,
            }
        }
    }

    
}
