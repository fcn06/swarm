use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value; // Import Value for flexible parameters
use std::env;

use crate::tools::Tool;
use anyhow::{Result,Context};

#[derive(Clone)]
pub struct ChatLlmInteraction {
    pub client: Client,
    pub llm_url:String,
    llm_api_key:String,
    pub model_id:String,

}


#[derive(Serialize, Debug, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>, // Keep existing message structure for history

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    // --- Tool Calling Additions (Request) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    // --- End Tool Calling Additions ---
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,    // "system", "user", "assistant", or "tool"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>, // Content for system/user/assistant, or result for tool

    // --- Tool Calling Additions (for Tool Result Message) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>, // Only used when role is "tool"
    // --- End Tool Calling Additions ---

    // --- Add tool_calls to Message for assistant messages ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)] // Handles string ("none", "auto") or object variants
pub enum ToolChoice {
    String(String), // Represents "none" or "auto"
    Function {
        r#type: String, // Should be "function"
        function: FunctionName,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionName {
    pub name: String,
}

// --- Structs for Response ---

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_fingerprint: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage, // Use the modified message struct
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<Value>, // Or specific struct if needed
    pub finish_reason: String,    // "stop", "length", "tool_calls", etc.
}

// --- Modified Response Message & Tool Call Structs (Response) ---
#[derive(Deserialize, Debug, Clone)]
pub struct ResponseMessage {
    pub role: String, // "assistant"
    // Content might be null if tool_calls is present
    pub content: Option<String>,
    // Tool calls requested by the model
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,     // ID to be sent back in the tool result message
    pub r#type: String, // Typically "function"
    pub function: FunctionCall,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    // Arguments is a STRING containing JSON, needs parsing
    pub arguments: String,
}
// --- End Tool Call Structs ---

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// --- API Call Function ---

impl ChatLlmInteraction {
    /// Create a new llm interaction entity
    pub fn new(llm_url:String,model_id:String,llm_api_key:String) -> Self {
        
        Self {
            client:reqwest::Client::new(),
            llm_url:llm_url,
            llm_api_key:llm_api_key,
            model_id:model_id,
        }
    }


/// for complex calls using tools, this is the api to use
pub async fn call_chat_completions_v2(
    &self,
    request_payload: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse, reqwest::Error> {
    //let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY must be set");
    
    let response = self.client
        .post(self.llm_url.clone())
        .bearer_auth(self.llm_api_key.clone())
        .header("Content-Type", "application/json; charset=utf-8")
        .json(request_payload)
        .send()
        .await?;

    // Check for HTTP errors first
    response.error_for_status_ref()?;

    // Then deserialize the successful response
    let response_body = response.json::<ChatCompletionResponse>().await?;

    Ok(response_body)
}

/// for very simple calls without tools, one can use this simpler api
pub async fn call_api_simple(
    &self,
    agent_role:String,
    user_query:String,
) -> anyhow::Result<Option<Message>> {

    let messages_origin = vec![Message {
        role: agent_role,
        content: Some(user_query),
        tool_call_id: None,
        tool_calls:None
    }];

    self.call_api_message(messages_origin).await

}


pub async fn call_api_message(
    &self,
    messages:  Vec<Message>,
) -> anyhow::Result<Option<Message>> {

    //let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY must be set");
    

    let llm_request_payload = ChatCompletionRequest {
        model: self.model_id.clone(),
        messages: messages.clone(),
        // Add other parameters like temperature if needed
        temperature: Some(0.7),
        tool_choice: None,
        max_tokens: None,
        top_p: None,
        stop: None,
        stream: None,
        tools: None,
    };

    let llm_response = self.call_chat_completions_v2( &llm_request_payload)
    .await
    .context("LLM API request failed during plan creation")?;

    let response_content = llm_response
        .choices
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("LLM response missing choices"))?
        .message
        .content
        .clone();

        
        // remove think tags from llm response
        let response_content = Some(
            self.remove_think_tags(response_content.clone().expect("REASON"))
                .await?,
        );

        /* 
        println!(
            "A2A Agent: LLM responded with plan content:{:?}",
            response_content
        );
        */

        let response_message = Some(Message {
            role: "assistant".to_string(),
            content: Some(response_content.expect("Incorrect Answer")),
            tool_call_id: None,
            tool_calls:None
        });
    
        Ok(response_message)
}

// Helper function to extract text from a Message
pub async fn remove_think_tags( &self,result: String) -> anyhow::Result<String> {
    let mut cleaned_result = String::new();
    let mut in_think_tag = false;

    // in case llm returns json markdown
    let result_clean= if result.contains("```json") {
        let result_1=result.replace("```json", "");
        result_1.replace("```", "")
    } else {result};
    let result=result_clean;

    // in case LLM return <think> and </think> tags
    for line in result.lines() {
        if line.contains("<think>") {
            in_think_tag = true;
        }
        if line.contains("</think>") {
            in_think_tag = false;
            continue;
        } // Continue to avoid adding the </think> line itself
        if !in_think_tag {
            cleaned_result.push_str(line);
            cleaned_result.push('\n');
        }
    }
    Ok(cleaned_result)
}

}