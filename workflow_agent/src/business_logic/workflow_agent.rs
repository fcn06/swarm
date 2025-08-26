use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error,warn};
use uuid::Uuid;
use std::env;

use llm_api::chat::{ChatLlmInteraction};

use crate::business_logic::workflow_runners::WorkFlowRunners;

use configuration::AgentConfig;
use agent_core::business_logic::services::EvaluationService;
use agent_core::business_logic::services::MemoryService;
use agent_core::business_logic::services::DiscoveryService;
use agent_core::business_logic::services::WorkflowServiceApi;
use workflow_management::graph::config::load_graph_from_file;

use workflow_management::graph::{ graph_orchestrator::PlanExecutor};


use agent_core::planning::plan_definition::ExecutionResult;
use llm_api::chat::Message as LlmMessage;
use agent_core::business_logic::agent::Agent;


/// Agent that executes predefined workflows.
#[allow(dead_code)]
#[derive(Clone)]
pub struct WorkFlowAgent {
    agent_config: Arc<AgentConfig>,
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
        let _llm_interaction = ChatLlmInteraction::new(
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
            workflow_runners,
            discovery_service,
        })
    }

    // todo:use metadata to potentially handle workflow execution request
    // add possibility to dynamically create a workflow based on user query.. or even step by step
    async fn handle_request(&self, request: LlmMessage) -> anyhow::Result<ExecutionResult> {
        let request_id = Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();
        let user_query = request.content.clone().unwrap_or_default();

        info!("---WorkflowAgent: Starting to handle user request -- Query: \'{}\'---", user_query);
        let graph_file="./workflow_management/example_workflow/multi_agent_workflow.json";
        
        // LOAD AND EXECUTE
        // 4. Load workflow and execute
        let workflow_file = graph_file;
        info!("Loading workflow from: {}", workflow_file);

        match load_graph_from_file(workflow_file) {
            Ok(graph) => {
                info!("Workflow loaded successfully. Plan: {}", graph.plan_name);
                
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
                            plan_details: None 
                        })
                    },
                    Err(e) => {
                        warn!("Error executing plan: {}", e);
                        Err(anyhow::anyhow!("Workflow execution failed: {}", e))
                    }
                }
            },
            Err(e) => {
                error!("Error loading workflow: {}", e);
                Err(anyhow::anyhow!("Failed to load workflow: {}", e))
            }
        }
    }
}
