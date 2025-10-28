use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, error, warn};
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
use std::collections::HashMap;

use workflow_management::graph::config::load_graph_from_file;
use agent_models::evaluation::evaluation_models::{AgentEvaluationLogData};

const DEFAULT_WORKFLOW_PROMPT_TEMPLATE: &str = "./configuration/prompts/detailed_workflow_agent_prompt.txt";
const DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE: &str = "./configuration/prompts/high_level_plan_workflow_agent_prompt.txt";
const A2A_TIMEOUT_SECONDS: u32 = 50;
const MAX_RETRIES: u8 = 3;
const TRIGGER_RETRY: u8 = 3;

#[derive(Clone)]
pub struct PlannerAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    discovery_service: Arc<dyn DiscoveryService>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    client: Arc<HttpClient>,
    // add dynamically executor.url
}

#[async_trait]
impl Agent for PlannerAgent {
    async fn new(
        agent_config: AgentConfig,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
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
            evaluation_service,
            client: Arc::new(HttpClient::new(executor_url)),
        })
    }

    async fn handle_request(&self, request: LlmMessage, metadata: Option<Map<String, Value>>) -> Result<ExecutionResult> {
        let mut user_query = request.content.clone().unwrap_or_default();
        let original_user_query = user_query.clone();
        let request_id = Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();

        debug!("---PlannerAgent: Starting to handle user request -- Query: \'{}\'---", user_query);

        let planning_strategy = self.determine_planning_strategy(&metadata)?;

        match planning_strategy {
            PlanningStrategy::FromFile(file_path) => {
                self.execute_from_file(
                    &file_path,
                    &original_user_query,
                    &request_id,
                    &conversation_id,
                ).await
            },
            PlanningStrategy::HighLevel => {
                let plan = self.create_high_level_plan(&user_query).await?;
                Ok(ExecutionResult {
                    request_id,
                    conversation_id,
                    success: true,
                    output: serde_json::Value::String(plan),
                })
            },
            PlanningStrategy::Dynamic => {
                let mut retry_count = 0;
                loop {
                    match self.execute_dynamic_plan_loop_iteration(
                        &mut user_query,
                        &original_user_query,
                        &request_id,
                        &conversation_id,
                        &mut retry_count,
                    ).await? {
                        Some(result) => return Ok(result),
                        None => { /* continue loop for retry */ },
                    }
                }
            }
        }
    }
}

impl PlannerAgent {
    async fn execute_from_file(
        &self,
        file_path: &str,
        original_user_query: &str,
        request_id: &str,
        conversation_id: &str,
    ) -> anyhow::Result<ExecutionResult> {
        info!("PlannerAgent: Loading workflow from file: {}", file_path);
        let graph = load_graph_from_file(file_path)
            .with_context(|| format!("Failed to load workflow from file: {}", file_path))?;
        
        let execution_result = self.internal_execute_plan(graph, original_user_query, request_id, conversation_id).await;

        let agent_output_string = match &execution_result {
            Ok(result) => {
                if let Some(text) = result.output.get("text").and_then(Value::as_str) {
                    text.to_string()
                } else {
                    result.output.to_string()
                }
            },
            Err(e) => format!("Plan execution from file failed: {}", e),
        };

        match execution_result {
            Ok(result) => {
                // We still call the evaluation service to log the outcome, but we ignore the retry suggestion.
                let _ = self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    original_user_query.to_string(), // agent_input
                    agent_output_string, // agent_output
                    HashMap::new(), // activities_outcome
                    &mut 0, // retry_count is not used here
                ).await;
                Ok(result)
            },
            Err(e) => {
                warn!("Error executing plan from file: {}", e);
                let error_message = format!("Plan execution from file failed: {}", e);

                // Log the error outcome
                let _ = self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    original_user_query.to_string(), // agent_input
                    error_message.clone(), // agent_output
                    HashMap::new(), // activities_outcome
                    &mut 0, // retry_count is not used here
                ).await;
                Err(anyhow::anyhow!("{}", error_message))
            }
        }
    }

    async fn execute_dynamic_plan_loop_iteration(
        &self,
        user_query: &mut String,
        original_user_query: &str,
        request_id: &str,
        conversation_id: &str,
        retry_count: &mut u8,
    ) -> anyhow::Result<Option<ExecutionResult>> {
        info!("PlannerAgent: No workflow file specified in metadata, creating workflow dynamically.");
        let graph = self.create_plan(user_query).await?;
        
        let execution_result = self.internal_execute_plan(graph, original_user_query, request_id, conversation_id).await;

        let agent_output_string = match &execution_result {
            Ok(result) => {
                if let Some(text) = result.output.get("text").and_then(Value::as_str) {
                    text.to_string()
                } else {
                    result.output.to_string()
                }
            },
            Err(e) => format!("Dynamic plan execution failed: {}", e),
        };

        match execution_result {
            Ok(result) => {
                match self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    user_query.clone(), // agent_input
                    agent_output_string, // agent_output
                    HashMap::new(), // activities_outcome
                    retry_count,
                ).await? {
                    Some(new_user_query) => {
                        *user_query = new_user_query;
                        Ok(None)
                    },
                    None => Ok(Some(result)),
                }
            },
            Err(e) => {
                warn!("Error executing dynamic plan: {}", e);
                let error_message = format!("Dynamic plan execution failed: {}", e);

                match self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    user_query.clone(), // agent_input
                    error_message.clone(), // agent_output
                    HashMap::new(), // activities_outcome
                    retry_count,
                ).await? {
                    Some(new_user_query) => {
                        *user_query = new_user_query;
                        Ok(None)
                    },
                    None => Err(anyhow::anyhow!("{}", error_message)),
                }
            }
        }
    }

    async fn internal_execute_plan(&self, graph: Graph, original_user_query: &str, request_id: &str, conversation_id: &str) -> Result<ExecutionResult> {
        let graph_json = serde_json::to_string(&graph)
            .context("Failed to serialize graph to JSON")?;
        
        let task_id = format!("task-{}", Uuid::new_v4());
        let message_id = Uuid::new_v4().to_string();

        let a2a_message = Message::builder()
            .role(Role::User)
            .parts(vec![Part::Text {
                text: graph_json,
                metadata: Some(self.extract_metadata_for_executor(original_user_query, request_id, conversation_id)),
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
                    request_id: request_id.to_string(),
                    conversation_id: conversation_id.to_string(),
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

    async fn handle_evaluation_and_retry(
        &self,
        request_id: &str,
        conversation_id: &str,
        original_user_query: &str,
        agent_input: String,
        agent_output: String,
        activities_outcome: HashMap<String, String>,
        retry_count: &mut u8,
    ) -> anyhow::Result<Option<String>> { 
        if let Some(eval_service) = &self.evaluation_service {
            let data = AgentEvaluationLogData {
                agent_id: self.agent_config.agent_name.clone(),
                request_id: request_id.to_string(),
                conversation_id: conversation_id.to_string(),
                step_id: None,
                original_user_query: original_user_query.to_string(),
                agent_input,
                activities_outcome,
                agent_output: agent_output.clone(),
                context_snapshot: None,
                success_criteria: None,
            };
            let evaluation = eval_service.log_evaluation(data).await;
            match evaluation {
                Ok(eval) => {
                    debug!("\nHere is the feedback : {}\n", eval.feedback);
                    if eval.score < TRIGGER_RETRY && *retry_count < MAX_RETRIES {
                        warn!("Evaluation score is low ({}). Retrying...", eval.score);
                        *retry_count += 1;
                        let new_user_query = format!("{} (Previous attempt failed with feedback: {})", original_user_query, eval.feedback);
                        Ok(Some(new_user_query))
                    } else if eval.score < TRIGGER_RETRY {
                        error!("Evaluation score is low ({}) and max retries reached. Aborting.", eval.score);
                        bail!("Plan execution failed after multiple retries due to low evaluation score.");
                    } else {
                        Ok(None)
                    }
                },
                Err(e) => {
                    error!("Error during evaluation logging: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
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

    //todo: to remove
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
        // The markdown block should already be removed by llm_api::chat::remove_think_tags
        // so we just need to parse the response as a JSON string.
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

    // Helper to extract relevant metadata for the executor agent
    fn extract_metadata_for_executor(&self, original_user_query: &str, request_id: &str, conversation_id: &str) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("original_user_query".to_string(), Value::String(original_user_query.to_string()));
        map.insert("request_id".to_string(), Value::String(request_id.to_string()));
        map.insert("conversation_id".to_string(), Value::String(conversation_id.to_string()));
        map
    }
}

enum PlanningStrategy {
    FromFile(String),
    HighLevel,
    Dynamic,
}