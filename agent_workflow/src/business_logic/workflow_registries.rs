


pub struct WorkflowRegistries {
    task_registry: Arc<TaskRegistry>,
    agent_registry: Arc<AgentRegistry>,
    tool_registry: Arc<ToolRegistry>,
    mcp_client: McpClient,
}

impl WorkflowRegistries {
    pub fn new(
        task_registry: Arc<TaskRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
        mcp_client: McpClient,
    ) -> Self {
        WorkflowRegistries {
            task_registry,
            agent_registry,
            tool_registry,
            mcp_client,
        }
    }
}