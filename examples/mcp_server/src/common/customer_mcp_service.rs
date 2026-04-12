use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;

#[derive(Clone)]
pub struct CustomerMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CustomerMcpService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Give customer details")]
    async fn get_customer_details(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestCustomerDetails>) 
            -> Result<CallToolResult, McpError> {
        McpTools::get_customer_details(params).await
    }
}

#[tool_handler]
impl ServerHandler for CustomerMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build())
            .with_instructions("This server provides a function 'get_customer_details' to get info about a customer.")
    }
}