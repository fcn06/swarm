use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
};


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestCustomerDetails {
    #[schemars(description = "Give customer details from a given customer_id")]
    pub _customer_id: String,
}

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
        &self,Parameters(StructRequestCustomerDetails { _customer_id }): Parameters<StructRequestCustomerDetails>) 
            -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            r#"{"Full Name": "Company A", "address": "Sunny Street"}"#,
        )]))
    }
}

#[tool_handler]
impl ServerHandler for CustomerMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a function 'get_customer_details' to get info about a customer.".to_string()),
        }
    }
}