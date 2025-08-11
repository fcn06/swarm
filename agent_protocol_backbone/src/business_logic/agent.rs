use async_trait::async_trait;
use llm_api::chat::Message as Message_Llm;
use serde::{Serialize,Deserialize};

/* 
#[async_trait]
pub trait Agent: Send + Sync + Clone + 'static {
    //async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
    async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<Message_Llm>;
}
*/

#[async_trait]
pub trait Agent: Send + Sync  + Clone + 'static {
    //async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
    async fn new( agent_config: impl AgentConfig) -> anyhow::Result<Self>;
    async fn handle_request(&self, request: Message_Llm) -> anyhow::Result<Message_Llm>;
}

pub trait AgentConfig: Send + Sync + Clone + 'static {
    fn agent_name(&self) -> String;
    fn agent_host(&self) -> String;
    fn agent_http_port(&self) -> u16;
    fn agent_ws_port(&self) -> u16;
    fn agent_discovery_url(&self) -> Option<String>;
    fn agent_system_prompt(&self) -> Option<String>;
    fn agent_version(&self) -> String;
    fn agent_description(&self) -> String;
    fn agent_skill_id(&self) -> String;
    fn agent_skill_name(&self) -> String;
    fn agent_skill_description(&self) -> String;
    fn agent_model_id(&self) -> String;
    fn agent_llm_url(&self) -> String;
    fn agent_mcp_config_path(&self) -> Option<String>;
    fn agent_doc_url(&self) -> Option<String>;
    fn agent_tags(&self) -> Vec<String>;
    fn agent_examples(&self) -> Vec<String>;
    fn agents_references(&self) -> Option<Vec<AgentReference>>;
}

///////////////////////////////////////////////////////////////
// SIMPLE AGENT REFERENCE IMPLEMENTATION
///////////////////////////////////////////////////////////////

// Agent info provider implementation
//#[derive(Clone)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReference {
    pub name: String,
    pub url: String,
    pub is_default:Option<bool>,
}

impl AgentReference {
    pub fn new(name: String, url: String) -> anyhow::Result<AgentReference> {
        // Create the agent card
        Ok(AgentReference {
            name: name,
            url: url,
            is_default:None,
        })
    }
}

impl AgentReference {
    pub async fn get_agent_reference(&self) -> anyhow::Result<AgentReference> {
        Ok(self.clone())
    }
}