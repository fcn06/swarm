use rmcp::{
    ErrorData as McpError,  ServerHandler, model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;

#[derive(Clone)]
pub struct ScrapeMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ScrapeMcpService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service.")]
    async fn scrape_url(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        McpTools::scrape_url(params).await
    }
}

#[tool_handler]
impl ServerHandler for ScrapeMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a function 'scrape_url' to scrape a given URL.".to_string()),
        }
    }
}