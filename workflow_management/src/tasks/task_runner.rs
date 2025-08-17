use std::collections::HashMap;
use async_trait::async_trait;
use agent_core::planning::plan_definition::TaskDefinition;

/// A trait for any task that can be executed by the PlanExecutor.
#[async_trait]
pub trait TaskRunner: Send + Sync {
    /// The name of the task, used to look it up in the registry.
    fn name(&self) -> String;

    /// Executes the task.
    ///
    /// # Arguments
    /// * `task_definition` - The full definition of the task.
    /// * `dependencies` - A map of dependency task IDs to their string results.
    ///
    /// # Returns
    /// A `Result` containing the string output of the task or an error.
    async fn execute(
        &self,

        task_definition: &TaskDefinition,
        dependencies: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
