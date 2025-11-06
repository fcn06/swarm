use async_trait::async_trait;
use uuid::Uuid;
use configuration::{McpRuntimeConfig};
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

use configuration::AgentConfig;

use agent_models::execution::execution_result::{ExecutionResult};

use agent_core::business_logic::services::WorkflowServiceApi;

/// Modern A2A server setup 
#[derive(Clone)]
pub struct BasicAgent {
    llm_interaction: ChatLlmInteraction,
    mcp_agent:Option<Arc<Mutex<McpAgent>>>
}

#[async_trait]
impl Agent for BasicAgent {

    /// Creation of a new simple a2a agent
    async fn new(
        agent_config: AgentConfig,
        agent_api_key:String,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {

               // Set model to be used
        let model_id = agent_config.agent_model_id();

        // Set system message to be used
        let _system_message = agent_config.agent_system_prompt();

        // Set API key for LLM
        let llm_a2a_api_key =agent_api_key;

        let llm_interaction= ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            model_id,
            llm_a2a_api_key,
        );

        // Load MCP agent if specified in planner config
        let mcp_agent = match agent_config.agent_mcp_config_path() {
            None => None,
            Some(path) => {
                let agent_mcp_config = McpRuntimeConfig::load_agent_config(path.as_str()).expect("Error loading MCP config for planner");
                let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                Some(Arc::new(Mutex::new(mcp_agent)))
            },
        };

          Ok(Self {
            llm_interaction,
            mcp_agent,
          })
    }


    

        /// business logic for handling user request
        async fn handle_request(&self, request: LlmMessage,_metadata:Option<Map<String, Value>>) ->anyhow::Result<ExecutionResult> {
       
            let request_id=uuid::Uuid::new_v4().to_string();
            let conversation_id = Uuid::new_v4().to_string();
     
             // use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
             let response =if self.mcp_agent.is_none() {
                     self.llm_interaction.call_api_simple("user".to_string(),request.content.expect("Empty Message").to_string()).await.unwrap()
     
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

                debug!("Output Value from Basic Agent: {:?}", output_value);
     
             Ok(ExecutionResult {
                    request_id,
                    conversation_id,
                    success: true, // Mark as not fully successful if summarization fails
                    output: output_value, // Wrapped String in serde_json::Value::String
                  })
     
         }


}

// todo: have it part of Agent trait

impl BasicAgent {

pub async fn new_with_mcp(
    agent_config: AgentConfig,
    agent_api_key:String,
    mcp_runtime_config: McpRuntimeConfig,
    mcp_runtime_api_key:String,
    _evaluation_service: Option<Arc<dyn EvaluationService>>,
    _memory_service: Option<Arc<dyn MemoryService>>,
    _discovery_service: Option<Arc<dyn DiscoveryService>>,
    _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
) -> anyhow::Result<Self> {

           // Set model to be used
    let model_id = agent_config.agent_model_id();

    // Set system message to be used
    let _system_message = agent_config.agent_system_prompt();

    // Set API key for LLM
    let llm_a2a_api_key =agent_api_key;

    let llm_interaction= ChatLlmInteraction::new(
        agent_config.agent_llm_url(),
        model_id,
        llm_a2a_api_key,
    );

    let mcp_agent=Some(Arc::new(Mutex::new(McpAgent::new_v2(mcp_runtime_config,mcp_runtime_api_key).await?)));


      Ok(Self {
        llm_interaction,
        mcp_agent,
      })
}

}