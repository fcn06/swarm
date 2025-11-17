use a2a_rs::{
    HttpClient,
    domain::{ AgentSkill},
};

use std::sync::Arc;
use async_trait::async_trait;

// describe how to interact with a single agent

#[async_trait]
pub trait AgentInteraction: Send + Sync  + Clone + 'static {
    async fn new(id: String, uri: String) -> anyhow::Result<Self>;
    async fn execute_task(&self, task_description: &str, _skill_to_use: &str) -> anyhow::Result<String>;
    fn get_skills(&self) -> &[AgentSkill];
    fn has_skill(&self, skill_name: &str) -> bool ;
    fn agent_id(&self) -> String ;
    fn agent_uri(&self) -> String ;
    fn agent_skills(&self) -> Vec<AgentSkill> ;
    fn agent_remote_http_client(&self) -> Arc<HttpClient> ;
}

