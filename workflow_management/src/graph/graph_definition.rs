use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    Idle,
    Busy,
    Unavailable,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ActivityType {
    #[serde(rename = "delegation_agent")]
    DelegationAgent,
    #[serde(rename = "direct_tool_use")]
    DirectToolUse,
    #[serde(rename = "direct_task_execution")]
    DirectTaskExecution,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Activity {
    pub activity_type: ActivityType,
    pub id: String,
    pub description: String,
    pub r#type: Option<String>, // Renamed from 'type' to 'r#type' to avoid keyword conflict
    pub skill_to_use: Option<String>,
    pub assigned_agent_id_preference: Option<String>,
    pub tool_to_use: Option<String>,
    pub tool_parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    pub expected_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_output: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Dependency {
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    Activity(Activity),
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
    pub condition: Option<String>,
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
