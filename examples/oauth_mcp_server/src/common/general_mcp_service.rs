use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;


#[derive(Clone)]
pub struct GeneralMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GeneralMcpService {
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

    #[tool(description = "Give customer details")]
    async fn get_customer_details(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestCustomerDetails>) 
            -> Result<CallToolResult, McpError> {
        McpTools::get_customer_details(params).await
    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service.")]
    async fn scrape_url(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        McpTools::scrape_url(params).await
    }

    #[tool(description = "Search for an entity on wikipedia.")]
    async fn search(
        &self, params: Parameters<crate::common::mcp_tools::StructRequestSearch>
    ) -> Result<CallToolResult, McpError> {
        McpTools::search(params).await
    }

}

#[tool_handler]
impl ServerHandler for GeneralMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides  a function 'get_current_weather' to retrieve weather from a specific location,  'get_customer_details' to get info about a customer, 'scrape_url' to scrape a given URL, and 'search' to search for an entity ( Name, Country, Animal,..)  on wikipedia content.".to_string()),
        }
    }

  
}