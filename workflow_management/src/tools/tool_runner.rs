use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

/// A trait for any tool that can be executed directly by the PlanExecutor.
/// Tools are intended to be stateless, atomic operations.
#[async_trait]
pub trait ToolRunner: Send + Sync {
    /// The name of the tool, used for lookup in the registry.
    fn name(&self) -> String;

    /// Executes the tool with the given parameters.
    ///
    /// # Arguments
    /// * `params` - A `serde_json::Value` containing the parameters for the tool.
    ///
    /// # Returns
    /// A `Result` containing the string output of the tool or an error.
    async fn run(&self, params: &Value) -> Result<String, Box<dyn Error + Send + Sync>>;
}
