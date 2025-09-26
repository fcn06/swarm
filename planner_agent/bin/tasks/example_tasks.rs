use workflow_management::tasks::task_runner::TaskRunner;
use workflow_management::graph::graph_definition::Activity;
use async_trait::async_trait;
use std::collections::HashMap;
use serde_json::Value; // Added this import

pub struct GreetTask;

#[async_trait]
impl TaskRunner for GreetTask {
    fn name(&self) -> String {
        "greet".to_string()
    }

    async fn execute(
        &self,
        activity: &Activity,
        _dependencies: &HashMap<String, Value>, // Modified type
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let name = activity
            .tool_parameters
            .as_ref()
            .and_then(|params| params.get("name"))
            .and_then(|name| name.as_str())
            .unwrap_or("World");

        Ok(format!("Hello, {}!", name))
    }
}

pub struct FarewellTask;

#[async_trait]
impl TaskRunner for FarewellTask {
    fn name(&self) -> String {
        "farewell".to_string()
    }

    async fn execute(
        &self,
        activity: &Activity,
        dependencies: &HashMap<String, Value>, // Modified type
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This task can use the output of a dependency
        let greeting = dependencies
            .values()
            .next()
            .and_then(|v| v.as_str()) // Modified to get str from Value
            .unwrap_or("...");
            
        let name = activity
            .tool_parameters
            .as_ref()
            .and_then(|params| params.get("name"))
            .and_then(|name| name.as_str())
            .unwrap_or("World");

        Ok(format!("{}, and now... Farewell, {}!", greeting, name))
    }
}
