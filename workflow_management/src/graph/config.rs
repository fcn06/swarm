use super::graph_definition::{Edge, Graph, Node, NodeType, Task, TaskStatus};
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
        let new_task = Task {
            id: task.id.clone(),
            name: task.task_type.clone(),
            description: format!("Task of type {}", task.task_type),
            status: TaskStatus::Pending,
            sub_tasks: None,
            required_resources: Vec::new(),
        };

        let node = Node {
            id: task.id.clone(),
            node_type: NodeType::Task(new_task),
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
