use super::graph_definition::{Activity, ActivityType, Graph, NodeType, PlanContext, PlanState};
use crate::agent_communication::agent_registry::AgentRegistry;
use crate::tasks::condition_evaluator::evaluate_condition;
use crate::tasks::task_registry::TaskRegistry;
use crate::tools::tool_registry::ToolRegistry;
use regex::Regex;
use serde_json::Value;
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
}

pub struct PlanExecutor {
    context: PlanContext,
    task_registry: Arc<TaskRegistry>,
    agent_registry: Arc<AgentRegistry>,
    tool_registry: Arc<ToolRegistry>,
    execution_queue: VecDeque<String>,
    dependency_tracker: HashMap<String, usize>,
}

impl PlanExecutor {
    pub fn new(
        graph: Graph,
        task_registry: Arc<TaskRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            context: PlanContext::new(graph),
            task_registry,
            agent_registry,
            tool_registry,
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
        } else if self.context.results.len() == self.context.graph.nodes.len() {
            self.context.plan_state = PlanState::Completed;
        } else {
            self.context.plan_state = PlanState::Completed;
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
                    })?;
                let runner = self
                    .agent_registry
                    .get(agent_id)
                    .ok_or_else(|| PlanExecutorError::AgentRunnerNotFound(agent_id.clone()))?;
                runner
                    .invoke(&activity)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
            }
            ActivityType::DirectToolUse => {
                let tool_name = activity
                    .tool_to_use
                    .as_ref()
                    .ok_or_else(|| PlanExecutorError::MissingTool(activity.id.clone()))?;
                let runner = self
                    .tool_registry
                    .get(tool_name)
                    .ok_or_else(|| PlanExecutorError::ToolRunnerNotFound(tool_name.clone()))?;
                let params = activity.tool_parameters.as_ref().unwrap_or(&Value::Null);
                runner
                    .run(params)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
            }
            ActivityType::DirectTaskExecution => {
                let skill = activity
                    .skill_to_use
                    .as_ref()
                    .ok_or_else(|| PlanExecutorError::MissingSkill(activity.id.clone()))?;
                let runner = self
                    .task_registry
                    .get(skill)
                    .ok_or_else(|| PlanExecutorError::TaskRunnerNotFound(skill.clone()))?;
                let dependencies = self.get_activity_dependencies(&activity);
                runner
                    .execute(&activity, &dependencies)
                    .await
                    .map_err(|e| PlanExecutorError::ExecutionFailed(e.to_string()))?
            }
        };

        self.context.results.insert(node_id.clone(), result.clone());
        println!("Executed node '{}', result: '{}'", node_id, result);
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
        println!(
            "Plan executed successfully. Final results: {:?}",
            self.context.results
        );
        Ok(())
    }

    fn handle_failure_state(&self, reason: String) -> Result<(), PlanExecutorError> {
        eprintln!("Execution failed: {}", reason);
        Err(PlanExecutorError::ExecutionFailed(reason))
    }

    fn get_activity_dependencies(&self, activity: &Activity) -> HashMap<String, String> {
        activity
            .dependencies
            .iter()
            .filter_map(|dep| {
                self.context
                    .results
                    .get(&dep.source)
                    .map(|res| (dep.source.clone(), res.clone()))
            })
            .collect()
    }

    fn interpolate_parameters(
        &self,
        activity: &Activity,
    ) -> Result<Activity, PlanExecutorError> {
        let mut hydrated_activity = activity.clone();
        let tool_params = match &mut hydrated_activity.tool_parameters {
            Some(params) => params,
            None => return Ok(hydrated_activity),
        };

        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        let mut replacements = Vec::new();

        if let Value::Object(map) = tool_params {
            for (key, value) in map.iter() {
                if let Value::String(s) = value {
                    if let Some(cap) = re.captures(s) {
                        let path = &cap[1];
                        let parts: Vec<&str> = path.splitn(2, '.').collect();
                        if parts.is_empty() {
                            continue;
                        }

                        let source_id = parts[0];
                        let result_str =
                            self.context.results.get(source_id).ok_or_else(|| {
                                PlanExecutorError::InterpolationFailed(format!(
                                    "Dependency result for '{}' not found",
                                    source_id
                                ))
                            })?;

                        let value_to_insert = match serde_json::from_str(result_str) {
                            Ok(result_json) => {
                                if parts.len() > 1 {
                                    let path_keys = parts[1].split('.');
                                    let mut current_value: &Value = &result_json;
                                    for key in path_keys {
                                        current_value = current_value.get(key).unwrap_or(&Value::Null);
                                    }
                                    current_value.clone()
                                } else {
                                    result_json
                                }
                            },
                            Err(_) => {
                                // If not valid JSON, treat as plain text
                                Value::String(result_str.to_string())
                            }
                        };

                        replacements.push((key.clone(), value_to_insert));
                    }
                }
            }

            for (key, value) in replacements {
                map.insert(key, value);
            }
        }

        Ok(hydrated_activity)
    }
}
