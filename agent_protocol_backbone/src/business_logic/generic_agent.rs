use async_trait::async_trait;
use anyhow::Result;
use url::Url;

#[async_trait]
pub trait Agent: Send + Sync + Clone + 'static {
    async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult>;
}

#[async_trait]
pub trait LlmClient: Send + Sync + Clone + 'static {
    async fn call_llm(&self, messages: Vec<LlmMessage>) -> anyhow::Result<LlmMessage>;
}

pub trait AgentConfig: Send + Sync + Clone + 'static {
    fn agent_name(&self) -> String;
    fn agent_host(&self) -> String;
    fn agent_http_port(&self) -> u16;
    fn agent_ws_port(&self) -> u16;
    fn agent_discovery_url(&self) -> Option<Url>;
    fn agent_system_prompt(&self) -> Option<String>;
    fn agent_version(&self) -> String;
    fn agent_description(&self) -> String;
    fn agent_skill_id(&self) -> String;
    fn agent_skill_name(&self) -> String;
    fn agent_skill_description(&self) -> String;
    fn agent_model_id(&self) -> String;
    fn agent_llm_url(&self) -> Url;
    fn agent_mcp_config_path(&self) -> Option<String>;
    fn agent_doc_url(&self) -> Option<Url>;
    fn agent_tags(&self) -> Vec<String>;
    fn agent_examples(&self) -> Vec<String>;
    fn agents_references(&self) -> Option<Vec<AgentReference>>;
}
