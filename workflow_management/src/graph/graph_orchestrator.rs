use super::graph_definition::{Graph, PlanContext, PlanState};
use crate::agent_communication::agent_registry::AgentRegistry;
use crate::tasks::condition_evaluator::evaluate_condition;
use crate::tasks::task_registry::TaskRegistry;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PlanExecutorError {
    #[error("Missing node in graph: {0}")]
    MissingNode(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid state transition")]
    InvalidState,
    #[error("Task runner not found for skill: {0}")]
    TaskRunnerNotFound(String),
    #[error("Agent runner not found: {0}")]
    AgentRunnerNotFound(String),
    #[error("Default agent execution failed: {0}")]
    DefaultAgentExecutionFailed(String),
    #[error("Cyclic dependency detected")]
    CyclicDependency,
}

pub struct PlanExecutor {
    context: PlanContext,
    task_registry: Arc<TaskRegistry>,
    agent_registry: Arc<AgentRegistry>,
    execution_queue: VecDeque<String>,
    dependency_tracker: HashMap<String, usize>,
}

impl PlanExecutor {
    pub fn new(
        graph: Graph,
        task_registry: Arc<TaskRegistry>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            context: PlanContext::new(graph),
            task_registry,
            agent_registry,
            execution_queue: VecDeque::new(),
            dependency_tracker: HashMap::new(),
        }
    }

    pub async fn execute_plan(&mut self) -> Result<(), PlanExecutorError> {
        self.context.plan_state = PlanState::Idle;
        loop {
            match self.context.plan_state.clone() {
                PlanState::Idle => self.handle_idle_state()?,
                PlanState::Initializing => self.handle_initializing_state()?,
                PlanState::DecidingNextStep => self.handle_deciding_next_step_state()?,
                PlanState::ExecutingStep => self.handle_executing_step_state().await?,
                PlanState::Completed => {
                    self.handle_completion_state()?;
                    break;
                }
                PlanState::Failed(ref reason) => {
                    self.handle_failure_state(reason.clone())?;
                    break;
                }
                _ => return Err(PlanExecutorError::InvalidState),
            }
        }
        Ok(())
    }

    fn handle_idle_state(&mut self) -> Result<(), PlanExecutorError> {
        self.context.plan_state = PlanState::Initializing;
        Ok(())
    }

    fn handle_initializing_state(&mut self) -> Result<(), PlanExecutorError> {
        for (node_id, node) in &self.context.graph.nodes {
            let dep_count = self.context.graph.edges.iter().filter(|e| e.target == *node_id).count();
            self.dependency_tracker.insert(node_id.clone(), dep_count);
        }

        for (node_id, count) in &self.dependency_tracker {
            if *count == 0 {
                self.execution_queue.push_back(node_id.clone());
            }
        }

        if self.execution_queue.is_empty() && !self.context.graph.nodes.is_empty() {
            return Err(PlanExecutorError::CyclicDependency);
        }

        self.context.plan_state = PlanState::DecidingNextStep;
        Ok(())
    }

    fn handle_deciding_next_step_state(&mut self) -> Result<(), PlanExecutorError> {
        if let Some(node_id) = self.execution_queue.pop_front() {
            self.context.current_step_id = Some(node_id);
            self.context.plan_state = PlanState::ExecutingStep;
        } else if self.context.results.len() == self.context.graph.nodes.len() {
            self.context.plan_state = PlanState::Completed;
        } else {
            // If the queue is empty but not all nodes are done, it could be a dead-end due to conditions
            self.context.plan_state = PlanState::Completed;
        }
        Ok(())
    }

    async fn handle_executing_step_state(&mut self) -> Result<(), PlanExecutorError> {
        let node_id = self.context.current_step_id.as_ref().cloned().ok_or(PlanExecutorError::InvalidState)?;
        let node = self.context.graph.nodes.get(&node_id).cloned().ok_or_else(|| PlanExecutorError::MissingNode(node_id.clone()))?;
    
        let result = match &node.node_type {
            super::graph_definition::NodeType::Task(task) => {
                let skill = task.skill_to_use.as_ref().ok_or_else(|| PlanExecutorError::TaskRunnerNotFound("No skill specified".to_string()))?;
                
                if let Some(runner) = self.task_registry.get(skill) {
                    let dependencies = self.get_task_dependencies(task);
                    runner.execute(task, &dependencies).await.map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
                } else {
                    // Default agent fallback
                    let runner = self.agent_registry.get("a2a_http_runner").ok_or_else(|| PlanExecutorError::AgentRunnerNotFound("a2a_http_runner".to_string()))?;
                    runner.invoke(task).await.map_err(|e| PlanExecutorError::DefaultAgentExecutionFailed(e.to_string()))?
                }
            }
            super::graph_definition::NodeType::Agent(agent_node) => {
                // For now, we assume agent nodes are also defined by a task. This can be evolved.
                let task_def = self.get_task_definition_for_agent(&node_id)?;
                let runner = self.agent_registry.get("a2a_http_runner").ok_or_else(|| PlanExecutorError::AgentRunnerNotFound("a2a_http_runner".to_string()))?;
                runner.invoke(task_def).await.map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
            }
        };
    
        self.context.results.insert(node_id.clone(), result.clone());
        println!("Executed node '{}', result: '{}'", node_id, result);
        self.update_downstream_dependencies(&node_id, &result)?;
        self.context.plan_state = PlanState::DecidingNextStep;
        Ok(())
    }

    fn update_downstream_dependencies(&mut self, completed_node_id: &str, result: &str) -> Result<(), PlanExecutorError> {
        for edge in &self.context.graph.edges {
            if edge.source == *completed_node_id {
                let mut condition_met = true;
                if let Some(condition) = &edge.condition {
                    let mut dependencies = HashMap::new();
                    dependencies.insert(completed_node_id.to_string(), result.to_string());
                    condition_met = evaluate_condition(condition, &dependencies);
                }

                if condition_met {
                    if let Some(count) = self.dependency_tracker.get_mut(&edge.target) {
                        *count -= 1;
                        if *count == 0 {
                            self.execution_queue.push_back(edge.target.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_completion_state(&mut self) -> Result<(), PlanExecutorError> {
        println!("Plan executed successfully. Final results: {:?}", self.context.results);
        Ok(())
    }

    fn handle_failure_state(&self, reason: String) -> Result<(), PlanExecutorError> {
        eprintln!("Execution failed: {}", reason);
        Err(PlanExecutorError::ExecutionFailed(reason))
    }

    fn get_task_dependencies(&self, task: &agent_core::planning::plan_definition::TaskDefinition) -> HashMap<String, String> {
        task.dependencies.iter().filter_map(|dep_id| 
            self.context.results.get(dep_id).map(|res| (dep_id.clone(), res.clone()))
        ).collect()
    }

    fn get_task_definition_for_agent(&self, agent_node_id: &str) -> Result<&agent_core::planning::plan_definition::TaskDefinition, PlanExecutorError> {
        // This is a placeholder. In a real scenario, an agent node might have its own
        // associated task, or we might need to find the task that assigned this agent.
        // For now, we'll assume the agent node IS a task node.
        if let Some(node) = self.context.graph.nodes.get(agent_node_id) {
            if let super::graph_definition::NodeType::Task(task_def) = &node.node_type {
                return Ok(task_def);
            }
        }
        Err(PlanExecutorError::InvalidState)
    }
}
