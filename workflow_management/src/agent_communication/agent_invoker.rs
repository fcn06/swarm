use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait AgentInvoker: Send + Sync + 'static {
    /// Interacts with an agent with the given ID, sending a message.
    /// The concrete implementation handles the agent-specific communication.
    async fn interact(&self, agent_id: String, message: String,skill:String) -> anyhow::Result<serde_json::Value>;
    fn as_any(&self) -> &dyn Any; // Added for downcasting
}