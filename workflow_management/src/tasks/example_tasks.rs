use crate::tasks::task_runner::TaskRunner;
use agent_core::planning::plan_definition::TaskDefinition;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct GreetTask;

#[async_trait]
impl TaskRunner for GreetTask {
    fn name(&self) -> String {
        "greet".to_string()
    }

    async fn execute(
        &self,
        task_definition: &TaskDefinition,
        _dependencies: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let name = task_definition
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
        task_definition: &TaskDefinition,
        dependencies: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This task can use the output of a dependency
        let greeting = dependencies
            .values()
            .next()
            .map(|s| s.as_str())
            .unwrap_or("...");
            
        let name = task_definition
            .tool_parameters
            .as_ref()
            .and_then(|params| params.get("name"))
            .and_then(|name| name.as_str())
            .unwrap_or("World");

        Ok(format!("{}, and now... Farewell, {}!", greeting, name))
    }
}
