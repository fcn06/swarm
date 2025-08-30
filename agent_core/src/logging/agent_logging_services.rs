use agent_evaluation_service::evaluation_server::judge_agent::AgentLogData;
use agent_memory_service::models::Role;
use crate::business_logic::services::{EvaluationService, MemoryService};

use crate::execution::execution_result::{ExecutionResult};
use tracing::warn;
use anyhow::Result;
use std::sync::Arc;

/// Agent that that can interact with other available agents, and also embed MCP runtime if needed
#[derive(Clone)]
pub struct AgentLogging {
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    memory_service: Option<Arc<dyn MemoryService>>,
}

#[allow(dead_code)]
impl AgentLogging  {

    pub fn new(evaluation_service: Option<Arc<dyn EvaluationService>>, memory_service: Option<Arc<dyn MemoryService>>) -> Self {
        Self {
            evaluation_service,
            memory_service,
        }
    }

    // New helper function for asynchronous evaluation logging
    async fn log_evaluation_data(&self,agent_name: &str, request_id: &str, user_query: &str, execution_result: &Result<ExecutionResult>) {
        if let Some(service) = self.evaluation_service.clone() {
            let user_query_clone = user_query.to_string();
            let request_id_clone = request_id.to_string();
            let agent_name=agent_name.to_string();

            // Extract and clone the output string before spawning the task
            let agent_output = match execution_result {
                Ok(result) => result.output.clone(),
                Err(e) => format!("Error during execution: {}", e),
            };

            tokio::spawn(async move {
                let log_data = AgentLogData {
                    agent_id: agent_name.to_string(),
                    request_id: request_id_clone,
                    step_id: "".to_string(),
                    original_user_query: user_query_clone.clone(),
                    agent_input: user_query_clone,
                    agent_output, // agent_output is now an owned String
                    context_snapshot: None,
                    success_criteria: None,
                };

                if let Err(e) = service.log_evaluation(log_data).await {
                    warn!("Failed to log evaluation: {}", e);
                }
            });
        }
    }

    async fn log_memory_data(&self,agent_name: &str, conversation_id: &str, user_query: &str, execution_result: &Result<ExecutionResult>) {
        if let Some(service) = self.memory_service.clone() {
           
            let user_query_clone = user_query.to_string();
            let conversation_id_clone = conversation_id.to_string();
            let agent_name = agent_name.to_string();

            // Extract and clone the output string before spawning the task
            let agent_response = match execution_result {
                Ok(result) => result.output.clone(),
                Err(e) => format!("Error during execution: {}", e),
            };

            tokio::spawn(async move {

                if let Err(e) = service.log(conversation_id_clone.clone(), Role::User, user_query_clone, None).await {
                    warn!("Failed to log user query to memory: {}", e);
                }


                if let Err(e) = service.log(conversation_id_clone.clone(), Role::Agent, agent_response, Some(agent_name)).await {
                    warn!("Failed to log agent response to memory: {}", e);
                }
            });
        }
    }

}