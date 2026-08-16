use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rmcp::RoleClient;
use rmcp::model::InitializeRequestParams;
use rmcp::service::RunningService;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientCapabilities, Implementation, ListToolsResult, Tool,
};

use llm_api::chat::ToolCall;
use configuration::McpRuntimeConfig;
use crate::mcp_client::mcp_client::create_transport;

pub type McpClient = RunningService<RoleClient, InitializeRequestParams>;

pub struct McpRuntime {
    agent_mcp_config: McpRuntimeConfig,
    client: McpClient,
    tool_cache: Arc<RwLock<HashMap<String, Vec<Tool>>>>,
}

impl McpRuntime {
    /// Initializes the MCP client and connects to the server.
    pub async fn initialize_mcp_client_v2(agent_mcp_config: McpRuntimeConfig)
        -> anyhow::Result<Self> {
        
        let mcp_server_url_string = agent_mcp_config
            .agent_mcp_server_url.clone()
            .ok_or_else(|| anyhow::anyhow!("Missing MCP server URL in agent_mcp_config"))?;
        let mcp_server_url = mcp_server_url_string.as_str();

        let api_key = agent_mcp_config.agent_mcp_server_api_key.clone();
        
        let transport = create_transport(mcp_server_url, api_key);

        let client_info = InitializeRequestParams::new(
            ClientCapabilities::default(),
            Implementation::new("tool execution client", "0.0.1"),
        );

        let client = rmcp::serve_client(client_info, transport).await?;

        Ok(Self {
            agent_mcp_config,
            client,
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn get_client(&self) -> anyhow::Result<&McpClient> {
        Ok(&self.client)
    }

    pub fn get_config(&self) -> &McpRuntimeConfig {
        &self.agent_mcp_config
    }

    pub async fn get_tools_list_v2(&self) -> anyhow::Result<Vec<Tool>> {
        {
            let cache = self.tool_cache.read().unwrap();
            if let Some(tools) = cache.get("") {
                if !tools.is_empty() {
                    return Ok(tools.clone());
                }
            }
        }

        let list_tools: ListToolsResult = self.client.list_tools(Default::default()).await?;
        let mut cache = self.tool_cache.write().unwrap();
        cache.insert("".to_string(), list_tools.tools.clone());
        Ok(list_tools.tools)
    }

    pub fn invalidate_tool_cache(&self, session_key: &str) {
        let mut cache = self.tool_cache.write().unwrap();
        cache.remove(session_key);
        tracing::info!("🔄 Invalidated MCP tool cache for: {}", session_key);
    }

    pub async fn execute_tool_call_v2(
        &self,
        tool_call: ToolCall,
    ) -> anyhow::Result<CallToolResult> {
        let args: Result<serde_json::Value, _> = serde_json::from_str(&tool_call.function.arguments);

        let tool_result = match args {
            Ok(parsed_args) => {
                self.client
                    .call_tool(CallToolRequestParams::new(tool_call.function.name.clone())
                        .with_arguments(parsed_args.as_object().cloned().unwrap_or_default()))
                    .await?
            }
            Err(e) => {
                tracing::error!(
                    "Failed to parse arguments for {}: {}",
                    tool_call.function.name,
                    e
                );
                CallToolResult::error(vec![
                    rmcp::model::Content::text(format!(
                        "Failed to parse tool arguments: {}", e
                    ))
                ])
            }
        };

        tracing::info!("Tool result: {tool_result:#?}");

        Ok(tool_result)
    }
}