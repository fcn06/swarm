use super::tool_runner::ToolRunner;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;

/// An example tool that simulates calling a weather API.
pub struct WeatherApiTool;

#[async_trait]
impl ToolRunner for WeatherApiTool {
    fn name(&self) -> String {
        "weather_api".to_string()
    }

    async fn run(&self, params: &Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let location = params
            .get("location")
            .and_then(Value::as_str)
            .ok_or("Missing 'location' parameter")?;

        // Simulate an API call
        let forecast = match location.to_lowercase().as_str() {
            "new york" => "Sunny with a high of 75°F",
            "london" => "Cloudy with a chance of rain",
            _ => "Clear skies",
        };
        
        let result = json!({
            "location": location,
            "forecast": forecast,
        });

        Ok(result.to_string())
    }
}
