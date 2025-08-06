use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::env;
use chrono::Utc;

// --- Data Structures for Input and Output ---

/// Represents the data received from the agent's log/message queue.
/// This struct holds all information necessary for the Judge LLM to evaluate.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentLogData {
    pub agent_id: String,
    pub request_id: String,
    pub step_id: String,
    pub original_user_query: String,
    pub agent_input: String,
    pub agent_output: String,
    pub context_snapshot: Option<String>, // Optional: Additional context for evaluation
    pub success_criteria: Option<String>, // Optional: Explicit criteria for evaluation
}

/// Represents the structured evaluation response from the Judge LLM.
#[derive(Debug, Serialize, Deserialize)]
pub struct JudgeEvaluation {
    pub rating: String, // e.g., "Good", "Needs Improvement", "Failed"
    pub score: u8,      // e.g., 1-10
    pub feedback: String, // Detailed textual feedback
    pub suggested_correction: Option<String>, // Optional: Corrected version of the output
}

/// The final combined data structure after evaluation.
#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluatedAgentData {
    #[serde(flatten)] // Flatten AgentLogData fields into this struct
    pub agent_log: AgentLogData,
    pub evaluation: JudgeEvaluation,
    pub timestamp: String, // When the evaluation happened
}

// --- LLM Interaction Logic ---

/// Error type for evaluation service.
#[derive(Debug)]
pub enum EvaluationError {
    Environment(env::VarError),
    Network(reqwest::Error),
    Serialization(serde_json::Error),
    LLMError(String), // For errors returned by the LLM itself
}

impl From<env::VarError> for EvaluationError {
    fn from(err: env::VarError) -> Self {
        EvaluationError::Environment(err)
    }
}

impl From<reqwest::Error> for EvaluationError {
    fn from(err: reqwest::Error) -> Self {
        EvaluationError::Network(err)
    }
}

impl From<serde_json::Error> for EvaluationError {
    fn from(err: serde_json::Error) -> Self {
        EvaluationError::Serialization(err)
    }
}

/// Main function to evaluate agent output using a Judge LLM.
/// This function should be called asynchronously by your trigger mechanism (e.g., Cloud Function).
pub async fn evaluate_agent_output(
    log_data: AgentLogData,
) -> Result<EvaluatedAgentData, EvaluationError> {
    // Load environment variables (e.g., LLM_API_KEY, LLM_ENDPOINT)
    dotenv::dotenv().ok();

    let llm_api_key = env::var("LLM_API_KEY")?;
    let llm_endpoint = env::var("LLM_ENDPOINT")?;

    // 1. Construct the Judge LLM Prompt
    let prompt = format!(
        r#"You are an expert AI evaluator. Your task is to assess the provided 'Agent Output' based on the 'Original User Query' and 'Context/Criteria'.
Original User Query: {}
Agent Input for this step: {}
Agent Output: {}
Context/Criteria: {}

Please provide a concise evaluation, focusing on:
1. Accuracy: Does the output correctly address the user's intent?
2. Completeness: Is all necessary information present?
3. Compliance: Does it meet any implicit or explicit constraints from the query or context?
4. Areas for Improvement: What specifically could be done better?

Respond in a structured JSON format. The JSON should be wrapped in triple backticks (```json...```) if the LLM sometimes wraps it, otherwise just the JSON object itself:
```json
{{
  "rating": "Good" | "Needs Improvement" | "Failed",
  "score": [1-10],
  "feedback": "Detailed textual feedback on accuracy, completeness, and compliance, with concrete suggestions for improvement.",
  "suggested_correction": "If applicable, a corrected or improved version of the output."
}}
```"#,
        log_data.original_user_query,
        log_data.agent_input,
        log_data.agent_output,
        log_data.context_snapshot.as_deref().unwrap_or("No specific context provided."),
    );

    // 2. Prepare the LLM API Request
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", llm_api_key))
        .map_err(|e| EvaluationError::LLMError(format!("Invalid API Key header: {}", e)))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Example payload for a generic LLM. Adjust according to your LLM's API spec.
    let llm_request_payload = serde_json::json!({{ // Double curly braces for literal braces in format! string
        "model": "gemini-pro", // Or your specific LLM model ID
        "messages": [
            {{
                "role": "user",
                "content": prompt
            }}
        ]
    }});

    let res = client
        .post(&llm_endpoint)
        .headers(headers)
        .json(&llm_request_payload)
        .send()
        .await?;

    let response_text = res.text().await?;

    // 3. Parse the LLM's Response
    let parsed_json_str = if response_text.contains("```json") {
        response_text
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(&response_text)
            .trim()
            .to_string()
    } else {
        response_text.trim().to_string()
    };

    let judge_evaluation: JudgeEvaluation = serde_json::from_str(&parsed_json_str)
        .map_err(|e| EvaluationError::Serialization(
            serde_json::Error(format!("Failed to parse LLM response JSON: {}. Raw: {}", e, parsed_json_str))
        ))?;

    // 4. Combine original data with evaluation
    let evaluated_data = EvaluatedAgentData {
        agent_log: log_data,
        evaluation: judge_evaluation,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(evaluated_data)
}
