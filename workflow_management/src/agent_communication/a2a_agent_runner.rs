use crate::agent_communication::agent_runner::AgentRunner;
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;

//use agent_protocol_backbone::a2a_protocol::{A2ARequest, A2AResponse};

use agent_protocol_backbone::planning::plan_definition::TaskDefinition;
use async_trait::async_trait;
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

/// An AgentRunner that communicates using the A2A protocol over HTTP.
pub struct A2AAgentRunner {
    discovery_client: Arc<AgentDiscoveryServiceClient>,
}

impl A2AAgentRunner {
    pub fn new(discovery_client: Arc<AgentDiscoveryServiceClient>) -> Self {
        Self { discovery_client }
    }
}

#[async_trait]
impl AgentRunner for A2AAgentRunner {
    fn name(&self) -> String {
        "a2a_http_runner".to_string()
    }

    async fn invoke(
        &self,
        task: &TaskDefinition,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let agent_id = task.assigned_agent_id_preference.as_ref().ok_or("Missing agent ID preference in task definition")?;

        /******************************************************/
        // TO BE REVIEWED
        /******************************************************/

        /* 

        // 1. Discover the agent's endpoint
        let agent_address = self.discovery_client.get_agent_address(agent_id.clone()).await?
            .ok_or(format!("Agent '{}' not found in discovery service", agent_id))?;
        
        let endpoint = Url::parse(&agent_address)?.join("/handle_request")?;

        // 2. Format the request according to the A2A protocol
        let a2a_request = A2ARequest {
            request_id: Uuid::new_v4().to_string(),
            source_agent_id: "workflow_orchestrator".to_string(), // Or a more dynamic ID
            destination_agent_id: agent_id.clone(),
            task_definition: task.clone(),
        };

        // 3. Send the request
        let client = reqwest::Client::new();
        let response = client.post(endpoint)
            .json(&a2a_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Agent request failed with status: {}", response.status()).into());
        }

        // 4. Parse the response
        let a2a_response: A2AResponse = response.json().await?;

        Ok(a2a_response.output)

        */

        /******************************************************/


        Ok("Mock_Agent_Response".to_string())

    }
}
