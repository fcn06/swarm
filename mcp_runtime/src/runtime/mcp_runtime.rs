use anyhow::Result;

use rmcp::RoleClient;
use rmcp::model::InitializeRequestParam;
use rmcp::service::RunningService;
use rmcp::{
    ServiceExt,
    model::{
        CallToolResult, ClientCapabilities, ClientInfo, Implementation, ListToolsResult, Tool,
    },
    transport::{SseClientTransport, sse_client::SseClientConfig,sse_client::SseTransportError},
};

use rmcp::model::CallToolRequestParam;

use std::borrow::Cow;

use std::sync::Arc;

use llm_api::chat::ToolCall;
use configuration::AgentMcpConfig;

// https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md
// https://github.com/modelcontextprotocol/rust-sdk/blob/b9d7d61ebd6e8385cbc4aa105d4e25774fc1a59c/crates/rmcp/src/transport/common/reqwest/sse_client.rs#L25 

// enable default headers. Does not exist in the original rust mcp-sdk crate https://github.com/modelcontextprotocol/rust-sdk
//https://github.com/seanmonstar/reqwest/blob/master/src/async_impl/client.rs

pub type McpClient = RunningService<RoleClient, InitializeRequestParam>;

#[allow(dead_code)]
pub struct McpRuntime {
    agent_mcp_config: AgentMcpConfig,
    client: McpClient,
}

impl McpRuntime {
    

pub async fn start_with_default_headers(
        uri: impl Into<Arc<str>>,
        api_key: Option<String>,
    ) -> Result<SseClientTransport<reqwest::Client>, SseTransportError<reqwest::Error>> {

    use reqwest::header;
    let mut headers = header::HeaderMap::new();
    headers.insert("X-MY-HEADER", header::HeaderValue::from_static("value"));
    
    // HeaderValue::from_str("key=value").unwrap()
    let bearer=format!("Bearer {}",api_key.expect("").as_str());
    let mut auth_value = header::HeaderValue::from_str(&bearer).unwrap();
    
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    //println!("Headers :{:?}",headers);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    SseClientTransport::start_with_client(
        client,
        SseClientConfig {
            sse_endpoint: uri.into(),
            ..Default::default()
        },
    )
    .await
}


/// Initializes the MCP client and connects to the server.
/// Initializes logging (potentially repeated if called multiple times).
pub async fn initialize_mcp_client_v2(agent_mcp_config: AgentMcpConfig)
    -> anyhow::Result<Self> {
    
    let mcp_server_url_string = agent_mcp_config
        .agent_mcp_server_url.clone()
        .expect("Missing mcp server Url");
    let mcp_server_url = mcp_server_url_string.as_str();

    // Need to create and pass a static String to inject a default header
    let api_key=agent_mcp_config.agent_mcp_server_api_key.clone();
    
    let transport = Self::start_with_default_headers(mcp_server_url,api_key).await?;

    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "tool execution client".to_string(),
            version: "0.0.1".to_string(),
        },
    };

    let client = client_info.serve(transport).await?;

    Ok(Self{agent_mcp_config,client})
}

pub  fn get_client(&self,) -> anyhow::Result<&McpClient> {
    Ok(&self.client)
}

pub async fn get_tools_list_v2(
    &self,
) -> anyhow::Result<Vec<Tool>> {
    let list_tools: ListToolsResult = self.client.list_tools(Default::default()).await?;
    Ok(list_tools.tools)
}

pub async fn execute_tool_call_v2(
    &self,
    tool_call: ToolCall,
) -> anyhow::Result<CallToolResult> {
    let args: Result<serde_json::Value, _> = serde_json::from_str(&tool_call.function.arguments);

    // todo : Make it more resilient
    let tool_result = match args {
        Ok(parsed_args) => {
            self.client
                .call_tool(CallToolRequestParam {
                    name: Cow::Owned(tool_call.function.name.clone()), // Use Cow::Owned for 'static lifetime
                    arguments: parsed_args.as_object().cloned(),       // Use parsed arguments
                })
                .await?
        }
        Err(e) => {
            // Handle the error appropriately, perhaps by returning an error
            // For now, let's log the error and potentially return a default/error result
            tracing::error!(
                "Failed to parse arguments for {}: {}",
                tool_call.function.name,
                e
            );
            // Depending on expected behavior, you might want to return Err here
            // For example: return Err(anyhow::anyhow!("Argument parsing failed: {}", e));
            // Returning a dummy ListToolsResult for now, adjust as needed
            CallToolResult {
                content: vec![],
                structured_content:None,
                is_error: Some(true),
            } // Fixed: Provide bool directly
        }
    };

    tracing::info!("Tool result: {tool_result:#?}");

    Ok(tool_result)
}

}