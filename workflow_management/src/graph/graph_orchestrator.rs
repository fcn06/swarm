use super::graph_definition::{Graph, PlanContext, PlanState};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PlanExecutorError {
    #[error("Missing node in graph: {0}")]
    MissingNode(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid state transition")]
    InvalidState,
}

pub struct PlanExecutor {
    context: PlanContext,
    sorted_nodes: VecDeque<String>,
}

impl PlanExecutor {
    pub fn new(graph: Graph) -> Self {
        Self {
            context: PlanContext::new(graph),
            sorted_nodes: VecDeque::new(),
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
                PlanState::AwaitingAgentResponse => self.handle_awaiting_response_state().await?,
                PlanState::ProcessingAgentResponse => self.handle_processing_response_state()?,
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
        println!("State: Idle -> Initializing");
        self.context.plan_state = PlanState::Initializing;
        Ok(())
    }

    fn handle_initializing_state(&mut self) -> Result<(), PlanExecutorError> {
        println!("State: Initializing");
        match self.topological_sort() {
            Ok(sorted_nodes) => {
                self.sorted_nodes = sorted_nodes.into();
                println!("State: Initializing -> DecidingNextStep");
                self.context.plan_state = PlanState::DecidingNextStep;
                Ok(())
            }
            Err(e) => {
                println!("State: Initializing -> Failed");
                self.context.plan_state = PlanState::Failed(e.to_string());
                Ok(())
            }
        }
    }

    fn handle_deciding_next_step_state(&mut self) -> Result<(), PlanExecutorError> {
        if let Some(node_id) = self.sorted_nodes.pop_front() {
            self.context.current_step_id = Some(node_id);
            println!("State: DecidingNextStep -> ExecutingStep ({})", self.context.current_step_id.as_ref().unwrap());
            self.context.plan_state = PlanState::ExecutingStep;
        } else {
            println!("State: DecidingNextStep -> Completed");
            self.context.plan_state = PlanState::Completed;
        }
        Ok(())
    }

    async fn handle_executing_step_state(&mut self) -> Result<(), PlanExecutorError> {
        let node_id = self.context.current_step_id.as_ref().cloned().ok_or_else(|| PlanExecutorError::InvalidState)?;
        let node = self.context.graph.nodes.get(&node_id).cloned().ok_or_else(|| PlanExecutorError::MissingNode(node_id.clone()))?;

        match &node.node_type {
            super::graph_definition::NodeType::Task(task) => {
                println!("Executing task: {}", task.description);
                // Simulate task execution
                let result = "Task completed successfully".to_string();
                self.context.results.insert(node.id, result);
                println!("State: ExecutingStep -> ProcessingAgentResponse");
                self.context.plan_state = PlanState::ProcessingAgentResponse;
            }
            super::graph_definition::NodeType::Agent(agent) => {
                println!("Executing agent: {}", agent.name);
                println!("State: ExecutingStep -> AwaitingAgentResponse");
                self.context.plan_state = PlanState::AwaitingAgentResponse;
            }
        }
        Ok(())
    }

    async fn handle_awaiting_response_state(&mut self) -> Result<(), PlanExecutorError> {
        // In a real scenario, this would involve asynchronous waiting for an external agent's response.
        // Here, we simulate receiving a response immediately.
        println!("State: AwaitingAgentResponse -> ProcessingAgentResponse");
        let response = "Agent responded successfully".to_string();
        let node_id = self.context.current_step_id.as_ref().cloned().ok_or_else(|| PlanExecutorError::InvalidState)?;
        self.context.results.insert(node_id, response);
        self.context.plan_state = PlanState::ProcessingAgentResponse;
        Ok(())
    }

    fn handle_processing_response_state(&mut self) -> Result<(), PlanExecutorError> {
        let node_id = self.context.current_step_id.as_ref().ok_or(PlanExecutorError::InvalidState)?;
        let result = self.context.results.get(node_id).ok_or(PlanExecutorError::InvalidState)?;
        println!("Processed response for {}: {}", node_id, result);
        println!("State: ProcessingAgentResponse -> DecidingNextStep");
        self.context.plan_state = PlanState::DecidingNextStep;
        Ok(())
    }

    fn handle_completion_state(&self) -> Result<(), PlanExecutorError> {
        println!("State: Completed");
        println!("Plan executed successfully. Final results: {:?}", self.context.results);
        Ok(())
    }

    fn handle_failure_state(&self, reason: String) -> Result<(), PlanExecutorError> {
        eprintln!("State: Failed");
        Err(PlanExecutorError::ExecutionFailed(reason))
    }

    fn topological_sort(&self) -> Result<Vec<String>, PlanExecutorError> {
        let mut in_degree: HashMap<String, usize> = self.context.graph.nodes.keys().map(|id| (id.clone(), 0)).collect();
        let mut adj: HashMap<String, Vec<String>> = self.context.graph.nodes.keys().map(|id| (id.clone(), vec![])).collect();

        for edge in &self.context.graph.edges {
            if let Some(degree) = in_degree.get_mut(&edge.target) {
                *degree += 1;
            }
            if let Some(neighbors) = adj.get_mut(&edge.source) {
                neighbors.push(edge.target.clone());
            }
        }

        let mut queue: VecDeque<String> = in_degree.iter().filter(|(_, &degree)| degree == 0).map(|(id, _)| id.clone()).collect();
        let mut sorted_nodes = Vec::new();

        while let Some(u) = queue.pop_front() {
            sorted_nodes.push(u.clone());
            if let Some(neighbors) = adj.get(&u) {
                for v in neighbors {
                    if let Some(degree) = in_degree.get_mut(v) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v.clone());
                        }
                    }
                }
            }
        }

        if sorted_nodes.len() != self.context.graph.nodes.len() {
            return Err(PlanExecutorError::ExecutionFailed("Circular dependency detected".to_string()));
        }

        Ok(sorted_nodes)
    }
}
