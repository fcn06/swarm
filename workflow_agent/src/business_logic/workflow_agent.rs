use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, error,warn};
use uuid::Uuid;
use std::env;
use anyhow::{Context,bail};

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

        debug!("---WorkflowAgent: Starting to handle user request -- Query: \'{}\'---", user_query);

        if self.extract_high_level_plan_flag(metadata.clone()) {
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
                    agent_output:high_level_plan.clone(),
                    context_snapshot:None,
                    success_criteria:None,
                };

                let _ = eval_service.log_evaluation(data).await;
            }


            // Directly return the high-level plan without further execution
            return Ok(ExecutionResult {
                request_id,
                conversation_id,
                success: true,
                output: high_level_plan,
            });
        }

        loop {
            let graph = if let Some(graph_file) = self.extract_workflow_filename(metadata.clone()) {
                info!("Loading workflow from file: {}", graph_file);
                load_graph_from_file(&graph_file)
                    .map_err(|e| {
                        error!("Error loading workflow from file: {}", e);
                        anyhow::anyhow!("Failed to load workflow from file: {}", e)
                    })?
            } else {
                info!("No workflow file specified in metadata, creating workflow dynamically.");
                self.create_plan(user_query.clone()).await
                    .map_err(|e| {
                        error!("Error creating dynamic plan: {}", e);
                        anyhow::anyhow!("Failed to create dynamic plan: {}", e)
                    })?
            };
            debug!("Graph Generated: {:#?}", graph);
      
            let mut executor =
                PlanExecutor::new(
                    graph,
                    self.workflow_runners.task_runner.clone(),
                    self.workflow_runners.agent_runner.clone(),
                    self.workflow_runners.tool_runner.clone(),
                    original_user_query.clone(), // Pass original_user_query here
                );
            
            match executor.execute_plan().await {
                Ok(execution_outcome) => {
                   
                    let mut parsed_outcome_map = serde_json::Map::new();
                    for (key, value_string) in execution_outcome.into_iter() {
                        // Try to parse the value string as JSON, if it fails, keep it as a plain string
                        let parsed_value = serde_json::from_str(&value_string)
                            .unwrap_or_else(|_| serde_json::Value::String(value_string));
                        parsed_outcome_map.insert(key, parsed_value);
                    }
                    let output = serde_json::to_string(&serde_json::Value::Object(parsed_outcome_map))
                        .context("Failed to serialize processed execution outcome to JSON")?;
    
                    debug!("\nWorkflow execution completed successfully. Outcome : {}\n", output);
    
                    if let Some(eval_service) = &self.evaluation_service {
                        let data=AgentEvaluationLogData{
                            agent_id:self.agent_config.agent_name.clone(),
                            request_id:request_id.clone(),
                            conversation_id:conversation_id.clone(),
                            step_id:None,
                            original_user_query:original_user_query.clone(),
                            agent_input:user_query.clone(),
                            agent_output: output.clone(),
                            context_snapshot:None,
                            success_criteria:None,
                        };
                        let evaluation = eval_service.log_evaluation(data).await;
                        match evaluation {
                            Ok(eval) => {
                                
                                debug!("\nHere is the feedback :{}\n",eval.feedback);

                                if eval.score < 5 && retry_count < MAX_RETRIES {
                                    warn!("Evaluation score is low ({}). Retrying...", eval.score);
                                    retry_count += 1;
                                    user_query = format!("{} (Previous attempt failed with feedback: {})", original_user_query, eval.feedback);
                                    continue; // Retry the loop with the modified user_query
                                } else if eval.score < 5 {
                                    error!("Evaluation score is low ({}) and max retries reached. Aborting.", eval.score);
                                    bail!("Workflow execution failed after multiple retries due to low evaluation score.");
                                }
                            },
                            Err(e) => {
                                error!("Error during evaluation logging: {}", e);
                            }
                        }
                    }
    
                    // You might want to return a more specific ExecutionResult based on execution_outcome
                    return Ok(ExecutionResult { 
                        request_id: request_id.clone(), 
                        conversation_id: conversation_id.clone(), 
                        success: true, 
                        output, 
                    })
                },
                Err(e) => {
                    warn!("Error executing plan: {}", e);
                    let error_message = format!("Workflow execution failed: {}", e);
    
                    if let Some(eval_service) = &self.evaluation_service { 
                        let data=AgentEvaluationLogData{
                            agent_id:self.agent_config.agent_name.clone(),
                            request_id:request_id.clone(),
                            conversation_id:conversation_id.clone(),
                            step_id:None,
                            original_user_query:original_user_query.clone(),
                            agent_input:user_query.clone(),
                            agent_output: error_message.clone(),
                            context_snapshot:None,
                            success_criteria:None,
                        };
                        let evaluation = eval_service.log_evaluation(data).await;
                        match evaluation {
                            Ok(eval) => {
                                if eval.score < 5 && retry_count < MAX_RETRIES {
                                    warn!("Evaluation score is low ({}). Retrying...", eval.score);
                                    retry_count += 1;
                                    user_query = format!("{} (Previous attempt failed with feedback: {})", original_user_query, eval.feedback);
                                    continue; // Retry the loop with the modified user_query
                                } else if eval.score < 5 {
                                    error!("Evaluation score is low ({}) and max retries reached. Aborting.", eval.score);
                                    bail!("Workflow execution failed after multiple retries due to low evaluation score.");
                                }
                            },
                            Err(e) => {
                                error!("Error during evaluation logging: {}", e);
                            }
                        }
                    }
    
                    return Err(anyhow::anyhow!("{}", error_message))
                }
            }
        }
    }

}


impl WorkFlowAgent {

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
