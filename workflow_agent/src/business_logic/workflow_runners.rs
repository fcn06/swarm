use std::sync::Arc;
use std::any::Any;

use workflow_management::agent_communication::agent_runner::AgentRunner;
use workflow_management::tasks::task_runner::TaskRunner;
use workflow_management::tools::tool_runner::ToolRunner;

use agent_core::business_logic::services::WorkflowServiceApi;

#[derive( Clone)]
pub struct WorkFlowRunners {
    pub task_runner: Arc<TaskRunner>,
    pub agent_runner: Arc<AgentRunner>,
    pub tool_runner: Arc<ToolRunner>,
}

impl WorkFlowRunners {
    pub async fn init(
        task_runner: Arc<TaskRunner>,
        agent_runner: Arc<AgentRunner>,
        tool_runner: Arc<ToolRunner>,
    ) -> anyhow::Result<Self> {

        Ok(Self {
            task_runner,
            agent_runner,
            tool_runner,
        })
    }
}

impl WorkflowServiceApi for WorkFlowRunners {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl WorkFlowRunners {

    pub  fn list_available_resources(&self) -> String {
    let list_tools_details = self.tool_runner.tool_registry.get_tool_details();
    let list_tasks_details = self.task_runner.task_registry.get_tasks_details();
    let list_agents_details = self.agent_runner.agent_registry.get_agent_details();

    let available_resources=format!("Here is the list of available resources : \n{}\n{}\n{}\n", list_tools_details, list_tasks_details, list_agents_details);
    available_resources
    }


}