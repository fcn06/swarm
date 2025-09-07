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

/// Agent that executes predefined workflows.
#[allow(dead_code)]
#[derive(Clone)]
pub struct WorkFlowAgent {
    agent_config: Arc<AgentConfig>,
    llm_interaction: ChatLlmInteraction,
    workflow_runners: Arc<WorkFlowRunners>,
    discovery_service: Arc<dyn DiscoveryService>,
}

#[async_trait]
impl Agent for WorkFlowAgent {
    async fn new(
        agent_config: AgentConfig,
        _evaluation_service: Option<Arc<dyn EvaluationService>>,
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
        })
    }


    // Use metadata to choose between workflow, high level plan, workflow
    async fn handle_request(&self, request: LlmMessage,metadata:Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult> {

        let request_id = Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();
        let user_query = request.content.clone().unwrap_or_default();

        debug!("---WorkflowAgent: Starting to handle user request -- Query: \'{}\'---", user_query);

        if self.extract_high_level_plan_flag(metadata.clone()) {
            info!("High level plan requested. Creating high level plan.");
            let high_level_plan = self.create_high_level_plan(user_query).await?;
            info!("High level plan: {}", high_level_plan);
            // Directly return the high-level plan without further execution
            return Ok(ExecutionResult {
                request_id,
                conversation_id,
                success: true,
                output: high_level_plan,
            });
        }

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
                user_query.clone(), // Pass user_query here
            );
        
        match executor.execute_plan().await {
            Ok(execution_outcome) => {
                info!("Workflow execution completed successfully.");
                // You might want to return a more specific ExecutionResult based on execution_outcome
                Ok(ExecutionResult { 
                    request_id: request_id.clone(), 
                    conversation_id: conversation_id.clone(), 
                    success: true, 
                    output: format!("Workflow executed successfully. Outcome: {:?}", execution_outcome), 
                })
            },
            Err(e) => {
                warn!("Error executing plan: {}", e);
                Err(anyhow::anyhow!("Workflow execution failed: {}", e))
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
            .replacen("{}",&user_query, 1)
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
        //print!("WorkFlow Generated: {} \n", json_string);

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
                .replacen("{}",&user_query, 1)
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
