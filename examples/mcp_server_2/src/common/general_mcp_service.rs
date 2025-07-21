use std::sync::Arc;

use rmcp::{
    Error as McpError, RoleServer, ServerHandler, const_string, model::*, schemars,
    service::RequestContext, tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
};
use serde_json::json;
use tokio::sync::Mutex;


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestLocation {
    #[schemars(description = "Location for which you desire to know weather")]
    pub location: String,
    #[schemars(description = "Temperature unit to use. You can specify Degree Celsius or Degree Farenheit")]
    pub unit: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestCustomerDetails {
    #[schemars(description = "Give customer details from a given customer_id")]
    pub customer_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestUrlToScrape {
    #[schemars(description = "The URL to scrape")]
    pub url_to_scrape: String,
}


#[derive(Clone)]
pub struct GeneralMcpService {
    tool_router: ToolRouter<Self>,
}

//#[tool(tool_box)]
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
        &self, Parameters(StructRequestLocation { location, unit }): Parameters<StructRequestLocation>) -> Result<CallToolResult, McpError> {
        let unit = unit.unwrap_or("Degree Celsius".to_string());
        let begining_string=r#""{"Temperature": "24", "unit":""#;
        let end_string=r#"","description":"Sunny"}"#;
        Ok(CallToolResult::success(vec![Content::text(
            format!("{}{}{}",begining_string,unit,end_string),
        )]))
    }



    #[tool(description = "Give customer details")]
    async fn get_customer_details(
        &self,Parameters(StructRequestCustomerDetails { customer_id }): Parameters<StructRequestCustomerDetails>) 
            -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            r#"{"Full Name": "Company A", "address": "Sunny Street"}"#,
        )]))
    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service.")]
    async fn scrape_url(
        &self, Parameters(StructRequestUrlToScrape { url_to_scrape }): Parameters<StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        let jina_ai_url = format!("https://r.jina.ai/{}", url_to_scrape);
        let client = reqwest::Client::new();
        let response = client.get(&jina_ai_url).send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        let body = response.text().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        Ok(CallToolResult::success(vec![Content::text(body)]))
    }



}

const_string!(Echo = "echo");
//#[tool(tool_box)]
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
            instructions: Some("This server provides  a function 'get_current_weather' to retrieve weather from a specific location,  'get_customer_details' to get info about a customer, and 'scrape_url' to scrape a given URL.".to_string()),
        }
    }

  
}