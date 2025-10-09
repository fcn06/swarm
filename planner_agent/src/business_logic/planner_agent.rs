
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, error};
use std::env;
use anyhow::{Context, bail};
use serde_json::{Map, Value};
use llm_api::chat::{ChatLlmInteraction, Message as LlmMessage};
use configuration::AgentConfig;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService};

use agent_models::graph::graph_definition::{WorkflowPlanInput,Graph};
use agent_models::execution::execution_result::{ExecutionResult};

use std::fs;
use a2a_rs::{HttpClient, domain::{Message, Part, Role, TaskState}};
use uuid::Uuid;
use a2a_rs::services::AsyncA2AClient;

use super::workflow_registry::WorkFlowRegistry;
use workflow_management::graph::config::load_graph_from_file; // Added import

static DEFAULT_WORKFLOW_PROMPT_TEMPLATE: &str = "./configuration/prompts/detailed_workflow_agent_prompt.txt";
static DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE: &str = "./configuration/prompts/high_level_plan_workflow_agent_prompt.txt";

#[allow(dead_code)]
#[derive(Clone)]
pub struct PlannerAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    workflow_registry: Arc<WorkFlowRegistry>,
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
        workflow_service: Option<Arc<dyn agent_core::business_logic::services::WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let llm_planner_api_key = env::var("LLM_PLANNER_API_KEY").expect("LLM_PLANNER_API_KEY must be set");

        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            llm_planner_api_key,
        );
        
        let discovery_service = discovery_service
            .ok_or_else(|| anyhow::anyhow!("DiscoveryService not provided"))?;

        let workflow_registry = workflow_service
        .and_then(|ws| ws.as_any().downcast_ref::<WorkFlowRegistry>().map(|wr| Arc::new(wr.clone())))
        .ok_or_else(|| anyhow::anyhow!("WorkFlowRegistry not provided or invalid type"))?;

        let executor_url=agent_config.agent_executor_url().unwrap();

        Ok(Self {
            agent_config: Arc::new(agent_config),
            llm_interaction,
            workflow_registry,
            discovery_service,
            client: Arc::new(HttpClient::new(executor_url)),
        })
    }

    async fn handle_request(&self, request: LlmMessage, metadata: Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult> {
        let user_query = request.content.clone().unwrap_or_default();
        
        let graph = if let Some(graph_file) = self.extract_workflow_filename(metadata.clone()) {
            info!("PlannerAgent: Loading workflow from file: {}", graph_file);
            load_graph_from_file(&graph_file)
                .map_err(|e| {
                    error!("PlannerAgent: Error loading workflow from file: {}", e);
                    anyhow::anyhow!("Failed to load workflow from file: {}", e)
                })?
        } else if self.extract_high_level_plan_flag(metadata.clone()) {
            let plan = self.create_high_level_plan(user_query).await?;
            return Ok(ExecutionResult {
                request_id: "".to_string(),
                conversation_id: "".to_string(),
                success: true,
                output: serde_json::Value::String(plan),
            })
        } else {
            info!("PlannerAgent: No workflow file specified in metadata, creating workflow dynamically.");
            self.create_plan(user_query).await?
        };

        let graph_json = serde_json::to_string(&graph)?;
        
        let task_id = format!("task-{}", Uuid::new_v4());
        let message_id = Uuid::new_v4().to_string();

        let a2a_message = Message::builder()
            .role(Role::User)
            .parts(vec![Part::Text {
                text: graph_json,
                metadata: metadata,
            }])
            .message_id(message_id)
            .build();

        let executor_agent_url=self.agent_config.agent_executor_url().unwrap();
        debug!("Sending plan to executor agent at: {}. Task ID: {}", executor_agent_url, task_id);
        
        let task_response = self.client
            .send_task_message(&task_id, &a2a_message, None, Some(50)) // Using A2A client
            .await?;
        
        info!("Executor agent response status for task {}: {:?}", task_id, task_response.status.state);

        if task_response.status.state == TaskState::Completed {
            if let Some(response_message) = task_response.status.message {
                if let Some(Part::Text { text, .. }) = response_message.parts.into_iter().next() {
                    Ok(ExecutionResult {
                        request_id: Uuid::new_v4().to_string(),
                        conversation_id: Uuid::new_v4().to_string(),
                        success: true,
                        output: serde_json::from_str(&text).context("Failed to parse executor agent response into serde_json::Value")?,
                    })
                } else {
                    bail!("Executor agent responded with non-text content or empty message for task {}", task_id);
                }
            } else {
                bail!("Executor agent responded with no message for task {}", task_id);
            }
        } else {
            let error_detail = task_response.status.message.and_then(|msg| {
                msg.parts.into_iter().filter_map(|part| {
                    if let Part::Text { text, .. } = part {
                        Some(text)
                    }
                    else {
                        None
                    }
                }).next()
            }).unwrap_or_else(|| "Unknown error".to_string());
            bail!("Executor agent task {} failed with state: {:?} and error: {}", task_id, task_response.status.state, error_detail);
        }
    }
}

impl PlannerAgent {
    async fn get_available_capabilities(&self) -> anyhow::Result<String> {
        let discovered_agents = self.discovery_service.discover_agents().await?;
        
        let mut capabilities = String::new();
        
        let agent_details: Vec<String> = discovered_agents.into_iter()
            .map(|agent| format!("- Agent: '{}', Purpose: '{}'", agent.name, agent.description))
            .collect();

        if !agent_details.is_empty() {
            capabilities.push_str("Available Agents: \n");
            capabilities.push_str(&agent_details.join("\n"));
        }

        debug!("Discovered Capabilities : {:#?}", capabilities);

        Ok(capabilities)
    }

    pub async fn create_plan(&self, user_query: String) -> anyhow::Result<Graph> {
        
        let capabilities = self.workflow_registry.list_available_resources();
        
        debug!("Capabilities for plan creation: \n {}", capabilities);

        let prompt_template = fs::read_to_string(DEFAULT_WORKFLOW_PROMPT_TEMPLATE)
                .context("Failed to read workflow_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", &user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for Plan creation : {}", prompt);

        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(), prompt).await?
            .context("LLM returned no content")?;
        info!("LLM responded with plan content: {:?}", response_content);

        let json_string = self.extract_json_from_response(&response_content)?;
        debug!("WorkFlow Generated: {}", json_string);

        let workflow: WorkflowPlanInput = serde_json::from_str(&json_string)?;
        Ok(workflow.into())
    }

    pub async fn create_high_level_plan(&self, user_query: String) -> anyhow::Result<String> {
        let capabilities = self.get_available_capabilities().await?;
        debug!("Capabilities for high-level plan creation: \n {}", capabilities);

        let prompt_template = fs::read_to_string(DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE)
                .context("Failed to read high_level_plan_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", &user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for high-level plan creation: {}", prompt);

        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(), prompt).await?
            .context("LLM returned no content")?;
        info!("LLM responded with high level plan content: {:?}", response_content);
        
        Ok(response_content)
    }

    // todo:to simplify, since the control : '''json{JSON}'''  is done at the source (llm_api crate)
    fn extract_json_from_response(&self, response: &str) -> anyhow::Result<String> {
        if let (Some(start_idx), Some(end_idx)) = (response.find("```json"), response.rfind("```")) {
            let start = start_idx + "```json".len();
            if start < end_idx {
                Ok(response[start..end_idx].trim().to_string())
            } else {
                bail!("Failed to extract JSON: malformed markdown block or empty content.");
            }
        } else {
            Ok(response.trim().to_string())
        }
    }

    fn extract_high_level_plan_flag(&self, metadata: Option<Map<String, Value>>) -> bool {
        metadata
            .and_then(|map| map.get("high_level_plan").cloned())
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    }

    // Added extract_workflow_filename function
    fn extract_workflow_filename(&self, metadata: Option<Map<String, Value>>) -> Option<String> {
        if let Some(map) = metadata {
            if let Some(value) = map.get("workflow_url") {
                if let Some(filename) = value.as_str() {
                    return Some(filename.to_string());
                }
            }
        }
        None
    }
}
