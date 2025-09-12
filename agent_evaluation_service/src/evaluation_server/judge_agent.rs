use serde::{Deserialize, Serialize};
use std::env;
use chrono::Utc;
use llm_api::chat::{ChatLlmInteraction};
use anyhow::{Context, Result};
use std::fs;

use tracing::trace;


use configuration::AgentConfig;

// todo:move in agent_core and rename in EvaluationLogData
/// Represents the data received from the agent's log/message queue.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentLogData {
    pub agent_id: String,
    pub request_id: String,
    pub conversation_id: String,
    pub step_id:Option<String>,
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
        let llm_a2a_api_key = env::var("LLM_JUDGE_API_KEY").expect("LLM_JUDGE_API_KEY must be set");

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

        // Read the prompt template from the file
        let prompt_template = fs::read_to_string("./configuration/prompts/judge_agent_prompt.txt")
            .context("Failed to read judge_agent_prompt.txt")?;

        let prompt = prompt_template
            .replacen("{}", &log_data.original_user_query, 1)
            .replacen("{}", &log_data.agent_input, 1)
            .replacen("{}", &log_data.agent_output, 1)
            .replacen("{}", &log_data.context_snapshot.as_deref().unwrap_or("No specific context provided."), 1);
        
        let response = self.llm_interaction.call_api_simple("user".to_string(), prompt).await?;

        let response_content = response
            .and_then(|msg| msg.content)
            .ok_or_else(|| anyhow::anyhow!("LLM response content is empty"))?;

        trace!("LLM Judge response: {}", response_content);
        
        let judge_evaluation: JudgeEvaluation = serde_json::from_str(&response_content)?;

        trace!("Judge Evaluation Structured Answer : {:?}",judge_evaluation );

        let evaluated_data = EvaluatedAgentData {
            agent_log: log_data,
            evaluation: judge_evaluation,
            timestamp: Utc::now().to_rfc3339(),
        };

        Ok(evaluated_data)
    }

}



