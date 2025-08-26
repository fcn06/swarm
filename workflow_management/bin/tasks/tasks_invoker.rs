//use workflow_management::graph::graph_definition::Activity;
use async_trait::async_trait;
use serde_json::Value; // Added this import
use serde_json::json;

use workflow_management::tasks::task_invoker::TaskInvoker;

pub struct GreetTask;

#[async_trait]
impl TaskInvoker for GreetTask {

    #[allow(unused_variables)]
    async fn invoke(
        &self,
        tool_id: String, 
        params: &Value
    ) ->anyhow::Result<Value> {

        let name = params.get("name").and_then(|value| value.as_str()).unwrap_or("World");

        Ok(json!({"response":format!("Hello, {}!", name)}))
    }
}

impl GreetTask {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self{})
    }
}

