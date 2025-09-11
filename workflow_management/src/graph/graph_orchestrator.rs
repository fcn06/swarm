use agent_core::graph::graph_definition::{Activity, ActivityType, Graph, NodeType, PlanContext, PlanState};

use crate::agent_communication::agent_runner::AgentRunner;
use crate::tasks::condition_evaluator::evaluate_condition;
use crate::tasks::task_runner::TaskRunner; // Changed from task_registry
use crate::tools::tool_runner::ToolRunner; // Changed from tool_registry
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug,info};

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
    #[error("Tool runner not found: {0}")]
    ToolRunnerNotFound(String),
    #[error("Cyclic dependency detected")]
    CyclicDependency,
    #[error("Missing tool to use for DirectToolUse activity: {0}")]
    MissingTool(String),
    #[error("Missing skill to use for DirectTaskExecution activity: {0}")]
    MissingSkill(String),
    #[error("Parameter interpolation failed: {0}")]
    InterpolationFailed(String),
    #[error("Missing task to use for DirectTaskExecution activity: {0}")]
    MissingTask(String),
}

pub struct PlanExecutor {
    context: PlanContext,
    task_runner: Arc<TaskRunner>, // Changed from task_registry
    agent_runner: Arc<AgentRunner>,
    tool_runner: Arc<ToolRunner>, // Changed from tool_registry
    execution_queue: VecDeque<String>,
    dependency_tracker: HashMap<String, usize>,
}

impl PlanExecutor {
    pub fn new(
        graph: Graph,
        task_runner: Arc<TaskRunner>, // Changed from task_registry
        agent_runner: Arc<AgentRunner>,
        tool_runner: Arc<ToolRunner>, // Changed from tool_registry
        user_query: String,
    ) -> Self {
        Self {
            context: PlanContext::new(graph, user_query),
            task_runner,
            agent_runner,
            tool_runner,
            execution_queue: VecDeque::new(),
            dependency_tracker: HashMap::new(),
        }
    }

    pub async fn execute_plan(&mut self) -> Result<HashMap<String, String>, PlanExecutorError> {
        self.context.plan_state = PlanState::Idle;
        loop {
            match self.context.plan_state.clone() {
                PlanState::Idle => self.handle_idle_state()?,
                PlanState::Initializing => self.handle_initializing_state()?,
                PlanState::DecidingNextStep => self.handle_deciding_next_step_state()?,
                PlanState::ExecutingStep => self.handle_executing_step_state().await?,
                PlanState::Completed => {
                    return self.handle_completion_state();
                }
                PlanState::Failed(ref reason) => {
                    return self.handle_failure_state(reason.clone());
                }
                _ => return Err(PlanExecutorError::InvalidState),
            }
        }
    }

    fn handle_idle_state(&mut self) -> Result<(), PlanExecutorError> {
        self.context.plan_state = PlanState::Initializing;
        Ok(())
    }

    fn handle_initializing_state(&mut self) -> Result<(), PlanExecutorError> {
        for (node_id, _node) in &self.context.graph.nodes {
            let dep_count = self
                .context
                .graph
                .edges
                .iter()
                .filter(|e| e.target == *node_id)
                .count();
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
        } else {
            if self.context.results.len() == self.context.graph.nodes.len() {
                self.context.plan_state = PlanState::Completed;
            } else {
                self.context.plan_state = PlanState::Failed("No executable tasks left and plan not completed.".to_string());
            }
        }
        Ok(())
    }

    async fn handle_executing_step_state(&mut self) -> Result<(), PlanExecutorError> {
        let node_id = self
            .context
            .current_step_id
            .as_ref()
            .cloned()
            .ok_or(PlanExecutorError::InvalidState)?;
        let node = self
            .context
            .graph
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or_else(|| PlanExecutorError::MissingNode(node_id.clone()))?;

        let NodeType::Activity(original_activity) = &node.node_type;

        let activity = self.interpolate_parameters(original_activity)?;
        let result = match activity.activity_type {
            ActivityType::DelegationAgent => {
                let agent_id = activity
                    .assigned_agent_id_preference
                    .as_ref()
                    .ok_or_else(|| {
                        PlanExecutorError::AgentRunnerNotFound(
                            "No agent preference specified".to_string(),
                        )
                    })?
                    .clone();

                let mut message = String::new();
                message.push_str(&format!("Here is the user_query :"));
                message.push_str(&activity.description.clone());
                if let Some(context) = &activity.agent_context {
                    message.push_str(&format!("\nHere are contextual information to take into account when processing user_query: {}\n", context.to_string()));
                }

                debug!("Executing activity '{}', message: '{}' \n", &activity.id, message);

                let skill = activity.skill_to_use.clone().unwrap_or_default();

                self.agent_runner
                    .interact(agent_id, message, skill)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
                    .to_string()
            }
            ActivityType::DirectToolUse => {
                let tool_id = activity
                    .tool_to_use
                    .as_ref()
                    .ok_or_else(|| PlanExecutorError::MissingTool(activity.id.clone()))?
                    .clone();
                let params = activity.tool_parameters.unwrap_or_else(|| Value::Null);

                self.tool_runner
                    .run(tool_id, params)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
                    .to_string()
            }
            ActivityType::DirectTaskExecution => {
                let tasks = activity.tasks.as_ref().ok_or_else(|| PlanExecutorError::MissingTask(activity.id.clone()))?;
                let task_config = tasks.get(0).ok_or_else(|| PlanExecutorError::MissingTask(activity.id.clone()))?;

                let task_id = task_config.task_to_use.as_ref().cloned().unwrap_or_default();
                let params = task_config.task_parameters.clone();

                self.task_runner
                    .run(task_id, params)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
                    .to_string()
            }
        };

        self.context.results.insert(node_id.clone(), result.clone());

        // Attempt to parse the result as JSON for pretty printing
        let printable_result = match serde_json::from_str::<serde_json::Value>(&result) {
            Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| result.clone()),
            Err(_) => result.clone(),
        };

        info!("Executed node '{}', result: '{}' \n", node_id, printable_result);
        self.update_downstream_dependencies(&node_id, &result)?;
        self.context.plan_state = PlanState::DecidingNextStep;

        Ok(())
    }

    fn update_downstream_dependencies(
        &mut self,
        completed_node_id: &str,
        result: &str,
    ) -> Result<(), PlanExecutorError> {
        for edge in &self.context.graph.edges {
            if edge.source == *completed_node_id {
                let mut condition_met = true;
                if let Some(condition) = &edge.condition {
                    let mut dependencies = HashMap::new();
                    let result_value = serde_json::from_str(result)
                        .unwrap_or_else(|_| Value::String(result.to_string()));
                    dependencies.insert(completed_node_id.to_string(), result_value);
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

    fn handle_completion_state(&mut self) -> Result<HashMap<String, String>, PlanExecutorError> {
        debug!("\nPlan executed successfully. Final results:");
        for (node_id, result) in &self.context.results {
            let printable_result = match serde_json::from_str::<serde_json::Value>(result) {
                Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| result.clone()),
                Err(_) => result.clone(),
            };
            debug!("  '{}': {}", node_id, printable_result);
        }
        Ok(self.context.results.clone())
    }

    fn handle_failure_state(&self, reason: String) -> Result<HashMap<String, String>, PlanExecutorError> {
        debug!("Execution failed: {}", reason);
        Err(PlanExecutorError::ExecutionFailed(reason))
    }


    fn interpolate_parameters(
        &self,
        activity: &Activity,
    ) -> Result<Activity, PlanExecutorError> {
        let mut hydrated_activity = activity.clone();
        // Regex to find placeholders like {{activity_id.activity_output}}
        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    
        // Simplified closure to get the replacement value.
        let get_interpolated_value = |path: &str| -> Result<String, PlanExecutorError> {
            // Extract the source activity ID, ignoring ".activity_output"
            let source_id = path.split('.').next().unwrap_or("");
            
            if source_id.is_empty() {
                return Err(PlanExecutorError::InterpolationFailed(
                    "Invalid interpolation path: empty source ID".to_string(),
                ));
            }
    
            // Retrieve the entire result string for the given source_id.
            self.context.results.get(source_id)
                .cloned()
                .ok_or_else(|| {
                    PlanExecutorError::InterpolationFailed(format!(
                        "Dependency result for '{}' not found for activity '{}'",
                        source_id, activity.id
                    ))
                })
        };
    
        // Generic function to replace placeholders in any serde_json::Value
        let interpolator = |json_value: &mut Value| {
            if let Value::Object(map) = json_value {
                for (_, value) in map.iter_mut() {
                    if let Value::String(s) = value {
                        if s.contains("{{") {
                            if let Some(caps) = re.captures(s) {
                                if let Some(path) = caps.get(1) {
                                    match get_interpolated_value(path.as_str()) {
                                        Ok(interpolated_val) => {
                                            // The result might be a JSON object string, a number, or a simple string.
                                            // Attempt to parse it as a generic JSON Value.
                                            // If it fails, treat it as a plain string.
                                            *value = serde_json::from_str(&interpolated_val).unwrap_or(Value::String(interpolated_val));
                                        },
                                        Err(e) => {
                                            // If interpolation fails, replace with an error message.
                                            *value = Value::String(format!("ERROR: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    
        // Interpolate tool_parameters
        if let Some(tool_params) = &mut hydrated_activity.tool_parameters {
            interpolator(tool_params);
        }
    
        // Interpolate tasks parameters
        if let Some(tasks) = &mut hydrated_activity.tasks {
            for task_config in tasks.iter_mut() {
                interpolator(&mut task_config.task_parameters);
            }
        }
    
        // Interpolate agent_context parameters
        if let Some(agent_context) = &mut hydrated_activity.agent_context {
            interpolator(agent_context);
        }
    
        debug!("Hydrated Activity: {:?}", hydrated_activity);
        
        Ok(hydrated_activity)
    }
    

}
