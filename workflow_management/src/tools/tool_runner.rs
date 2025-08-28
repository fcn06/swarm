use std::sync::Arc;
use super::tool_registry::ToolRegistry;
use super::tool_invoker::ToolInvoker;

/* 

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

*/

// V2 implementation, more flexible

pub struct ToolRunner {
    pub tool_registry: Arc<ToolRegistry>, // To get metadata
    tool_invoker: Arc<dyn ToolInvoker>, // To perform actual invocation
}

impl ToolRunner {
    // Constructor using dependency injection
    pub fn new(tool_registry: Arc<ToolRegistry>, tool_invoker: Arc<dyn ToolInvoker>) -> Self {
        ToolRunner { tool_registry, tool_invoker }
    }

    /// Executes a tool identified by its ID.
    pub async fn run(&self, tool_id: String, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Optional: Fetch metadata for logging or validation before invoking
        if let Some(tool_def) = self.tool_registry.get_tool_definition(&tool_id) {
            // You now have easy access to tool_def.description here
            println!("Preparing to run tool: {} - {}", tool_def.name, tool_def.description);
            // Potential input validation against tool_def.input_schema
        } else {
            anyhow::bail!(format!("Tool '{}' not found in registry.", tool_id));
        }

        // Delegate the actual, protocol-specific invocation to the injected invoker
        self.tool_invoker.invoke(tool_id, &params).await
    }
}