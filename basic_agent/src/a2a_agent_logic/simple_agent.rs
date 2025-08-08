//use std::sync::{Arc};
//use async_trait::async_trait;

use configuration::{AgentA2aConfig,AgentMcpConfig};
use llm_api::chat::{ChatLlmInteraction};
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;
use llm_api::chat::Message as Message_Llm;
use std::env;

/// Modern A2A server setup 
#[derive(Clone)]
pub struct SimpleAgent {
    //agent_a2a_config: AgentA2aConfig,
    llm_interaction: ChatLlmInteraction,
    mcp_agent:Option<McpAgent>
}

impl SimpleAgent{

    /// Creation of a new simple a2a agent
    pub async fn new(
        agent_a2a_config: AgentA2aConfig) -> anyhow::Result<Self> {

               // Set model to be used
        let model_id = agent_a2a_config.agent_a2a_model_id.clone();

        // Set model to be used
        let _system_message = agent_a2a_config.agent_a2a_system_prompt.clone();

        // Set API key for LLM
        let llm_a2a_api_key = env::var("LLM_A2A_API_KEY").expect("LLM_A2A_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
            agent_a2a_config.agent_a2a_llm_url.clone(),
            model_id,
            llm_a2a_api_key,
        );

        // Load MCP agent if specified in planner config
        let mcp_agent = match agent_a2a_config.agent_a2a_mcp_config_path.clone() {
            None => None,
            Some(path) => {
                let agent_mcp_config = AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading MCP config for planner");
                let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                Some(mcp_agent)
            },
        };

          Ok(Self {
           // agent_a2a_config, // possible future use
            llm_interaction,
            mcp_agent,
          })
    }

    /// business logic for handling user request
    pub async fn handle_user_request(&mut self, user_request: Message_Llm) -> anyhow::Result<Message_Llm> {
       

        // use MCP LLM to answer if there is a MCP runtime, Agent LLM otherwise 
        let response =if self.mcp_agent.is_none() {
                self.llm_interaction.call_api_simple("user".to_string(),user_request.content.expect("Empty Message").to_string()).await.unwrap()

            } else {
                self.mcp_agent.clone().unwrap().run_agent_internal(user_request.clone())
                .await
                // todo : make it more robust
                .unwrap()
            };

        Ok(response.expect("No Return from LLM"))

    }
    

}
