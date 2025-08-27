use std::sync::Arc;
use super::task_registry::TaskRegistry;
use super::task_invoker::TaskInvoker;

/* 
/// A trait for any task that can be executed by the PlanExecutor.
#[async_trait]
pub trait TaskRunner: Send + Sync {
    /// The name of the task, used to look it up in the registry.
    fn name(&self) -> String;

    /// Executes the task.
    ///
    /// # Arguments
    /// * `activity` - The activity to be executed.
    /// * `dependencies` - A map of dependency activity IDs to their results, now as `serde_json::Value`.
    ///
    /// # Returns
    /// A `Result` containing the string output of the activity or an error.
    async fn execute(
        &self,
        activity: &Activity,
        dependencies: &HashMap<String, Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
*/

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
    #[allow(unreachable_code)]
    pub async fn run(&self, task_id: String, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Optional: Fetch metadata for logging or validation before invoking
        if let Some(task_def) = self.task_registry.get_task_definition(&task_id) {
            // You now have easy access to tool_def.description here
            println!("Preparing to run task: {} - {}", task_def.name, task_def.description);
            // Potential input validation against tool_def.input_schema
        } else {
            return anyhow::bail!(format!("Task '{}' not found in registry.", task_id));
        }

        // Delegate the actual, protocol-specific invocation to the injected invoker
        self.task_invoker.invoke(task_id, &params).await
    }
}