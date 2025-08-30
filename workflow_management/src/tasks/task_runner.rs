use std::sync::Arc;
use super::task_registry::TaskRegistry;
use super::task_invoker::TaskInvoker;


// V2 implementation, more flexible

pub struct TaskRunner {
    pub task_registry: Arc<TaskRegistry>, // To get metadata
    task_invoker: Arc<dyn TaskInvoker>, // To perform actual invocation
}

impl TaskRunner {
    // Constructor using dependency injection
    pub fn new(task_registry: Arc<TaskRegistry>, task_invoker: Arc<dyn TaskInvoker>) -> Self {
        TaskRunner { task_registry, task_invoker }
    }

    /// Executes a tool identified by its ID.
    pub async fn run(&self, task_id: String, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Optional: Fetch metadata for logging or validation before invoking
        if let Some(task_def) = self.task_registry.get_task_definition(&task_id) {
            // You now have easy access to tool_def.description here
            println!("Preparing to run task: {} - {}", task_def.name, task_def.description);
            // Potential input validation against tool_def.input_schema
        } else {
            anyhow::bail!(format!("Task '{}' not found in registry.", task_id));
        }

        // Delegate the actual, protocol-specific invocation to the injected invoker
        self.task_invoker.invoke(task_id, &params).await
    }
}