use async_trait::async_trait;

#[async_trait]
pub trait AgentInvoker: Send + Sync {
    /// Interacts with an agent with the given ID, sending a message.
    /// The concrete implementation handles the agent-specific communication.
    async fn interact(&self, agent_id: String, message: serde_json::Value) -> Result<serde_json::Value, String>;
}