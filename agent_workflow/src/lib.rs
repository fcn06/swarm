use async_trait::async_trait;
use log::{info, error};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use regex::Regex;

use agent_core::agent_interaction_protocol::AgentDelegate;
use llm_api::chat::ChatCompletionRequest;
use mcp_runtime::mcp_client::McpClient;

use workflow_management::graph::graph_definition::{Activity, ActivityType, Dependency, Graph, Node, NodeType, WorkflowPlanInput, AgentConfigInput, ToolConfigInput};
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::agents::agent_registry::AgentRegistry;
use workflow_management::tasks::task_registry::TaskRegistry;
use workflow_management::tools::tool_registry::ToolRegistry;

pub struct AgentWorkflow {
    task_registry: Arc<TaskRegistry>,
    agent_registry: Arc<AgentRegistry>,
    tool_registry: Arc<ToolRegistry>,
    mcp_client: McpClient,
}

impl AgentWorkflow {
    pub fn new(
        task_registry: Arc<TaskRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
        mcp_client: McpClient,
    ) -> Self {
        AgentWorkflow {
            task_registry,
            agent_registry,
            tool_registry,
            mcp_client,
        }
    }

    /// Lists all available skills and tools registered in the registries.
    pub fn list_available_skills_and_tools(&self) -> String {
        let skills = self.task_registry.list_skills();
        let tools = self.tool_registry.list_tools();

        let mut output = String::new();
        output.push_str("Available Skills:\n");
        for skill in skills {
            output.push_str(&format!("- {}\n", skill));
        }
        output.push_str("\nAvailable Tools:\n");
        for tool in tools {
            output.push_str(&format!("- {}\n", tool));
        }
        output
    }

    /// Dynamically creates a workflow graph based on a predefined scenario.
    /// In a real-world scenario, this would involve LLM-based planning.
    pub async fn create_dynamic_weather_notification_workflow(&self) -> Result<Graph> {
        info!("Dynamically creating Personalized Weather Notification workflow...");

        let fetch_customer_data_activity = ActivityInput {
            activity_type: ActivityType::DelegationAgent,
            id: "fetch_customer_data".to_string(),
            description: "Fetches customer 12345 data using their ID.".to_string(),
            r#type: "customer_data_access".to_string(),
            agent: AgentConfigInput {
                skill_to_use: None,
                assigned_agent_id_preference: Some("Basic_Agent".to_string()),
            },
            tools: vec![
                ToolConfigInput {
                    tool_to_use: None,
                    tool_parameters: serde_json::from_str(r#"{"customer_id": "12345"}"#).unwrap(),
                },
            ],
            tasks_parameters: HashMap::new(),
            dependencies: vec![],
            expected_outcome: "Customer data (e.g., name, address, email).".to_string(),
        };

        let fetch_weather_forecast_activity = ActivityInput {
            activity_type: ActivityType::DelegationAgent,
            id: "fetch_weather_forecast".to_string(),
            description: "Fetches the weather forecast for the customer's location.".to_string(),
            r#type: "weather_information_retrieval".to_string(),
            agent: AgentConfigInput {
                skill_to_use: None,
                assigned_agent_id_preference: Some("Basic_Agent".to_string()),
            },
            tools: vec![
                ToolConfigInput {
                    tool_to_use: Some("weather_api".to_string()),
                    tool_parameters: serde_json::from_str(r#"{"location": "{{fetch_customer_data.result.address.city}}"}"#).unwrap(),
                },
            ],
            tasks_parameters: HashMap::new(),
            dependencies: vec![
                Dependency { source: "fetch_customer_data".to_string(), condition: None },
            ],
            expected_outcome: "Weather forecast for the specified location.".to_string(),
        };

        let compose_personalized_message_activity = ActivityInput {
            activity_type: ActivityType::DelegationAgent,
            id: "compose_personalized_message".to_string(),
            description: "Composes a personalized message using customer and weather data.".to_string(),
            r#type: "content_creation".to_string(),
            agent: AgentConfigInput {
                skill_to_use: None,
                assigned_agent_id_preference: Some("Basic_Agent".to_string()),
            },
            tools: vec![
                ToolConfigInput {
                    tool_to_use: None,
                    tool_parameters: serde_json::from_str(r#"{"customer_name": "{{fetch_customer_data.result.name}}", "weather_forecast": "{{fetch_weather_forecast.result.forecast}}"}"#).unwrap(),
                },
            ],
            tasks_parameters: HashMap::new(),
            dependencies: vec![
                Dependency { source: "fetch_customer_data".to_string(), condition: None },
                Dependency { source: "fetch_weather_forecast".to_string(), condition: None },
            ],
            expected_outcome: "A personalized message for the customer.".to_string(),
        };

        let send_notification_activity = ActivityInput {
            activity_type: ActivityType::DelegationAgent,
            id: "send_notification".to_string(),
            description: "Sends the personalized message to the customer.".to_string(),
            r#type: "send_email".to_string(),
            agent: AgentConfigInput {
                skill_to_use: None,
                assigned_agent_id_preference: Some("Basic_Agent".to_string()),
            },
            tools: vec![
                ToolConfigInput {
                    tool_to_use: Some("email_sender".to_string()),
                    tool_parameters: serde_json::from_str(r#"{"email_address": "{{fetch_customer_data.result.email}}", "message": "{{compose_personalized_message.result.message}}"}"#).unwrap(),
                },
            ],
            tasks_parameters: HashMap::new(),
            dependencies: vec![
                Dependency { source: "compose_personalized_message".to_string(), condition: None },
            ],
            expected_outcome: "Email notification sent successfully.".to_string(),
        };

        let workflow_input = WorkflowPlanInput {
            plan_name: "Personalized Weather Notification".to_string(),
            activities: vec![
                fetch_customer_data_activity,
                fetch_weather_forecast_activity,
                compose_personalized_message_activity,
                send_notification_activity,
            ],
        };

        Ok(workflow_input.into())
    }

    /// Executes a given workflow graph.
    pub async fn execute_workflow_plan(&self, graph: Graph) -> Result<HashMap<String, String>> {
        info!("Executing workflow plan: {}", graph.plan_name);
        let mut executor = PlanExecutor::new(
            graph,
            Arc::clone(&self.task_registry),
            Arc::clone(&self.agent_registry),
            Arc::clone(&self.tool_registry),
        );

        executor.execute_plan().await?;
        Ok(executor.context.results)
    }
}

#[async_trait]
impl AgentDelegate for AgentWorkflow {
    fn name(&self) -> String {
        "AgentWorkflow".to_string()
    }

    async fn handle_request(
        &self,
        _task_id: String,
        _input: &str,
    ) -> Result<String, anyhow::Error> {
        info!("AgentWorkflow received a request.");
        // In a real scenario, the input would drive the dynamic workflow creation.
        // For now, we'll demonstrate by creating a predefined workflow.

        let graph = self.create_dynamic_weather_notification_workflow().await?;
        let results = self.execute_workflow_plan(graph).await?;

        info!("Workflow execution completed. Final results: {:?}", results);
        Ok(format!("Workflow executed. Results: {:?}", results))
    }

    async fn handle_tool_output(
        &self,
        _task_id: String,
        _tool_name: String,
        _output: &str,
    ) -> Result<(), anyhow::Error> {
        // AgentWorkflow typically orchestrates, so it might not directly handle tool outputs
        // unless it's designed to react to intermediate tool results.
        Ok(())
    }

    async fn handle_llm_response(
        &self,
        _task_id: String,
        _response: &ChatCompletionRequest,
    ) -> Result<(), anyhow::Error> {
        // AgentWorkflow typically orchestrates, so it might not directly handle LLM responses
        // unless it's designed to react to intermediate LLM outputs for dynamic adjustments.
        Ok(())
    }
}
