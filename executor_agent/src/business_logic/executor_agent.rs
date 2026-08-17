use std::sync::Arc;
use std::any::Any;
use async_trait::async_trait;
use tracing::{debug, warn};

use serde_json::{Value, json};
use uuid::Uuid;

use configuration::AgentConfig;
use agent_core::business_logic::mcp_runtime::McpRuntimeDetails;
use agent_core::business_logic::agent::Agent;
use agent_core::business_logic::services::{DiscoveryService, MemoryService, EvaluationService, WorkflowServiceApi};
use agent_models::graph::graph_definition::Graph;
use agent_models::execution::execution_result::ExecutionResult;
use workflow_management::graph::graph_orchestrator::PlanExecutor;
use workflow_management::agent_communication::agent_invoker::AgentInvoker;
use workflow_management::tasks::task_invoker::TaskInvoker;
use workflow_management::tools::tool_invoker::ToolInvoker;
use resource_invoker::A2AAgentInvoker;
use agent_models::agent_request::AgentRequest;

// TODO: Move this to a separate file if it grows
#[derive(Clone)]
pub struct WorkFlowInvokers {
    pub task_invoker: Arc<dyn TaskInvoker>,
    pub agent_invoker: Arc<dyn AgentInvoker>,
    pub tool_invoker: Arc<dyn ToolInvoker>,
}

impl WorkFlowInvokers {
    pub async fn init(
        task_invoker: Arc<dyn TaskInvoker>,
        agent_invoker: Arc<dyn AgentInvoker>,
        tool_invoker: Arc<dyn ToolInvoker>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            task_invoker,
            agent_invoker,
            tool_invoker,
        })
    }
}

#[async_trait]
impl WorkflowServiceApi for WorkFlowInvokers {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    async fn refresh_agents(&self) -> anyhow::Result<()> {
        // Attempt to downcast the agent_invoker to A2AAgentInvoker to call refresh_agents
        if let Some(a2a_invoker) = self.agent_invoker.as_any().downcast_ref::<A2AAgentInvoker>() {
            a2a_invoker.refresh_agents().await?;
        } else {
            warn!("WorkFlowInvokers::refresh_agents: agent_invoker is not an A2AAgentInvoker. Skipping agent refresh.");
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ExecutorAgent {
    agent_config: Arc<AgentConfig>,
    workflow_invokers: Arc<WorkFlowInvokers>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
}

#[async_trait]
impl Agent for ExecutorAgent {
    async fn new(
        agent_config: AgentConfig,
        _agent_api_key: String,
        _mcp_runtime_details: Option<McpRuntimeDetails>,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        _memory_service: Option<Arc<dyn MemoryService>>,
        _discovery_service: Option<Arc<dyn DiscoveryService>>,
        workflow_service: Option<Arc<dyn WorkflowServiceApi>>,
    ) -> anyhow::Result<Self> {
        let workflow_invokers_arc = workflow_service
            .and_then(|ws_api_arc| {
                // Downcast Arc<dyn WorkflowServiceApi> to Arc<WorkFlowInvokers>
                ws_api_arc.as_ref().as_any().downcast_ref::<WorkFlowInvokers>().map(|wr| Arc::new(wr.clone()))
            })
            .ok_or_else(|| anyhow::anyhow!("WorkFlowInvokers not provided or invalid type"))?;

        Ok(Self {
            agent_config: Arc::new(agent_config),
            workflow_invokers: workflow_invokers_arc,
            evaluation_service,
        })
    }

    async fn handle_request(
        &self,
        request: AgentRequest,
    ) -> anyhow::Result<ExecutionResult> {
        let plan_json = request.user_query();
        let graph: Graph = serde_json::from_str(&plan_json)?;

        let original_user_query = request.metadata
            .as_ref()
            .and_then(|m| m.get("original_user_query"))
            .and_then(|v| v.as_str())
            .unwrap_or("User query not available in executor")
            .to_string();

        debug!("---ExecutorAgent: Starting to execute plan---");
        debug!("Graph Received: {:#?}", graph);

        let mut executor = PlanExecutor::new(
            graph,
            self.workflow_invokers.task_invoker.clone(),
            self.workflow_invokers.agent_invoker.clone(),
            self.workflow_invokers.tool_invoker.clone(),
            original_user_query,
        );

        match executor.execute_plan().await {
            Ok((execution_outcome, _activities_outcome)) => {
                debug!("\nWorkflow execution completed successfully. Outcome : {}\n", execution_outcome);

                let parsed_outcome: Value = match serde_json::from_str(&execution_outcome) {
                    Ok(v) => v,
                    Err(_) => Value::String(execution_outcome),
                };

                Ok(ExecutionResult {
                    request_id: Uuid::new_v4().to_string(), // Generate a new UUID
                    conversation_id: Uuid::new_v4().to_string(), // Generate a new UUID
                    success: true,
                    output: json!({ "text_response": parsed_outcome }),
                })
            },
            Err(e) => {
                warn!("Error executing plan: {}", e);
                let error_message = format!("Workflow execution failed: {}", e);
                Ok(ExecutionResult {
                    request_id: Uuid::new_v4().to_string(),
                    conversation_id: Uuid::new_v4().to_string(),
                    success: false,
                    output: json!({ "error": error_message }),
                })
            }
        }
    }
}
