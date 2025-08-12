use serde::{Deserialize, Serialize};
use std::env;
use chrono::Utc;
use llm_api::chat::{ChatLlmInteraction, Message};
use anyhow::Result;



use agent_protocol_backbone::config::agent_config::AgentConfig;

/// Represents the data received from the agent's log/message queue.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentLogData {
    pub agent_id: String,
    pub request_id: String,
    pub step_id: String,
    pub original_user_query: String,
    pub agent_input: String,
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
    pub agent_log: AgentLogData,
    pub evaluation: JudgeEvaluation,
    pub timestamp: String,
}


/// Modern A2A server setup 
#[derive(Clone)]
pub struct JudgeAgent {
    llm_interaction: ChatLlmInteraction,
}


impl JudgeAgent {

    /// Creation of a new simple a2a agent
    pub async fn new(agent_config: AgentConfig ) -> anyhow::Result<Self> {

        // Set model to be used
        let model_id = agent_config.agent_model_id();

        // Set system message to be used
        let _system_message = agent_config.agent_system_prompt();

        // Set API key for LLM
        let llm_a2a_api_key = env::var("LLM_Judge_API_KEY").expect("LLM_A2A_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
            agent_config.agent_llm_url(),
            model_id,
            llm_a2a_api_key,
        );

        Ok(Self {
            llm_interaction,
        })

        }
    

    /// Main function to evaluate agent output using a Judge LLM.
    pub async fn evaluate_agent_output(&self,log_data: AgentLogData) -> Result<EvaluatedAgentData> {

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

    Respond in a structured JSON format:
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

        
        let response = self.llm_interaction.call_api_simple("user".to_string(), prompt).await?;

        let response_content = response
            .and_then(|msg| msg.content)
            .ok_or_else(|| anyhow::anyhow!("LLM response content is empty"))?;

        let judge_evaluation: JudgeEvaluation = serde_json::from_str(&response_content)?;

        let evaluated_data = EvaluatedAgentData {
            agent_log: log_data,
            evaluation: judge_evaluation,
            timestamp: Utc::now().to_rfc3339(),
        };

        Ok(evaluated_data)
    }

}



