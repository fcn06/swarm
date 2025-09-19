
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug};
use std::env;
use anyhow::Context;
use std::collections::HashMap;

use serde_json::Map;
use serde_json::Value;

use llm_api::chat::{ChatLlmInteraction};
use llm_api::chat::Message as LlmMessage;

use configuration::AgentConfig;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService, WorkflowServiceApi};
use agent_core::graph::graph_definition::{Graph, WorkflowPlanInput};
use agent_core::execution::execution_result::ExecutionResult;

use std::fs;
use anyhow::bail;

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
        _workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let llm_workflow_api_key = env::var("LLM_WORKFLOW_API_KEY").expect("LLM_WORKFLOW_API_KEY must be set");

        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            llm_workflow_api_key,
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
                anyhow::bail!("Executor agent failed with status: {} and error: {}", res.status(), error_text);
            }
        }
    }
}

impl PlannerAgent {


    pub async fn create_plan(
        &self,
        user_query: String,
    ) -> anyhow::Result<Graph>  {

        // todo : Make resources list available without runners
        // 1. Get capabilities string from workflow_runners
        let capabilities = self.workflow_runners.list_available_resources();

        // 2. Format the prompt with dynamic data
        // Read the prompt template from the file
        let prompt_template = fs::read_to_string(DEFAULT_WORKFLOW_PROMPT_TEMPLATE)
                .context("Failed to read workflow_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", &user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for Plan creation : {}", prompt);

        // 3. Call the LLM API
        // This api returns raw text from llm
        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(),prompt.to_string()).await?;
        let response_content=response_content.expect("No plan created from LLM");
        info!("WorkflowAgent: LLM responded with plan content:{:?}", response_content);


        // 4. Extract JSON from the LLM\'s response (in case it\'s wrapped in markdown code block)
        let json_string = if let (Some(start_idx), Some(end_idx)) = (response_content.find("```json"), response_content.rfind("```")) {
            let start = start_idx + "```json".len();
            if start < end_idx {
                response_content[start..end_idx].trim().to_string()
            } else {
                // to be improved
                bail!("Failed to extract JSON: malformed markdown block or empty content.");
            }
        } else {
            // If no markdown block, assume the entire response is the JSON string
            response_content.trim().to_string()
        };

        debug!("WorkFlow Generated: {}", json_string);

        // 5. Parse the LLM\'s JSON response into the Workflow struct
        let workflow: WorkflowPlanInput = serde_json::from_str(&json_string)?;

        
        Ok(workflow.into())
    }



    pub async fn create_high_level_plan(
        &self,
        user_query: String,
    ) -> anyhow::Result<String>  {

        // 1. Get capabilities string from workflow_runners
        let capabilities = self.workflow_runners.list_available_resources();

        // 2. Format the prompt with dynamic data
        // Read the prompt template from the file
        let prompt_template = fs::read_to_string(DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE)
                .context("Failed to read high_level_plan_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", &user_query, 1)
            .replacen("{}", &capabilities, 1);

        debug!("Prompt for Plan creation : {}", prompt);

        // 3. Call the LLM API
        // This api returns raw text from llm
        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(),prompt.to_string()).await?;
        let response_content=response_content.expect("No plan created from LLM");
        info!("WorkflowAgent: LLM responded with high level plan content:{:?}", response_content);
        
        Ok(response_content)
    }




    fn extract_json_from_response(&self, response: &str) -> anyhow::Result<String> {
        if let (Some(start_idx), Some(end_idx)) = (response.find("```json"), response.rfind("```")) {
            let start = start_idx + "```json".len();
            if start < end_idx {
                Ok(response[start..end_idx].trim().to_string())
            } else {
                anyhow::bail!("Failed to extract JSON: malformed markdown block or empty content.")
            }
        } else {
            Ok(response.trim().to_string())
        }
    }

    fn extract_high_level_plan_flag(&self, metadata: Option<Map<String, Value>>) -> bool {
        if let Some(map) = metadata {
            if let Some(value) = map.get("high_level_plan") {
                return value.as_bool().unwrap_or(false);
            }
        }
        false
    }
}
