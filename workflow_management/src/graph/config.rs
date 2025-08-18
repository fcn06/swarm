use super::graph_definition::{Edge, Graph, Node, NodeType, ActivityType, Activity, Dependency};

//use chrono::Utc;

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
struct JsonActivity {
    activity_type: ActivityType,
    id: String,
    description: String,
    #[serde(rename = "type")]
    r#type: Option<String>,
    skill_to_use: Option<String>,
    assigned_agent_id_preference: Option<String>,
    tool_to_use: Option<String>,
    tool_parameters: Option<serde_json::Value>,
    dependencies: Vec<JsonDependency>,
    expected_outcome: Option<String>,
}

#[derive(Deserialize, Debug,Clone)]
struct JsonDependency {
    source: String,
    condition: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonWorkflow {
    plan_name: String,
    activities: Vec<JsonActivity>,
}

pub fn load_graph_from_file(file_path: &str) -> Result<Graph, ConfigurationError> {
    let content = fs::read_to_string(file_path)?;
    let workflow: JsonWorkflow = serde_json::from_str(&content)?;

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    for activity in workflow.activities {
        let activity_definition = Activity {
            activity_type: activity.activity_type,
            id: activity.id.clone(),
            description: activity.description.clone(),
            r#type: activity.r#type,
            skill_to_use: activity.skill_to_use,
            assigned_agent_id_preference: activity.assigned_agent_id_preference,
            tool_to_use: activity.tool_to_use,
            tool_parameters: activity.tool_parameters,
            dependencies: activity.dependencies.clone().into_iter().map(|d| Dependency { source: d.source }).collect(),
            expected_outcome: activity.expected_outcome,
            activity_output: None,
        };

        let node = Node {
            id: activity.id.clone(),
            node_type: NodeType::Activity(activity_definition),
        };
        nodes.insert(activity.id.clone(), node);

        for dep in activity.dependencies {
            edges.push(Edge {
                source: dep.source,
                target: activity.id.clone(),
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
