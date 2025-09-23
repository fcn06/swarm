
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug};
use std::env;
use anyhow::{Context, bail};
use serde_json::{Map, Value};
use llm_api::chat::{ChatLlmInteraction, Message as LlmMessage};
use configuration::AgentConfig;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService};
use agent_core::graph::graph_definition::{Graph, WorkflowPlanInput};
use agent_core::execution::execution_result::ExecutionResult;
use std::fs;

static DEFAULT_WORKFLOW_PROMPT_TEMPLATE: &str = "./configuration/prompts/detailed_workflow_agent_prompt.txt";
static DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE: &str = "./configuration/prompts/high_level_plan_workflow_agent_prompt.txt";
static EXECUTOR_AGENT_URL: &str = "http://127.0.0.1:8082/handle_request"; // TODO: Move to config

#[derive(Clone)]
pub struct PlannerAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    discovery_service: Arc<dyn DiscoveryService>,
    client: reqwest::Client,
}

#[async_trait]
impl Agent for PlannerAgent {
    async fn new(
        agent_config: AgentConfig,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service: Option<Arc<dyn DiscoveryService>>,
        _workflow_service: Option<Arc<dyn agent_core::business_logic::services::WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let llm_planner_api_key = env::var("LLM_PLANNER_API_KEY").expect("LLM_PLANNER_API_KEY must be set");

        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            llm_planner_api_key,
        );
        
        let discovery_service = discovery_service
            .ok_or_else(|| anyhow::anyhow!("DiscoveryService not provided"))?;

        Ok(Self {
            agent_config: Arc::new(agent_config),
            llm_interaction,
            discovery_service,
            client: reqwest::Client::new(),
        })
    }

    async fn handle_request(&self, request: LlmMessage, metadata: Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult> {
        let user_query = request.content.clone().unwrap_or_default();
        
        if self.extract_high_level_plan_flag(metadata) {
            let plan = self.create_high_level_plan(user_query).await?;
            Ok(ExecutionResult {
                request_id: "".to_string(),
                conversation_id: "".to_string(),
                success: true,
                output: plan,
            })
        } else {
            let graph = self.create_plan(user_query).await?;
            let graph_json = serde_json::to_string(&graph)?;
            
            let executor_request = LlmMessage {
                role: "user".to_string(),
                content: Some(graph_json),
                tool_call_id:None,
                tool_calls: None,
            };

            let res = self.client.post(EXECUTOR_AGENT_URL)
                .json(&executor_request)
                .send()
                .await?;

            if res.status().is_success() {
                let execution_result: ExecutionResult = res.json().await?;
                Ok(execution_result)
            } else {
                let error_text = res.text().await?;
                //anyhow::bail!("Executor agent failed with status: {} and error: {}", res.status(), error_text);
                anyhow::bail!("Executor agent failed with error: {}", error_text);
            }
        }
    }
}

impl PlannerAgent {
    async fn get_available_capabilities(&self) -> anyhow::Result<String> {
        // The user_query parameter for discover_agents is not needed if we are listing all agents
        let discovered_agents = self.discovery_service.discover_agents().await?;
        
        let mut capabilities = String::new();
        
        // Format discovered agents
        let agent_details: Vec<String> = discovered_agents.into_iter()
            .map(|agent| format!("- Agent: '{}', Purpose: '{}'", agent.name, agent.description))
            .collect();

        if !agent_details.is_empty() {
            capabilities.push_str("Available Agents:\n");
            capabilities.push_str(&agent_details.join("\n"));
        }

        // TODO: Add tasks from the `common_tasks` crate here later.

        Ok(capabilities)
    }

    pub async fn create_plan(&self, user_query: String) -> anyhow::Result<Graph> {
        let capabilities = self.get_available_capabilities().await?;
        debug!("Capabilities for plan creation:\n{}", capabilities);

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
        debug!("Capabilities for high-level plan creation:\n{}", capabilities);

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
}
