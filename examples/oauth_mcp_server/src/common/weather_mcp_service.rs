use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;

#[derive(Clone)]
pub struct WeatherMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl WeatherMcpService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get the current weather in a given location")]
    async fn get_current_weather(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestLocation>) -> Result<CallToolResult, McpError> {
        McpTools::get_current_weather(params).await
    }
}

#[tool_handler]
impl ServerHandler for WeatherMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a function 'get_current_weather' to retrieve weather from a specific location.".to_string()),
        }
    }
}