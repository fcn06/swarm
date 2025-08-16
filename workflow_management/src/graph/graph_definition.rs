use agent_protocol_backbone::planning::plan_definition::TaskDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    Idle,
    Busy,
    Unavailable,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub available_tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    Task(TaskDefinition),
    Agent(Agent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Graph {
    pub id: String,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlanState {
    Idle,
    Initializing,
    ExecutingStep,
    AwaitingAgentResponse,
    ProcessingAgentResponse,
    DecidingNextStep,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct PlanContext {
    pub plan_state: PlanState,
    pub graph: Graph,
    pub current_step_id: Option<String>,
    pub results: HashMap<String, String>,
}

impl PlanContext {
    pub fn new(graph: Graph) -> Self {
        Self {
            plan_state: PlanState::Idle,
            graph,
            current_step_id: None,
            results: HashMap::new(),
        }
    }
}
