use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug};
use std::env;
use anyhow::{Context, bail, Result};
use serde_json::{Map, Value};
use llm_api::chat::{ChatLlmInteraction, Message as LlmMessage};
use configuration::AgentConfig;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService, WorkflowServiceApi};
use agent_models::graph::graph_definition::{WorkflowPlanInput, Graph};
use agent_models::execution::execution_result::ExecutionResult;
use a2a_rs::{HttpClient, domain::{Message, Part, Role, TaskState}};
use uuid::Uuid;
use a2a_rs::services::AsyncA2AClient;
use tokio::fs as async_fs;

use workflow_management::graph::config::load_graph_from_file;

const DEFAULT_WORKFLOW_PROMPT_TEMPLATE: &str = "./configuration/prompts/detailed_workflow_agent_prompt.txt";
const DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE: &str = "./configuration/prompts/high_level_plan_workflow_agent_prompt.txt";
const A2A_TIMEOUT_SECONDS: u32 = 50;

#[derive(Clone)]
pub struct PlannerAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    discovery_service: Arc<dyn DiscoveryService>,
    client: Arc<HttpClient>,
}

#[async_trait]
impl Agent for PlannerAgent {
    async fn new(
        agent_config: AgentConfig,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> Result<Self> {
        let llm_planner_api_key = env::var("LLM_PLANNER_API_KEY")
            .context("LLM_PLANNER_API_KEY must be set")?;

        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            llm_planner_api_key,
        );
        
        let discovery_service = discovery_service
            .ok_or_else(|| anyhow::anyhow!("DiscoveryService not provided"))?;

        let executor_url = agent_config.agent_executor_url()
            .context("agent_executor_url not found in configuration")?;

        Ok(Self {
            agent_config: Arc::new(agent_config),
            llm_interaction,
            discovery_service,
            client: Arc::new(HttpClient::new(executor_url)),
        })
    }

    async fn handle_request(&self, request: LlmMessage, metadata: Option<Map<String, Value>>) -> Result<ExecutionResult> {
        let user_query = request.content.clone().unwrap_or_default();
        
        let planning_strategy = self.determine_planning_strategy(&metadata)?;

        match planning_strategy {
            PlanningStrategy::FromFile(file_path) => {
                info!("PlannerAgent: Loading workflow from file: {}", file_path);
                let graph = load_graph_from_file(&file_path)
                    .with_context(|| format!("Failed to load workflow from file: {}", file_path))?;
                self.execute_plan(graph, &metadata).await
            },
            PlanningStrategy::HighLevel => {
                let plan = self.create_high_level_plan(&user_query).await?;
                Ok(ExecutionResult {
                    request_id: "".to_string(),
                    conversation_id: "".to_string(),
                    success: true,
                    output: serde_json::Value::String(plan),
                })
            },
            PlanningStrategy::Dynamic => {
                info!("PlannerAgent: No workflow file specified in metadata, creating workflow dynamically.");
                let graph = self.create_plan(&user_query).await?;
                self.execute_plan(graph, &metadata).await
            }
        }
    }
}

impl PlannerAgent {
    async fn execute_plan(&self, graph: Graph, metadata: &Option<Map<String, Value>>) -> Result<ExecutionResult> {
        let graph_json = serde_json::to_string(&graph)
            .context("Failed to serialize graph to JSON")?;
        
        let task_id = format!("task-{}", Uuid::new_v4());
        let message_id = Uuid::new_v4().to_string();

        let a2a_message = Message::builder()
            .role(Role::User)
            .parts(vec![Part::Text {
                text: graph_json,
                metadata: metadata.clone(),
            }])
            .message_id(message_id)
            .build();

        let executor_agent_url = self.agent_config.agent_executor_url()
            .context("agent_executor_url not found in configuration")?;
        debug!("Sending plan to executor agent at: {}. Task ID: {}", executor_agent_url, task_id);
        
        let task_response = self.client
            .send_task_message(&task_id, &a2a_message, None, Some(A2A_TIMEOUT_SECONDS))
            .await
            .with_context(|| format!("Failed to send task {} to executor agent", task_id))?;
        
        info!("Executor agent response status for task {}: {:?}", task_id, task_response.status.state);

        if task_response.status.state == TaskState::Completed {
            let response_part = task_response.status.message
                .and_then(|msg| msg.parts.into_iter().next())
                .ok_or_else(|| anyhow::anyhow!("Executor agent responded with no message for task {}", task_id))?;

            if let Part::Text { text, .. } = response_part {
                let output = serde_json::from_str(&text)
                    .context("Failed to parse executor agent response into serde_json::Value")?;
                Ok(ExecutionResult {
                    request_id: Uuid::new_v4().to_string(),
                    conversation_id: Uuid::new_v4().to_string(),
                    success: true,
                    output,
                })
            } else {
                bail!("Executor agent responded with non-text content for task {}", task_id);
            }
        } else {
            let error_detail = task_response.status.message
                .and_then(|msg| {
                    msg.parts.into_iter().find_map(|part| {
                        if let Part::Text { text, .. } = part { Some(text) } else { None }
                    })
                })
                .unwrap_or_else(|| "Unknown error".to_string());
            bail!("Executor agent task {} failed with state: {:?} and error: {}", task_id, task_response.status.state, error_detail);
        }
    }

    pub async fn create_plan(&self, user_query: &str) -> Result<Graph> {
        let capabilities = self.discovery_service.list_available_resources().await?;
        debug!("Capabilities for plan creation: \n {}", capabilities);

        let prompt_template = async_fs::read_to_string(DEFAULT_WORKFLOW_PROMPT_TEMPLATE).await
                .context("Failed to read workflow_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for Plan creation : {}", prompt);

        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(), prompt).await?
            .context("LLM returned no content")?;
        info!("LLM responded with plan content: {:?}", response_content);

        let json_string = self.extract_json_from_response(&response_content)?;
        debug!("WorkFlow Generated: {}", json_string);

        let workflow: WorkflowPlanInput = serde_json::from_str(&json_string)
            .context("Failed to deserialize workflow plan from LLM response")?;
        Ok(workflow.into())
    }

    pub async fn create_high_level_plan(&self, user_query: &str) -> Result<String> {
        let capabilities = self.get_available_capabilities().await?;
        debug!("Capabilities for high-level plan creation: \n {}", capabilities);

        let prompt_template = async_fs::read_to_string(DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE).await
                .context("Failed to read high_level_plan_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for high-level plan creation: {}", prompt);

        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(), prompt).await?
            .context("LLM returned no content")?;
        info!("LLM responded with high level plan content: {:?}", response_content);
        
        Ok(response_content)
    }

    async fn get_available_capabilities(&self) -> Result<String> {
        let discovered_agents = self.discovery_service.discover_agents().await?;
        
        let agent_details: Vec<String> = discovered_agents.into_iter()
            .map(|agent| format!("- Agent: '{}', Purpose: '{}'", agent.name, agent.description))
            .collect();

        let capabilities = if !agent_details.is_empty() {
            format!("Available Agents: \n{}", agent_details.join("\n"))
        } else {
            String::new()
        };

        debug!("Discovered Capabilities : {:#?}", capabilities);

        Ok(capabilities)
    }

    fn extract_json_from_response(&self, response: &str) -> Result<String> {
        // This regex is more robust for extracting JSON from a markdown block
        let re = regex::Regex::new(r"```json\s*([\s\S]*?)\s*```").unwrap();
        if let Some(caps) = re.captures(response) {
            if let Some(json) = caps.get(1) {
                return Ok(json.as_str().trim().to_string());
            }
        }
        // Fallback for cases where there are no backticks
        Ok(response.trim().to_string())
    }

    fn determine_planning_strategy(&self, metadata: &Option<Map<String, Value>>) -> Result<PlanningStrategy> {
        if let Some(meta) = metadata {
            if let Some(file) = meta.get("workflow_url").and_then(Value::as_str) {
                return Ok(PlanningStrategy::FromFile(file.to_string()));
            }
            if meta.get("high_level_plan").and_then(Value::as_bool).unwrap_or(false) {
                return Ok(PlanningStrategy::HighLevel);
            }
        }
        Ok(PlanningStrategy::Dynamic)
    }
}

enum PlanningStrategy {
    FromFile(String),
    HighLevel,
    Dynamic,
}
