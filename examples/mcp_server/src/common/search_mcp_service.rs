use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;

#[derive(Clone)]
pub struct SearchMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SearchMcpService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Performs a simple search in the internet")]
    async fn search(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestSearch>
    ) -> Result<CallToolResult, McpError> {
        McpTools::search(params).await
    }
}

#[tool_handler]
impl ServerHandler for SearchMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder()
                .enable_tools()
                .build())
            .with_instructions("This server provides a 'search' function from internet")
    }
}
