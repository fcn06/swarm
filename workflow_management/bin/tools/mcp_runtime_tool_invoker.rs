use async_trait::async_trait;
use std::sync::Arc;


use rmcp::model::{ListToolsResult, Tool as RmcpTool,CallToolRequestParam}; // Alias for clarity

use llm_api::tools::{FunctionDefinition, FunctionParameters, Tool};

use configuration::AgentMcpConfig;

use serde_json::{Map,Value, from_value};
use anyhow::{Context};

use mcp_runtime::runtime::mcp_runtime::{McpRuntime};
use workflow_management::tools::tool_invoker::ToolInvoker;


pub struct  McpRuntimeToolInvoker  {
    mcp_runtime: Arc<McpRuntime>, // Your client for communicating with the MCP runtime
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

    pub async fn get_tools_list_v2(&self) -> anyhow::Result<Vec<Tool>> {
        let list_tools: ListToolsResult = self.mcp_runtime.get_client()?.list_tools(Default::default()).await?;
        let list_tools:Vec<RmcpTool> = list_tools.tools;
        let tools=McpRuntimeToolInvoker::transcode_tools(list_tools);
        Ok(tools?)
    }

    pub fn transcode_tools(rmcp_tools: Vec<RmcpTool>) -> anyhow::Result<Vec<Tool>> {
        rmcp_tools
            .into_iter()
            .map(|tool| {
                let tool_name = tool.name.to_string(); // Get name early for potential error context
                let description = tool
                    .description
                    .ok_or_else(|| {
                        anyhow::anyhow!("Tool description is missing for tool '{}'", tool_name)
                    })?
                    .to_string(); // Convert Arc<str> to String
    
                // Clone the input schema map directly
                let properties_map: Map<String, Value> = tool.input_schema.as_ref().clone();
    
                let properties = properties_map.get("properties");
                //println!("Properties : {:#?}", properties);
    
                Ok(Tool {
                    r#type: "function".to_string(),
                    function: FunctionDefinition {
                        name: tool_name, // Use owned name
                        description,
                        parameters: FunctionParameters {
                            r#type: "object".to_string(),
                            properties: properties
                                .cloned()
                                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
                            required: None, // Keep as None for now
                        },
                    },
                })
            })
            .collect::<anyhow::Result<Vec<Tool>>>()
            .with_context(|| "Failed to define tools from rmcp::model::Tool vector")
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