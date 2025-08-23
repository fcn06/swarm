use std::collections::HashMap;
use async_trait::async_trait;
use crate::graph::graph_definition::Activity;
use serde_json::Value;

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
