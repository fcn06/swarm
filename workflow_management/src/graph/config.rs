use super::graph_definition::{Edge, Graph, Node, NodeType};
use agent_core::planning::plan_definition::TaskDefinition;
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

/// Definitions of the fields in the workflow definition
#[derive(Deserialize, Debug)]
struct JsonTask {
    id: String,
    #[serde(rename = "type")]
    skill_to_use: String,
    description: Option<String>,
    assigned_agent_id_preference:Option<String>,
    dependencies: Vec<JsonDependency>,
}

#[derive(Deserialize, Debug)]
struct JsonDependency {
    source: String,
    condition: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonWorkflow {
    plan_name: String,
    tasks: Vec<JsonTask>,
}

pub fn load_graph_from_file(file_path: &str) -> Result<Graph, ConfigurationError> {
    let content = fs::read_to_string(file_path)?;
    let workflow: JsonWorkflow = serde_json::from_str(&content)?;

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    for task in workflow.tasks {
        let task_definition = TaskDefinition {
            id: task.id.clone(),
            description: task.description.unwrap_or_else(|| task.skill_to_use.clone()),
            skill_to_use: Some(task.skill_to_use),
            tool_to_use: None,
            tool_parameters: None,
            assigned_agent_id_preference: Some(task.assigned_agent_id_preference.clone().expect("REASON")),
            expected_outcome: None,
            dependencies: task.dependencies.iter().map(|d| d.source.clone()).collect(),
            created_at: Utc::now(),
            task_output: None,
        };

        let node = Node {
            id: task.id.clone(),
            node_type: NodeType::Task(task_definition),
        };
        nodes.insert(task.id.clone(), node);

        for dep in task.dependencies {
            edges.push(Edge {
                source: dep.source,
                target: task.id.clone(),
                condition: dep.condition,
            });
        }
    }

    Ok(Graph {
        id: workflow.plan_name,
        nodes,
        edges,
    })
}
