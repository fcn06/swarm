use serde::{Deserialize, Serialize};
use super::graph_definition::ActivityType;



/// **High-Level PLan**
/// 
/// **Resource Template**
/// A simplified representation of an available resource (tool, agent, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    pub id: String,
    pub description: String,
    pub activity_type: ActivityType,
    pub r#type: Option<String>,
    pub agent_to_use: Option<String>,
    pub tool_to_use: Option<String>,
    pub task_to_use: Option<String>,
}

/// **Stage 2: High-Level Workflow**
/// A sequence of activities linking abstract plan steps to concrete resources.
#[derive(Debug, Clone)]
pub struct HighLevelActivity {
    pub step_description: String,
    pub resource: ResourceTemplate,
}

#[derive(Debug, Clone)]
pub struct HighLevelPLan {
    pub plan_name: String,
    pub high_level_activities: Vec<HighLevelActivity>,
}

// Example High Level Plan
// Agent A (Data Fetcher): "Fetches data from various data sources like databases, APIs, or file systems."
// Tool B (Sentiment Analyzer): "Analyzes text for sentiment (positive, negative, neutral) and extracts key entities."
// Agent C (Report Generator): "Compiles data into a structured report, supporting various formats like PDF or Markdown."
// Tool D (Email Sender): "Sends emails to specified recipients with attached documents or inline content."


