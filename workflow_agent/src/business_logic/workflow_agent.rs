use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, error,warn};
use uuid::Uuid;
use std::env;
use anyhow::{Context,bail};
use std::collections::HashMap;

use serde_json::Map;
use serde_json::Value;

use llm_api::chat::{ChatLlmInteraction};
use llm_api::chat::Message as LlmMessage;

use crate::business_logic::workflow_runners::WorkFlowRunners;

use configuration::AgentConfig;

use agent_core::business_logic::services::EvaluationService;
use agent_evaluation_service::evaluation_server::judge_agent::{AgentEvaluationLogData};

use agent_core::business_logic::services::MemoryService;
use agent_core::business_logic::services::DiscoveryService;
use agent_core::business_logic::services::WorkflowServiceApi;

use workflow_management::graph::config::load_graph_from_file;


use workflow_management::graph::{ graph_orchestrator::PlanExecutor};

use agent_core::execution::execution_result::ExecutionResult;

use std::fs;

use agent_core::graph::graph_definition::{WorkflowPlanInput,Graph};
use agent_core::business_logic::agent::Agent;

static DEFAULT_WORKFLOW_PROMPT_TEMPLATE: &str = "./configuration/prompts/detailed_workflow_agent_prompt.txt";
static DEFAULT_HIGH_LEVEL_PLAN_PROMPT_TEMPLATE: &str = "./configuration/prompts/high_level_plan_workflow_agent_prompt.txt";
const MAX_RETRIES: u8 = 3;
const TRIGGER_RETRY: u8 = 3;

/// Agent that executes predefined workflows.
#[allow(dead_code)]
#[derive(Clone)]
pub struct WorkFlowAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    workflow_runners: Arc<WorkFlowRunners>,
    discovery_service: Arc<dyn DiscoveryService>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
}

#[async_trait]
impl Agent for WorkFlowAgent {
    async fn new(
        agent_config: AgentConfig,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service: Option<Arc<dyn DiscoveryService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {

        let llm_workflow_api_key = env::var("LLM_WORKFLOW_API_KEY").expect("LLM_WORKFLOW_API_KEY must be set");

        let llm_interaction = ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            agent_config.agent_model_id(),
            llm_workflow_api_key,
        );

        let workflow_runners = workflow_service
            .and_then(|ws| ws.as_any().downcast_ref::<WorkFlowRunners>().map(|wr| Arc::new(wr.clone())))
            .ok_or_else(|| anyhow::anyhow!("WorkFlowRunnerss not provided or invalid type"))?;

        let discovery_service = discovery_service
            .ok_or_else(|| anyhow::anyhow!("DiscoveryService not provided"))?;

        Ok(Self {
            agent_config: Arc::new(agent_config),
            llm_interaction,
            workflow_runners,
            discovery_service,
            evaluation_service,
        })
    }


    // Use metadata to choose between workflow, high level plan, workflow
    async fn handle_request(&self, request: LlmMessage,metadata:Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult> {

        let request_id = Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();
        let mut user_query = request.content.clone().unwrap_or_default();
        let original_user_query = user_query.clone();
        let mut retry_count = 0;

        debug!("---WorkflowAgent: Starting to handle user request -- Query: '{}'---", user_query);

        if self.extract_high_level_plan_flag(metadata.clone()) {
            return self.handle_high_level_plan_request(user_query, request_id, conversation_id).await;
        }

        loop {
            match self.execute_workflow_loop_iteration(
                metadata.clone(),
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


impl WorkFlowAgent {

    async fn execute_workflow_loop_iteration(
        &self,
        metadata: Option<Map<String, Value>>,
        user_query: &mut String,
        original_user_query: &str,
        request_id: &str,
        conversation_id: &str,
        retry_count: &mut u8,
    ) -> anyhow::Result<Option<ExecutionResult>> {
        let graph = self.get_workflow_graph(metadata, user_query.clone()).await?;
        debug!("Graph Generated: {:#?}", graph);
  
        let mut executor =
            PlanExecutor::new(
                graph,
                self.workflow_runners.task_runner.clone(),
                self.workflow_runners.agent_runner.clone(),
                self.workflow_runners.tool_runner.clone(),
                original_user_query.to_string(),
            );
        
        match executor.execute_plan().await {
            Ok(execution_outcome) => {
                let output = execution_outcome;

                debug!("\nWorkflow execution completed successfully. Outcome : {}\n", output);

                match self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    user_query.clone(),
                    output.clone(),
                    retry_count,
                ).await? {
                    Some(new_user_query) => {
                        *user_query = new_user_query;
                        Ok(None) // Indicate retry needed
                    },
                    None => {
                        Ok(Some(ExecutionResult {
                            request_id: request_id.to_string(),
                            conversation_id: conversation_id.to_string(),
                            success: true,
                            output,
                        }))
                    }
                }
            },
            Err(e) => {
                warn!("Error executing plan: {}", e);
                let error_message = format!("Workflow execution failed: {}", e);

                match self.handle_evaluation_and_retry(
                    request_id,
                    conversation_id,
                    original_user_query,
                    user_query.clone(),
                    error_message.clone(),
                    retry_count,
                ).await? {
                    Some(new_user_query) => {
                        *user_query = new_user_query;
                        Ok(None) // Indicate retry needed
                    },
                    None => {
                        Err(anyhow::anyhow!("{}", error_message))
                    }
                }
            }
        }
    }

    async fn handle_evaluation_and_retry(
        &self,
        request_id: &str,
        conversation_id: &str,
        original_user_query: &str,
        agent_input: String,
        agent_output: String,
        retry_count: &mut u8,
    ) -> anyhow::Result<Option<String>> { // Returns Some(new_user_query) if retry, None if done
        if let Some(eval_service) = &self.evaluation_service {
            let data = AgentEvaluationLogData {
                agent_id: self.agent_config.agent_name.clone(),
                request_id: request_id.to_string(),
                conversation_id: conversation_id.to_string(),
                step_id: None,
                original_user_query: original_user_query.to_string(),
                agent_input,
                activities_outcome: HashMap::new(),
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
                        bail!("Workflow execution failed after multiple retries due to low evaluation score.");
                    } else {
                        Ok(None)
                    }
                },
                Err(e) => {
                    error!("Error during evaluation logging: {}", e);
                    // If evaluation itself fails, we still might want to proceed or abort based on other factors
                    // For now, let's just proceed without retry based on evaluation if evaluation itself failed.
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn handle_high_level_plan_request(&self, user_query: String, request_id: String, conversation_id: String) -> anyhow::Result<ExecutionResult> {
        info!("High level plan requested. Creating high level plan.");
        let high_level_plan = self.create_high_level_plan(user_query.clone()).await?;
        info!("High level plan: {}", high_level_plan);

        if let Some(eval_service) = &self.evaluation_service {
            let data=AgentEvaluationLogData{
                agent_id:self.agent_config.agent_name.clone(),
                request_id:request_id.clone(),
                conversation_id:conversation_id.clone(),
                step_id:None,
                original_user_query:user_query.clone(),
                agent_input:user_query.clone(),
                activities_outcome: HashMap::new(),
                agent_output:high_level_plan.clone(),
                context_snapshot:None,
                success_criteria:None,
            };

            let _ = eval_service.log_evaluation(data).await;
        }

        Ok(ExecutionResult {
            request_id,
            conversation_id,
            success: true,
            output: high_level_plan,
        })
    }

    async fn get_workflow_graph(&self, metadata: Option<Map<String, Value>>, user_query: String) -> anyhow::Result<Graph> {
        if let Some(graph_file) = self.extract_workflow_filename(metadata.clone()) {
            info!("Loading workflow from file: {}", graph_file);
            Ok(load_graph_from_file(&graph_file)
                .map_err(|e| {
                    error!("Error loading workflow from file: {}", e);
                    anyhow::anyhow!("Failed to load workflow from file: {}", e)
                })?)
        } else {
            info!("No workflow file specified in metadata, creating workflow dynamically.");
            Ok(self.create_plan(user_query.clone()).await
                .map_err(|e| {
                    error!("Error creating dynamic plan: {}", e);
                    anyhow::anyhow!("Failed to create dynamic plan: {}", e)
                })?)
        }
    }

    pub async fn create_plan(
        &self,
        user_query: String,
    ) -> anyhow::Result<Graph>  {

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


        // 4. Extract JSON from the LLM's response (in case it's wrapped in markdown code block)
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

        // 5. Parse the LLM's JSON response into the Workflow struct
        let workflow: WorkflowPlanInput = serde_json::from_str(&json_string)?;

        
        Ok(workflow.into())
    }
}


impl WorkFlowAgent {
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

    fn extract_high_level_plan_flag(&self, metadata: Option<Map<String, Value>>) -> bool {
        if let Some(map) = metadata {
            if let Some(value) = map.get("high_level_plan") {
                return value.as_bool().unwrap_or(false);
            }
        }
        false
    }
}


impl WorkFlowAgent {
    
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
    }
