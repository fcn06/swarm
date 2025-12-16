use async_trait::async_trait;
use uuid::Uuid;
use configuration::AgentConfig;

use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;

use llm_api::chat::{ChatLlmInteraction};
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::debug;

use serde_json::Map;
use serde_json::Value;

#[allow(unused)]
use anyhow::Context;

// todo : change the prompt of mcp runtime , so that he tries to use internal knowledge if possible
// todo: see if the method of delegation to mcp_runtime is optimal
use mcp_runtime::mcp_agent_logic::agent::McpAgent;

use llm_api::chat::Message as LlmMessage;

use agent_core::business_logic::agent::{Agent};
use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};


use agent_models::execution::execution_result::{ExecutionResult};

use agent_core::business_logic::services::WorkflowServiceApi;

/// Modern A2A server setup 
#[derive(Clone)]
pub struct IdentityAgent {
    llm_interaction: ChatLlmInteraction,
    mcp_agent:Option<Arc<Mutex<McpAgent>>>,
    system_message: String, // Store the specific prompt
    // authentication_service: Arc<dyn AuthenticationService>, // Placeholder for actual authentication service
}

#[async_trait]
impl Agent for IdentityAgent {

    /// Creation of a new identity agent
    async fn new(
        agent_config: AgentConfig,
        agent_api_key:String,
        mcp_runtime_details: Option<McpRuntimeDetails>,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {

               // Set model to be used
        let model_id = agent_config.agent_model_id();

        // Set system message to be used, ensuring it's present in the config
        let system_message = agent_config.agent_system_prompt().expect("agent_system_prompt not found in config");

        // Set API key for LLM
        let llm_a2a_api_key =agent_api_key;

        let llm_interaction= ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            model_id,
            llm_a2a_api_key,
        );

        let mcp_agent = if let Some(details) = mcp_runtime_details {
            // Case 1: McpRuntimeDetails struct provided directly
            Some(Arc::new(Mutex::new(McpAgent::new(details.config, Some(details.api_key)).await?)))
        } else if let Some(path) = agent_config.agent_mcp_config_path() {
            // Case 2: MCP config path specified in AgentConfig (API key from environment)
            let agent_mcp_config = configuration::McpRuntimeConfig::load_agent_config(path.as_str())
                .context("Error loading MCP config for identity agent from agent_config.agent_mcp_config_path")?;
            let mcp_agent = McpAgent::new(agent_mcp_config, None).await?; // Pass None for API key to use environment variable
            Some(Arc::new(Mutex::new(mcp_agent)))
        } else {
            // Case 3: No MCP config provided in any way
            None
        };

          Ok(Self {
            llm_interaction,
            mcp_agent,
            system_message,
          })
    }


    

        /// business logic for handling user request with authentication
        async fn handle_request(&self, request: LlmMessage,_metadata:Option<Map<String, Value>>) ->anyhow::Result<ExecutionResult> {
       
            let request_id=uuid::Uuid::new_v4().to_string();
            let conversation_id = Uuid::new_v4().to_string();
     
            // Placeholder for authentication check
            // In a real scenario, this would involve validating tokens, checking permissions, etc.
            let is_authenticated = if let Some(metadata) = _metadata {
                metadata.get("authenticated").and_then(|v| v.as_bool()).unwrap_or(false)
            } else {
                false
            };

            if !is_authenticated {
                return Ok(ExecutionResult {
                    request_id,
                    conversation_id,
                    success: false,
                    output: Value::String("Authentication required.".to_string()),
                });
            }

             // use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
             let response =if self.mcp_agent.is_none() {
                let messages = vec![
                    LlmMessage {
                        role: "system".to_string(),
                        content: Some(self.system_message.clone()),
                        tool_call_id: None,
                        tool_calls: None,
                    },
                    request.clone(), // Use the user's request
                ];
                self.llm_interaction.call_api(messages, None, None).await?

                 } else {
                     let mut locked_mcp_agent = self.mcp_agent.as_ref().unwrap().lock().await;
                     locked_mcp_agent.run_agent_internal(request.clone())
                     .await
                     // todo : make it more robust
                     .unwrap()
                 };
     
                 let llm_content=response.expect("No Return from LLM").content.expect("Empty result from Llm");
     
                 let output_value = match serde_json::from_str::<Value>(&llm_content) {
                    Ok(json_val) => json_val,
                    Err(_) => Value::String(llm_content),
                };

                debug!("Output Value from Identity Agent: {:?}", output_value);
     
             Ok(ExecutionResult {
                    request_id,
                    conversation_id,
                    success: true, // Mark as not fully successful if summarization fails
                    output: output_value, // Wrapped String in serde_json::Value::String
                  })
     
         }


}
