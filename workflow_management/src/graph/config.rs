use agent_core::graph::graph_definition::{ Graph,  WorkflowPlanInput};

use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

pub fn load_graph_from_file(file_path: &str) -> Result<Graph, ConfigurationError> {
    let content = fs::read_to_string(file_path)?;
    let workflow: WorkflowPlanInput = serde_json::from_str(&content)?;

    // Use the From trait implementation to convert WorkflowPlanInput to Graph
    Ok(workflow.into())
}
