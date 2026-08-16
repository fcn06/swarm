use anyhow::Result;

use rmcp::RoleClient;
use rmcp::model::InitializeRequestParams;
use rmcp::service::RunningService;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientCapabilities, Implementation, ListToolsResult, Tool,
};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::StreamableHttpClientTransport;

use std::sync::Arc;

use llm_api::chat::ToolCall;
use configuration::McpRuntimeConfig;

// https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md

pub type McpClient = RunningService<RoleClient, InitializeRequestParams>;

// todo: implement client for oauth2 protected mcp server
// see example mcp_client

pub fn create_transport(
    uri: impl Into<Arc<str>>,
    api_key: Option<String>,
) -> StreamableHttpClientTransport<reqwest::Client> {
    use reqwest::header;
    let mut headers = header::HeaderMap::new();
    headers.insert("X-MY-HEADER", header::HeaderValue::from_static("value"));
    
    if let Some(key) = api_key {
        if !key.is_empty() {
            let bearer = format!("Bearer {}", key);
            if let Ok(mut auth_value) = header::HeaderValue::from_str(&bearer) {
                auth_value.set_sensitive(true);
                headers.insert(header::AUTHORIZATION, auth_value);
            }
        }
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build reqwest client");

    let config = StreamableHttpClientTransportConfig::with_uri(uri);
    StreamableHttpClientTransport::with_client(client, config)
}


/// Initializes the MCP client and connects to the server.
/// Initializes logging (potentially repeated if called multiple times).
pub async fn initialize_mcp_client_v2(agent_mcp_config: McpRuntimeConfig)
    -> anyhow::Result<McpClient> {
    
    let mcp_server_url_string = agent_mcp_config
        .agent_mcp_server_url
        .ok_or_else(|| anyhow::anyhow!("Missing MCP server URL in agent_mcp_config"))?;
    let mcp_server_url = mcp_server_url_string.as_str();

    let api_key = agent_mcp_config.agent_mcp_server_api_key.clone();
    
    let transport = create_transport(mcp_server_url, api_key);

    let client_info = InitializeRequestParams::new(
        ClientCapabilities::default(),
        Implementation::new("tool execution client", "0.0.1"),
    );

    let client = rmcp::serve_client(client_info, transport).await?;

    Ok(client)
}

pub async fn get_tools_list_v2(
    client: Arc<McpClient>,
) -> anyhow::Result<Vec<Tool>> {
    let list_tools: ListToolsResult = client.list_tools(Default::default()).await?;
    Ok(list_tools.tools)
}

pub async fn execute_tool_call_v2(
    client: Arc<McpClient>,
    tool_call: ToolCall,
) -> anyhow::Result<CallToolResult> {
    let args: Result<serde_json::Value, _> = serde_json::from_str(&tool_call.function.arguments);

    let tool_result = match args {
        Ok(parsed_args) => {
            client
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