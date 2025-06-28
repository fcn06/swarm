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
    messages: &Vec<Message>,
) -> AgentResponse {
    match choice.finish_reason.as_str() {
        "stop" => {
            // Case 1: Model generated text response
            if let Some(content) = &choice.message.content {
                //println!("{}", content);
                let final_message = Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                    tool_call_id: None,
                };
                let mut new_messages = messages.clone();
                new_messages.push(final_message.clone());
                let agent_response = AgentResponse {
                    should_exit: true,
                    nb_loop: loop_number,
                    final_message: Some(final_message),
                };
                agent_response
            } else {
                //println!("Finish reason 'stop' but no content found.");
                AgentResponse {
                    should_exit: true,
                    nb_loop: loop_number,
                    final_message: None,
                }
            }
            //true // Exit loop
        }
        "tool_calls" => {
            // Case 2: Model requested tool calls
            //false
            AgentResponse {
                should_exit: false,
                nb_loop: loop_number,
                final_message: None,
            }
        }
        _ => {
            // Handle other finish reasons
            eprintln!("Unhandled finish reason: {}", choice.finish_reason);
            if let Some(content) = &choice.message.content {
                println!("Assistant Message (Partial/Error?): {}", content);
            }
            //true // Exit loop on unhandled reason
            AgentResponse {
                should_exit: true,
                nb_loop: loop_number,
                final_message: None,
            }
        }
    }
    // todo: return agent_response
}
