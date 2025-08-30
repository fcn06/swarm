use serde::{Deserialize, Serialize};

/// Final outcome of the execution of the plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request_id: String,
    pub conversation_id: String,
    pub success: bool,
    pub output: String,
}
