use rmcp::{
    ErrorData as McpError,  model::*, schemars,
    tool, 
    handler::server::wrapper::Parameters,
};

use serde_json::json;

use lopdf::{Document, Object};

static DUCKDUCK_SEARCH_URL_PART1: &str = r#"https://api.duckduckgo.com/?q="#;

static DUCKDUCK_SEARCH_URL_PART2: &str = r#"&format=json"#;

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructPdfExtraction {
    #[schemars(description = "An Url from a pdf you want to extract data from")]
    pub pdf_url: String,
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
    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service")]
    pub async fn scrape_url(
        Parameters(StructRequestUrlToScrape { url_to_scrape }): Parameters<StructRequestUrlToScrape>
    ) -> Result<CallToolResult, McpError> {
        let jina_ai_url = format!("{}/{}",JINA_AI_URL, url_to_scrape);
        let client = reqwest::Client::new();
        let response = client.get(&jina_ai_url).send().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        let body = response.text().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": url_to_scrape.to_string()}))))?;
        Ok(CallToolResult::success(vec![Content::text(body)]))
    }

    // Limit the size of the search query to 1000 tokens
    #[tool(description = "Search for an entity on the internet")]
    pub async fn search(
        Parameters(StructRequestSearch { search_query }): Parameters<StructRequestSearch>
    ) -> Result<CallToolResult, McpError> {
    
        let duckduckgo_url = format!("{}{}{}",DUCKDUCK_SEARCH_URL_PART1, &search_query,DUCKDUCK_SEARCH_URL_PART2);

        let client = reqwest::Client::new();

        let response =client
        .get(&duckduckgo_url)
        .send()
        .await
        .map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": duckduckgo_url.to_string()}))))?;

        let parsed_json_response: serde_json::Value = response.json().await.map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": duckduckgo_url.to_string()}))))?;

        let extract_from_response=    if let Some(extract_text) = parsed_json_response["Abstract"].as_str() {
                extract_text
            } else {
                "'extract' field not found or not a string."
            };

        let result = format!("Search result for '{}' : '{}' ", search_query, extract_from_response);

        Ok(CallToolResult::success(vec![Content::text(result)]))

    }

    #[tool(description = "Scrapes a given URL using Jina AI's web scraping service")]
    pub async fn pdf_extract( Parameters(StructPdfExtraction { pdf_url }): Parameters<StructPdfExtraction> ) -> Result<CallToolResult, McpError> {

        let doc = Document::load(&pdf_url)
            .map_err(|e| McpError::invalid_request(e.to_string(),Some(json!({"messages": pdf_url.to_string()}))))?;

        let mut pdf_text = String::new();
        let pages = doc.get_pages();
        for (page_number, page_id) in pages.iter() {
            // Placeholder: just output the page number and object ID
            let page_text = format!("Page {} object ID: {:?}", page_number, page_id);
            pdf_text.push_str(&page_text);
            pdf_text.push('\n');
        };

        Ok(CallToolResult::success(vec![Content::text(pdf_text)]))
    }


}
