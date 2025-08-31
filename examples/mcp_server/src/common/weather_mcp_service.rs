use rmcp::{
    ErrorData as McpError,  ServerHandler,  model::*, schemars,
    tool,  tool_handler, tool_router,
    handler::server::{router::tool::ToolRouter,wrapper::Parameters},
};

//use rmcp::{handler::server::{tool::CallToolHandler},};


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequestLocation {
    #[schemars(description = "Location for which you desire to know weather")]
    pub _location: String,
    #[schemars(description = "Temperature unit to use. You can specify Degree Celsius or Degree Farenheit")]
    pub unit: Option<String>,
}

#[derive(Clone)]
pub struct WeatherMcpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl WeatherMcpService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get the current weather in a given location")]
    async fn get_current_weather(
        &self, Parameters(StructRequestLocation { _location, unit }): Parameters<StructRequestLocation>) -> Result<CallToolResult, McpError> {
        let unit = unit.unwrap_or("Degree Celsius".to_string());
        let begining_string=r#""{"Temperature": "24", "unit":""#;
        let end_string=r#"","description":"Sunny"}"#;
        Ok(CallToolResult::success(vec![Content::text(
            format!("{}{}{}",begining_string,unit,end_string),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for WeatherMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a function 'get_current_weather' to retrieve weather from a specific location.".to_string()),
        }
    }
}