use async_trait::async_trait;

use configuration::{AgentMcpConfig};
use llm_api::chat::{ChatLlmInteraction};

// todo : change the prompt of mcp runtime , so that he tries to use internal knowledge if possible
// todo: see if the method of delegation to mcp_runtime is optimal
use mcp_runtime::mcp_agent_logic::agent::McpAgent;

use llm_api::chat::Message as LlmMessage;
use std::env;

use agent_protocol_backbone::business_logic::agent::{Agent};
use agent_protocol_backbone::config::agent_config::{AgentConfig};
use agent_protocol_backbone::planning::plan_definition::{ExecutionResult};


/// Modern A2A server setup 
#[derive(Clone)]
pub struct BasicAgent {
    llm_interaction: ChatLlmInteraction,
    mcp_agent:Option<McpAgent>
}

#[async_trait]
impl Agent for BasicAgent {

    /// Creation of a new simple a2a agent
    async fn new(
        agent_config: AgentConfig ) -> anyhow::Result<Self> {

               // Set model to be used
        let model_id = agent_config.agent_model_id();

        // Set system message to be used
        let _system_message = agent_config.agent_system_prompt();

        // Set API key for LLM
        let llm_a2a_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            model_id,
            llm_a2a_api_key,
        );

        // Load MCP agent if specified in planner config
        let mcp_agent = match agent_config.agent_mcp_config_path() {
            None => None,
            Some(path) => {
                let agent_mcp_config = AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading MCP config for planner");
                let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                Some(mcp_agent)
            },
        };

          Ok(Self {
            llm_interaction,
            mcp_agent,
          })
    }

    /// business logic for handling user request
    async fn handle_request(&self, request: LlmMessage) ->anyhow::Result<ExecutionResult> {
       
       let request_id=uuid::Uuid::new_v4().to_string();

        // use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
        let response =if self.mcp_agent.is_none() {
                self.llm_interaction.call_api_simple("user".to_string(),request.content.expect("Empty Message").to_string()).await.unwrap()

            } else {
                self.mcp_agent.clone().unwrap().run_agent_internal(request.clone())
                .await
                // todo : make it more robust
                .unwrap()
            };


        Ok(ExecutionResult {
            request_id,
            success: true, // Mark as not fully successful if summarization fails
            output: response.expect("No Return from LLM").content.expect("Empty result from Llm"),
            plan_details: None,
        })

    }
    

}
