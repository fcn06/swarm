use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};


use serde_json::json;



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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestSearch {
    #[schemars(description = "The wikipedia search query")]
    pub search_query: String,
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
        let _location = location;  
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
        let _customer_id = customer_id;
        Ok(CallToolResult::success(vec![Content::text(
            r#"{"Full Name": "Company A", "address": "Sunny Street BOSTON"}"#,
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

    #[tool(description = "Performs a simple search")]
    async fn search(
        &self, Parameters(StructRequestSearch { search_query }): Parameters<StructRequestSearch>
    ) -> Result<CallToolResult, McpError> {

        let wikipedia_search_url = reqwest::Url::parse_with_params(
            "https://en.wikipedia.org/w/api.php",
            &[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", &search_query),
                ("format", "json"),
                ("utf8", "1"),
                ("srlimit", "5"), // Limit to 5 results
                ("srlength", "200"), // Set snippet length to 200 characters
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
            instructions: Some("This server provides  a function 'get_current_weather' to retrieve weather from a specific location,  'get_customer_details' to get info about a customer, 'scrape_url' to scrape a given URL, and 'search' to perform a simple search on wikipedia content.".to_string()),
        }
    }

  
}