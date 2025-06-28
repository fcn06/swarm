use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use a2a_rs::{Task, TaskState};

// Re-exporting Task from a2a_rs for direct use in the Plan, or define a similar one if it needs specific fields.
// For now, let's assume a2a_rs::domain::Task is suitable or we'll define our own PlannerTask.
// Let's define a PlannerTask that's similar to what was in planner_agent.rs for clarity.
// If a2a_rs::domain::Task is preferred, this can be replaced.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub request_id: String,
    pub user_query: String,   // Storing the original user query
    pub plan_summary: String, // Summary from LLM about the plan
    pub tasks_definition: Vec<TaskDefinition>, // Tasks to be executed
    pub task_results: HashMap<String, String>, // Store task_id -> result content
    // insert tasks_order:Vec<String>
    pub status: PlanStatus,
    pub final_summary: Option<String>, // Overall result summary for the user
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// Represents the expected JSON structure from the LLM for a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResponse {
    pub plan_summary: String,
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskDefinition {
    pub id: String,
    pub description: String,
    pub skill_to_use: Option<String>, // Specific skill required
    pub expected_outcome: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>, // IDs of tasks that must be completed before this one
    #[serde(default = "Utc::now")] // for tracking
    created_at: chrono::DateTime<chrono::Utc>,
    pub task_output: Option<String>, // New field to store the actual output of the task
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ExecutionPlan {
    id: String,
    plan_id: String,
    task_definition_id: String,
    task: Task,
    task_status: TaskState,
    assigned_agent_id: Option<String>,
    pub task_output: Option<String>, // New field to store the actual output of the task
}

/// Final outcome of the execution of the plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request_id: String,
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_details: Option<Plan>,
}

impl Plan {
    pub fn new(
        request_id: String,
        user_query: String,
        plan_summary: String,
        tasks_definition: Vec<TaskDefinition>,
    ) -> Self {
        let plan_id = Uuid::new_v4().to_string();
        let mut tasks = HashMap::new();
        let mut task_order = Vec::new();

        for task_def in &tasks_definition {
            let task_id = task_def.id.clone();
            let task = TaskDefinition {
                id: task_id.clone(),
                description: task_def.description.clone(),
                skill_to_use: task_def.skill_to_use.clone(),
                expected_outcome: task_def.expected_outcome.clone(),
                dependencies: task_def.dependencies.clone(),
                created_at: Utc::now(),
                task_output: None, // Initialize task_output as None
            };

            tasks.insert(task_id.clone(), task);
            task_order.push(task_id); // Initial order from LLM
        }

        Plan {
            id: plan_id,
            request_id,
            user_query,
            plan_summary,
            tasks_definition,
            task_results: HashMap::new(), // Initialize the new field
            status: PlanStatus::Pending,
            final_summary: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}
