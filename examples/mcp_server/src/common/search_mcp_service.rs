
use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use serde_json::json;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    #[schemars(description = "The search query for wikipedia")]
    pub search_query: String,
}

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

    #[tool(description = "Performs a simple search against Wikipedia")]
    async fn search(
        &self, Parameters(SearchRequest { search_query }): Parameters<SearchRequest>
    ) -> Result<CallToolResult, McpError> {

        let wikipedia_search_url = reqwest::Url::parse_with_params(
            "https://en.wikipedia.org/w/api.php",
            &[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", &search_query),
                ("format", "json"),
                ("utf8", "1"),
            ],
        ).unwrap();

        let client = reqwest::Client::new();

        let response = client.get(wikipedia_search_url.clone())
        .header("User-Agent", "MyRustApp/1.0 (contact@example.com)")
        .send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": wikipedia_search_url.to_string()}))))?;

        let body = response.text().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": wikipedia_search_url.to_string()}))))?;

        Ok(CallToolResult::success(vec![Content::text(
            format!("{{\"result\": \"Search result for '{}' : '{}' \"}}", search_query, body),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for SearchMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a 'search' function against wikipedia content.".to_string()),
        }
    }
}
