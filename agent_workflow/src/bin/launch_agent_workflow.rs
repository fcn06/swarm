use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio;

use agent_workflow::AgentWorkflow;
use mcp_runtime::mcp_server::McpServer;
use mcp_runtime::mcp_client::McpClient;
use workflow_management::agents::agent_registry::AgentRegistry;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;

// Dummy BasicAgentRunner and related types for registration
// In a real scenario, these would come from the basic_agent crate
mod basic_agent_dummy {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use anyhow::Result;
    use workflow_management::graph::graph_definition::Activity;
    use workflow_management::agents::agent_runner::AgentRunner;
    use log::info;
    use serde_json::Value;

    pub struct DummyBasicAgentRunner;

    #[async_trait]
    impl AgentRunner for DummyBasicAgentRunner {
        fn name(&self) -> String {
            "Basic_Agent".to_string()
        }

        async fn invoke(&self, activity: &Activity) -> Result<String> {
            info!("DummyBasicAgentRunner invoked for activity: {}", activity.id);
            // Simulate agent behavior based on activity ID
            match activity.id.as_str() {
                "fetch_customer_data" => {
                    // Access tool_parameters correctly
                    let customer_id = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("customer_id"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("unknown");
                    Ok(format!("Okay, I have fetched the customer details for customer ID {}. The customer's full name is Company A and their address is Sunny Street BOSTON.", customer_id))
                },
                "fetch_weather_forecast" => {
                    let location = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("location"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("unknown");
                    Ok(format!("The weather forecast for {} is sunny with a high of 25C.", location))
                },
                "compose_personalized_message" => {
                    let customer_name = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("customer_name"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("Customer");
                    let weather_forecast = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("weather_forecast"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("no forecast");
                    Ok(format!("Hello {}. Your personalized weather update: {}. Have a great day!", customer_name, weather_forecast))
                },
                "send_notification" => {
                    let email_address = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("email_address"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("unknown@example.com");
                    let message = activity.tool_parameters
                        .as_ref()
                        .and_then(|params| params.as_object())
                        .and_then(|obj| obj.get("message"))
                        .and_then(|val| val.as_str())
                        .unwrap_or("empty message");
                    Ok(format!("Email sent to {} with message: '{}'", email_address, message))
                },
                _ => Err(anyhow::anyhow!("Unknown activity for DummyBasicAgentRunner"))
            }
        }
    }

    pub struct DummyWeatherApiTool;

    #[async_trait]
    impl workflow_management::tools::tool_runner::ToolRunner for DummyWeatherApiTool {
        fn name(&self) -> String {
            "weather_api".to_string()
        }

        async fn run(&self, parameters: &Value) -> Result<String> {
            let location = parameters.get("location").and_then(|v| v.as_str()).unwrap_or("unknown");
            info!("DummyWeatherApiTool invoked for location: {}", location);
            Ok(format!("The weather in {} is 25C and sunny.", location))
        }
    }

    pub struct DummyEmailSenderTool;

    #[async_trait]
    impl workflow_management::tools::tool_runner::ToolRunner for DummyEmailSenderTool {
        fn name(&self) -> String {
            "email_sender".to_string()
        }

        async fn run(&self, parameters: &Value) -> Result<String> {
            let email = parameters.get("email_address").and_then(|v| v.as_str()).unwrap_or("unknown");
            let message = parameters.get("message").and_then(|v| v.as_str()).unwrap_or("empty");
            info!("DummyEmailSenderTool sending email to {} with message: {}", email, message);
            Ok(format!("Email successfully sent to {}.", email))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Launching agent_workflow...");

    // Start MCP Server
    let mcp_server = McpServer::new("127.0.0.1:8080".to_string());
    let mcp_server_handle = tokio::spawn(async move { mcp_server.start().await });

    // Create registries
    let task_registry = Arc::new(TaskRegistry::new());
    let mut agent_registry = AgentRegistry::new();
    let tool_registry = Arc::new(ToolRegistry::new());

    // Register the DummyBasicAgentRunner with the AgentRegistry
    agent_registry.register(Box::new(basic_agent_dummy::DummyBasicAgentRunner));

    // Register dummy tools
    tool_registry.register(Box::new(basic_agent_dummy::DummyWeatherApiTool));
    tool_registry.register(Box::new(basic_agent_dummy::DummyEmailSenderTool));

    // Create MCP Client
    let mcp_client = McpClient::new("http://127.0.0.1:8080".to_string());

    // Create AgentWorkflow instance
    let agent_workflow = AgentWorkflow::new(
        Arc::clone(&task_registry),
        Arc::new(agent_registry), // Wrap in Arc for shared ownership
        Arc::clone(&tool_registry),
        mcp_client,
    );

    // Register AgentWorkflow with MCP Server (conceptual, actual registration might differ based on mcp_runtime)
    // For demonstration, we'll directly call its handle_request method.
    info!("AgentWorkflow initialized and ready to receive requests.");

    // Simulate a user request by directly calling handle_request on AgentWorkflow
    // In a real system, this would come from an external client via the MCP server.
    let dummy_task_id = "user_workflow_request_123".to_string();
    let dummy_input = "Please create and execute the personalized weather notification workflow.";

    match agent_workflow.handle_request(dummy_task_id, dummy_input).await {
        Ok(response) => info!("AgentWorkflow Response: {}", response),
        Err(e) => error!("Error from AgentWorkflow: {}", e),
    }

    // Keep the MCP server running
    mcp_server_handle.await?;

    Ok(())
}
