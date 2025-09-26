use std::sync::Arc;
use std::any::Any;

use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::agent_communication::agent_registry::AgentRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;
use agent_core::business_logic::services::{WorkflowServiceApi};


#[derive( Clone)]
pub struct WorkFlowRegistry {
    pub task_registry: Arc<TaskRegistry>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
}

impl WorkFlowRegistry {
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



impl WorkFlowRegistry {

    pub  fn list_available_resources(&self) -> String {
    let list_agents_details = self.agent_registry.get_agent_details();
    let list_tools_details = self.tool_registry.get_tool_details();
    let list_tasks_details = self.task_registry.get_tasks_details();

    let available_resources=format!("{}\n{}\n{}\n", list_tools_details, list_tasks_details, list_agents_details);
    available_resources
    }


}

impl WorkflowServiceApi for WorkFlowRegistry {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
