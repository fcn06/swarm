use async_trait::async_trait;

use configuration::AgentMcpConfig;
use rmcp::model::{CallToolRequestParam};

use serde_json::{Value, from_value};
use anyhow::{Context};


use mcp_runtime::runtime::mcp_runtime::{McpRuntime};
use workflow_management::tools::tool_invoker::ToolInvoker;
use std::sync::Arc;



pub struct  McpRuntimeToolInvoker  {
    mcp_runtime: Arc<McpRuntime>, // Your client for communicating with the MCP runtime
    //list_tools:Vec<String >,
}

impl McpRuntimeToolInvoker  {
    pub async fn new(mcp_config_path: String) -> anyhow::Result<Self> {
        let mcp_runtime = Arc::new(Self::initialize_mcp_agent(mcp_config_path).await?);
        // todo : instantiate list of tools
        Ok(Self { mcp_runtime  })
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

        let arguments_map = from_value(params.clone())?;

        let call_tool_request_param = CallToolRequestParam {
            name: tool_id.into(),
            arguments: Some(arguments_map),
        };

        let tool_result = self.mcp_runtime.get_client()?.call_tool(call_tool_request_param).await?;
        
        let tool_result_value = serde_json::to_value(&tool_result.content)?;
        Ok(tool_result_value)
    }



}