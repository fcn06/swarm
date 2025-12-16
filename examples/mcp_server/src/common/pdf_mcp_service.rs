use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use crate::common::mcp_tools::McpTools;


#[derive(Clone)]
pub struct PdfMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PdfExtractionMcpService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Extract Data ( text/metaata) from a pdf defined by its url")]
    async fn pdf_extract(
        &self, params: Parameters<crate::common::mcp_tools::StructPdfExtraction>
    ) -> Result<CallToolResult, McpError> {
        McpTools::pdf_extract(params).await
    }
}

#[tool_handler]
impl ServerHandler for PdfExtractionMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a function to extract data ( text and metadata) from pdf".to_string()),
        }
    }
}
