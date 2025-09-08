
use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use serde_json::json;

static WIKIPEDIA_SEARCH_URL: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";

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

        let wikipedia_media_url = format!("{}/{}",WIKIPEDIA_SEARCH_URL,&search_query );

        let client = reqwest::Client::new();

        let response = client.get(wikipedia_media_url.clone())
            .header("User-Agent", "MyRustApp/1.0 (contact@example.com)")
            .send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": wikipedia_media_url.to_string()}))))?;

        let parsed_json_response: serde_json::Value = response.json().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": wikipedia_media_url.to_string()}))))?;

        let extract_from_response=    if let Some(extract_text) = parsed_json_response["extract"].as_str() {
                extract_text
            } else {
                "'extract' field not found or not a string."
            };

        Ok(CallToolResult::success(vec![Content::text(
            format!(r#"{{ "result": "Search result for '{}' : '{}' " }}"#, search_query, extract_from_response),
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
