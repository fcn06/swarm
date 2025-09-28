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

// New structs for the input JSON schema
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AgentConfigInput {
    pub skill_to_use: Option<String>,
    pub assigned_agent_id_preference: Option<String>,
    #[serde(default)]
    pub agent_context: Option<serde_json::Value>, // Added agent_context
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolConfigInput {
    pub tool_to_use: Option<String>,
    #[serde(default)]
    pub tool_parameters: serde_json::Value, // Changed to Value to allow any object
}

// New struct for Task configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskConfigInput {
    pub task_to_use: Option<String>,
    #[serde(default)]
    pub task_parameters: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ActivityInput {
    pub activity_type: ActivityType,
    pub id: String,
    pub description: String,
    pub r#type: String, // 'type' is required in the schema
    #[serde(default)]
    pub agent: Option<AgentConfigInput>, // Made optional
    #[serde(default)]
    pub tools: Option<Vec<ToolConfigInput>>, // Made optional
    #[serde(default)]
    pub tasks: Option<Vec<TaskConfigInput>>, // Replaced tasks_parameters and made optional
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    pub expected_outcome: String, // 'expected_outcome' is required in the schema
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WorkflowPlanInput {
    pub plan_name: String,
    pub activities: Vec<ActivityInput>,
}


// Existing Activity struct, adapted to flatten information from ActivityInput
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Activity {
    pub activity_type: ActivityType,
    pub id: String,
    pub description: String,
    pub r#type: Option<String>, // Renamed from 'type' to 'r#type' to avoid keyword conflict
    pub skill_to_use: Option<String>,
    pub assigned_agent_id_preference: Option<String>,
    #[serde(default)]
    pub agent_context: Option<serde_json::Value>, // Added agent_context here
    pub tool_to_use: Option<String>,
    pub tool_parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub tasks: Option<Vec<TaskConfigInput>>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    pub expected_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_output: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Dependency {
    pub source: String,
    pub condition: Option<String>, // Added condition to Dependency
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
    pub plan_name: String, // Changed from 'id' to 'plan_name'
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
pub struct PlanContextOld {
    pub plan_state: PlanState,
    pub graph: Graph,
    pub current_step_id: Option<String>,
    pub results: HashMap<String, String>, 
    pub user_query: String, // Add this field
}

impl PlanContextOld {
    pub fn new(graph: Graph, user_query: String) -> Self { // Modify signature
        Self {
            plan_state: PlanState::Idle,
            graph,
            current_step_id: None,
            results: HashMap::new(),
            user_query, // Initialize user_query
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlanContext {
    pub plan_state: PlanState,
    pub graph: Graph,
    pub current_step_id: Option<String>,
    pub activities_outcome: HashMap<String, String>, // to store the outcome of each activity processed during graph execution
    pub final_outcome: String,  // to store the very final result of the last node
    pub user_query: String, // Add this field
}

impl PlanContext {
    pub fn new(graph: Graph, user_query: String) -> Self { // Modify signature
        
        Self {
            plan_state: PlanState::Idle,
            graph,
            current_step_id: None,
            activities_outcome: HashMap::new(),
            final_outcome: String::new(),
            user_query, // Initialize user_query
        }
        
    }
}


// Conversion from WorkflowPlanInput to Graph
impl From<WorkflowPlanInput> for Graph {
    fn from(plan_input: WorkflowPlanInput) -> Self {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        for activity_input in plan_input.activities {
            // Flatten the agent and tools configuration into the Activity struct
            let activity = Activity {
                activity_type: activity_input.activity_type,
                id: activity_input.id.clone(),
                description: activity_input.description,
                r#type: Some(activity_input.r#type),
                skill_to_use: activity_input.agent.as_ref().and_then(|a| a.skill_to_use.clone()),
                assigned_agent_id_preference: activity_input.agent.as_ref().and_then(|a| a.assigned_agent_id_preference.clone()),
                agent_context: activity_input.agent.as_ref().and_then(|a| a.agent_context.clone()), // Mapped agent_context
                tool_to_use: activity_input.tools.as_ref().and_then(|t_vec| t_vec.get(0)).and_then(|t| t.tool_to_use.clone()),
                tool_parameters: activity_input.tools.as_ref().and_then(|t_vec| t_vec.get(0)).map(|t| t.tool_parameters.clone()),
                tasks: activity_input.tasks.clone(),
                dependencies: activity_input.dependencies.clone(),
                expected_outcome: Some(activity_input.expected_outcome),
                activity_output: None, // This will be populated during execution
            };

            // Add dependencies as edges
            for dep in activity_input.dependencies {
                edges.push(Edge {
                    source: dep.source,
                    target: activity.id.clone(),
                    condition: dep.condition, // Include condition from dependency
                });
            }

            nodes.insert(
                activity.id.clone(),
                Node {
                    id: activity.id.clone(),
                    node_type: NodeType::Activity(activity),
                },
            );
        }

        Graph {
            plan_name: plan_input.plan_name,
            nodes,
            edges,
        }
    }
}
