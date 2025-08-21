use std::sync::Arc;
use std::any::Any;


use workflow_management::agent_communication::agent_registry::AgentRegistry;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;
use agent_core::business_logic::services::WorkflowServiceApi;

#[derive( Clone)]
pub struct WorkFlowRegistries {
    pub task_registry: Arc<TaskRegistry>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
}

impl WorkFlowRegistries {
    pub async fn init(
        task_registry: Arc<TaskRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
    ) -> anyhow::Result<Self> {

        Ok(Self {
            task_registry,
            agent_registry,
            tool_registry,
        })
    }
}

impl WorkflowServiceApi for WorkFlowRegistries {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
