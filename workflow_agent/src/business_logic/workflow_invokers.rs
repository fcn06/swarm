use std::sync::Arc;
use std::any::Any;

use workflow_management::agent_communication::agent_invoker::AgentInvoker;
use workflow_management::tasks::task_invoker::TaskInvoker;
use workflow_management::tools::tool_invoker::ToolInvoker;
use agent_core::business_logic::services::WorkflowServiceApi;

// TODO: Move this to a separate file if it grows
#[derive(Clone)]
pub struct WorkFlowInvokers {
    pub task_invoker: Arc<dyn TaskInvoker>,
    pub agent_invoker: Arc<dyn AgentInvoker>,
    pub tool_invoker: Arc<dyn ToolInvoker>,
}

impl WorkFlowInvokers {
    pub async fn init(
        task_invoker: Arc<dyn TaskInvoker>,
        agent_invoker: Arc<dyn AgentInvoker>,
        tool_invoker: Arc<dyn ToolInvoker>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            task_invoker,
            agent_invoker,
            tool_invoker,
        })
    }
}

impl WorkflowServiceApi for WorkFlowInvokers {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
