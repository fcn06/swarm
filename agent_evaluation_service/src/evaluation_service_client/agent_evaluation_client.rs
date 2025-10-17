use reqwest::Client;
use anyhow::{Context, Result};
use tracing::{error};
use agent_models::evaluation::evaluation_models::{AgentEvaluationLogData,JudgeEvaluation};


#[derive(Debug,Clone)]
pub struct AgentEvaluationServiceClient {
    evaluation_service_url: String,
    client: Client,
}

impl AgentEvaluationServiceClient {
    pub fn new(evaluation_service_url: String) -> Self {
        AgentEvaluationServiceClient {
            evaluation_service_url,
            client: Client::new(),
        }
    }

    pub async fn log_evaluation(&self, log_data: AgentEvaluationLogData) -> Result<JudgeEvaluation> {
        let url = format!("{}/log", self.evaluation_service_url);
        
        let response = self.client.post(&url)
            .json(&log_data)
            .send()
            .await
            .context(format!("Failed to send evaluation request to {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let text_body = response.text().await
                .context("Failed to read error response body as text")?;
            error!("Evaluation service returned an error status: {}. Body: {}", status, text_body);
            anyhow::bail!("Evaluation service returned an error status: {} with body: {}", status, text_body)
        }

        response.json::<JudgeEvaluation>().await
            .context("Failed to decode evaluation service JSON response")
    }

      
    
}