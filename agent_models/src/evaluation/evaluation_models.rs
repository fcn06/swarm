use serde::{Deserialize, Serialize};
use std::collections::HashMap;




/// Represents the data received from the agent's log/message queue.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentEvaluationLogData {
    pub agent_id: String,
    pub request_id: String,
    pub conversation_id: String,
    pub step_id:Option<String>,
    pub original_user_query: String,
    pub agent_input: String,
    pub activities_outcome: HashMap<String, String>, // in case there are, store all activities processed by the agent
    pub agent_output: String,
    pub context_snapshot: Option<String>,
    pub success_criteria: Option<String>,
}

/// Represents the structured evaluation response from the Judge LLM.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgeEvaluation {
    pub rating: String,
    pub score: u8,
    pub feedback: String,
    pub suggested_correction: Option<String>,
}

/// The final combined data structure after evaluation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvaluatedAgentData {
    #[serde(flatten)]
    pub agent_log: AgentEvaluationLogData,
    pub evaluation: JudgeEvaluation,
    pub timestamp: String,
}


