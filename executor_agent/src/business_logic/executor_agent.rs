
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, warn};
use serde_json::Map;
use serde_json::Value;
use std::collections::HashMap;

use llm_api::chat::Message as LlmMessage;

use configuration::AgentConfig;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService, WorkflowServiceApi};
use agent_core::graph::graph_definition::Graph;
use agent_core::execution::execution_result::ExecutionResult;
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::agent_communication::agent_runner::AgentRunner;
use workflow_management::tasks::task_runner::TaskRunner;
use workflow_management::tools::tool_runner::ToolRunner;
use std::any::Any;

// TODO: Move this to a separate file if it grows
#[derive(Clone)]
pub struct WorkFlowRunners {
    pub task_runner: Arc<TaskRunner>,
    pub agent_runner: Arc<AgentRunner>,
    pub tool_runner: Arc<ToolRunner>,
}

impl WorkFlowRunners {
    pub async fn init(
        task_runner: Arc<TaskRunner>,
        agent_runner: Arc<AgentRunner>,
        tool_runner: Arc<ToolRunner>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            task_runner,
            agent_runner,
            tool_runner,
        })
    }
}

impl WorkflowServiceApi for WorkFlowRunners {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}


#[derive(Clone)]
pub struct ExecutorAgent {
    agent_config: Arc<AgentConfig>,
    workflow_runners: Arc<WorkFlowRunners>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
}

#[async_trait]
impl Agent for ExecutorAgent {
    async fn new(
        agent_config: AgentConfig,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let workflow_runners = workflow_service
            .and_then(|ws| ws.as_any().downcast_ref::<WorkFlowRunners>().map(|wr| Arc::new(wr.clone())))
            .ok_or_else(|| anyhow::anyhow!("WorkFlowRunners not provided or invalid type"))?;

        Ok(Self {
            agent_config: Arc::new(agent_config),
            workflow_runners,
            evaluation_service,
        })
    }

    async fn handle_request(&self, request: LlmMessage, _metadata: Option<Map<String, Value>>) -> anyhow::Result<ExecutionResult> {
        let plan_json = request.content.unwrap_or_default();
        let graph: Graph = serde_json::from_str(&plan_json)?;

        debug!("---ExecutorAgent: Starting to execute plan---");
        debug!("Graph Received: {:#?}", graph);

        let mut executor = PlanExecutor::new(
            graph,
            self.workflow_runners.task_runner.clone(),
            self.workflow_runners.agent_runner.clone(),
            self.workflow_runners.tool_runner.clone(),
            "User query not available in executor".to_string(), // TODO: Pass user query through
        );

        match executor.execute_plan().await {
            Ok((execution_outcome, _activities_outcome)) => {
                debug!("\nWorkflow execution completed successfully. Outcome : {}\n", execution_outcome);
                Ok(ExecutionResult {
                    request_id: "".to_string(),
                    conversation_id: "".to_string(),
                    success: true,
                    output: execution_outcome,
                })
            },
            Err(e) => {
                warn!("Error executing plan: {}", e);
                let error_message = format!("Workflow execution failed: {}", e);
                Err(anyhow::anyhow!(error_message))
            }
        }
    }
}
