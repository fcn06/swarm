use reqwest::{Client};
//use crate::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
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

    pub async fn log_evaluation(&self, log_data: AgentEvaluationLogData) -> anyhow::Result<JudgeEvaluation> {
        let url = format!("{}/log", self.evaluation_service_url);
        
        let response = self.client.post(&url)
            .json(&log_data)
            .send()
            .await?;

        Ok(response.json::<JudgeEvaluation>().await?)
    }

      
    
}