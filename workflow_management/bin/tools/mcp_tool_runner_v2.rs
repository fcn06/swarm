use workflow_management::tools::tool_runner::ToolRunner;
use async_trait::async_trait;

use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use configuration::AgentMcpConfig;
use rmcp::model::{CallToolRequestParam};

use serde_json::{Value, from_value};
use std::error::Error;
use anyhow::{Context};


use mcp_runtime::runtime::mcp_runtime::{McpClient,McpRuntime};
use workflow_management::tools::tool_invoker::ToolInvoker;
use std::collections::HashMap;
use std::sync::Arc;



pub struct  McpRuntimeToolInvoker  {
    mcp_runtime: Arc<McpRuntime>, // Your client for communicating with the MCP runtime
    list_tools: Vec<String>, // assuming it is same name than MCP server we are using
}

impl McpRuntimeToolInvoker  {
    pub fn new(mcp_runtime: McpRuntime, list_tools: Vec<String>) -> Self {
        Self { mcp_runtime : Arc::new(mcp_runtime), list_tools }
    }


    pub async fn initialize_mcp_agent(mcp_config_path: String) -> anyhow::Result<McpRuntime> {
        let agent_mcp_config = AgentMcpConfig::load_agent_config(mcp_config_path.as_str())
            .context("Error loading MCP config for planner")?;
        
        let mcp_runtime = McpRuntime::initialize_mcp_client_v2(agent_mcp_config).await?;
        Ok(mcp_runtime)
    }

}


#[async_trait]
impl ToolInvoker for McpRuntimeToolInvoker  {

    async fn invoke(&self, tool_id:String,params: &Value) -> anyhow::Result<serde_json::Value>  {

        let tool_name_value = match self.list_tools.contains(&tool_id) {
            true => tool_id,
            false => return anyhow::bail!(format!("Tool ID '{}' not mapped to an MCP runtime tool.", tool_id)),
        };

        let tool_name = match  tool_name_value.as_str() {
            tool_name => tool_name.to_string(),
            _ => return anyhow::bail!(format!("tool_name' must be a string.")),
        };

        let arguments_map = from_value(params.clone())?;

        let call_tool_request_param = CallToolRequestParam {
            name: tool_name.into(),
            arguments: Some(arguments_map),
        };

        let tool_result = self.mcp_runtime.get_client()?.call_tool(call_tool_request_param).await?;
        
        let tool_result_value = serde_json::to_value(&tool_result.content)?;
        Ok(tool_result_value)
    }



}