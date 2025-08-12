use reqwest::{Client, Error};
use crate::evaluation_server::llm_judge::AgentLogData;

#[derive(Debug)]
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

    pub async fn log_evaluation(&self, log_data: AgentLogData) -> Result<String, Error> {
        let url = format!("{}/log", self.evaluation_service_url);
        
        let response = self.client.post(&url)
            .json(&log_data)
            .send()
            .await?;

        response.json::<String>().await
    }
}