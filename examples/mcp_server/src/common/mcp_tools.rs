use rmcp::{
    ErrorData as McpError,  model::*, schemars,
    tool, 
    handler::server::wrapper::Parameters,
};


use serde_json::json;

static WIKIPEDIA_SEARCH_URL: &str = "https://en.wikipedia.org/api/rest_v1/page/summary";
static JINA_AI_URL: &str = "https://r.jina.ai";


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
    #[schemars(description = "An entity ( a person, a country, an animal) to search for")]
    pub search_query: String,
}


pub struct McpTools;

impl McpTools {
    #[tool(description = "Get the current weather in a given location")]
    pub async fn get_current_weather(
        Parameters(StructRequestLocation { location, unit }): Parameters<StructRequestLocation>) -> Result<CallToolResult, McpError> {
        let _location = location;  
        let unit = unit.unwrap_or("Degree Celsius".to_string());
        let result_value = json!({
            "Temperature": "24",
            "unit": unit,
            "description": "Sunny"
        });
        Ok(CallToolResult::success(vec![Content::text(result_value.to_string())]))
        //Ok(CallToolResult::success(vec![Content::json(result_value)]))
    }


    #[tool(description = "Give customer details")]
    pub async fn get_customer_details(
        Parameters(StructRequestCustomerDetails { customer_id }): Parameters<StructRequestCustomerDetails>) 
            -> Result<CallToolResult, McpError> {
        let _customer_id = customer_id;
        let result_value = json!({
            "Full Name": "Company A",
            "address": "Sunny Street BOSTON"
        });
        Ok(CallToolResult::success(vec![Content::text(result_value.to_string())]))
        //Ok(CallToolResult::success(vec![Content::json(result_value)]))
    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service.")]
    pub async fn scrape_url(
        Parameters(StructRequestUrlToScrape { url_to_scrape }): Parameters<StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        let jina_ai_url = format!("{}/{}",JINA_AI_URL, url_to_scrape);
        let client = reqwest::Client::new();
        let response = client.get(&jina_ai_url).send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        let body = response.text().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        Ok(CallToolResult::success(vec![Content::text(body)]))
    }

    #[tool(description = "Search for an entity on wikipedia.")]
    pub async fn search(
        Parameters(StructRequestSearch { search_query }): Parameters<StructRequestSearch>
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

        let result = format!("Search result for '{}' : '{}' ", search_query, extract_from_response);

        Ok(CallToolResult::success(vec![Content::text(result)]))

    }
}
