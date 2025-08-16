use super::graph_definition::{Edge, Graph, Node, NodeType};
use agent_protocol_backbone::planning::plan_definition::TaskDefinition;
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

#[derive(Deserialize, Debug)]
struct JsonTask {
    id: String,
    #[serde(rename = "type")]
    task_type: String,
    description: Option<String>,
    dependencies: Vec<String>,
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
            description: task.description.unwrap_or(task.task_type),
            skill_to_use: None,
            tool_to_use: None,
            tool_parameters: None,
            assigned_agent_id_preference: None,
            expected_outcome: None,
            dependencies: task.dependencies.clone(),
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
                source: dep,
                target: task.id.clone(),
            });
        }
    }

    Ok(Graph {
        id: workflow.plan_name,
        nodes,
        edges,
    })
}
