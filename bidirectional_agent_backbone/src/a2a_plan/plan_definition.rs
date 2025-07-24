use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanResponse {
    pub plan_summary: String,
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: String,
    pub description: String,
    pub skill_to_use: Option<String>,
    pub tool_to_use: Option<String>,
    pub assigned_agent_id_preference: Option<String>,
    pub tool_parameters: Option<serde_json::Value>,
    pub dependencies: Vec<String>,
    pub expected_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub request_id: String, // This should link to the original request
    pub user_query: String,
    pub plan_summary: String,
    pub tasks_definition: Vec<TaskDefinition>,
    pub task_results: HashMap<String, String>, // Task ID to result content
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub final_summary: Option<String>,
}

impl Plan {
    pub fn new(
        id: String,
        user_query: String,
        plan_summary: String,
        tasks_definition: Vec<TaskDefinition>,
    ) -> Self {
        Self {
            id,
            request_id: Uuid::new_v4().to_string(), // New UUID for request_id
            user_query,
            plan_summary,
            tasks_definition,
            task_results: HashMap::new(),
            status: PlanStatus::Created,
            created_at: Utc::now(),
            updated_at: None,
            final_summary: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanStatus {
    Created,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request_id: String,
    pub success: bool,
    pub output: String,
    pub plan_details: Option<Plan>,
}

use uuid::Uuid;
