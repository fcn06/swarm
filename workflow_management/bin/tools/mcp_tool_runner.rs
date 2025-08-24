use workflow_management::tools::tool_runner::ToolRunner;
use async_trait::async_trait;

use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use configuration::AgentMcpConfig;
use rmcp::model::{CallToolRequestParam};

use serde_json::{Value, from_value};
use std::error::Error;
use anyhow::Context;

/// A ToolRunner that calls tools via an McpClient.
pub struct McpToolRunner {
    mcp_agent: McpAgent,
    tool_name: String,
}

impl McpToolRunner {
    pub fn new(mcp_agent: McpAgent, tool_name: String) -> Self {
        Self { mcp_agent, tool_name }
    }

    pub async fn initialize_mcp_agent(mcp_config_path: String) -> anyhow::Result<Option<McpAgent>> {
        let agent_mcp_config = AgentMcpConfig::load_agent_config(mcp_config_path.as_str())
            .context("Error loading MCP config for planner")?;
        let mcp_agent = McpAgent::new(agent_mcp_config).await?;
        Ok(Some(mcp_agent))
    }

}
        
    

#[async_trait]
impl ToolRunner for McpToolRunner {
    fn name(&self) -> String {
        self.tool_name.clone()
    }

    async fn run(&self, params: &Value) -> Result<String, Box<dyn Error + Send + Sync>> {

        let tool_name_value = params.get("tool_to_use")
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Missing 'tool_name' in parameters for McpToolRunner".to_string()))?;

        let tool_name = tool_name_value.as_str()
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("'tool_name' must be a string".to_string()))?
            .to_string();

        let arguments_map = from_value(params.clone())?;

        let call_tool_request_param = CallToolRequestParam {
            name: tool_name.into(),
            arguments: Some(arguments_map),
        };


        let tool_result = self.mcp_agent.mcp_client.call_tool(call_tool_request_param).await?;
        
        let result_str = serde_json::to_string(&tool_result.content)?;
        Ok(result_str)
    }
}
