use reqwest::{Client, Error};
use crate::evaluation_server::judge_agent::AgentLogData;

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

    pub async fn log_evaluation(&self, log_data: AgentLogData) -> anyhow::Result<String> {
        let url = format!("{}/log", self.evaluation_service_url);
        
        let response = self.client.post(&url)
            .json(&log_data)
            .send()
            .await?;

        Ok(response.json::<String>().await?)
    }
}