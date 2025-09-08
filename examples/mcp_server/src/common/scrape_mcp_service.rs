use rmcp::{
    ErrorData as McpError,  ServerHandler, model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

use serde_json::json;

static JINA_AI_URL: &str = "https://r.jina.ai";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestUrlToScrape {
    #[schemars(description = "The URL to scrape")]
    pub url_to_scrape: String,
}

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
        &self, Parameters(StructRequestUrlToScrape { url_to_scrape }): Parameters<StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        let jina_ai_url = format!("{}/{}",JINA_AI_URL, url_to_scrape);
        let client = reqwest::Client::new();
        let response = client.get(&jina_ai_url).send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        let body = response.text().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        Ok(CallToolResult::success(vec![Content::text(body)]))
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