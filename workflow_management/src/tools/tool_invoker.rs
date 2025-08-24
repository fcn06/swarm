use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ToolInvoker: Send + Sync {
    /// Invokes a tool with the given ID and input.
    /// The concrete implementation knows how to translate this into a protocol-specific call.
    async fn invoke(&self, tool_id: String, params: &Value) -> anyhow::Result<serde_json::Value>;
}
