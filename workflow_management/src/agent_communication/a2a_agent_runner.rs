use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, warn};

use agent_core::agent_interaction_protocol::agent_interaction::AgentInteraction;
use agent_core::business_logic::services::{EvaluationService, MemoryService};
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use configuration::AgentReference;

use crate::agent_communication::a2a_agent_interaction::A2AAgentInteraction;
use crate::agent_communication::agent_runner::AgentRunner;
use crate::graph::graph_definition::Activity;

/// An AgentRunner that communicates using the A2A protocol over HTTP.
#[allow(dead_code)]
pub struct A2AAgentRunner {
    agents_references: Vec<AgentReference>,
    client_agents: HashMap<String, A2AAgentInteraction>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    memory_service: Option<Arc<dyn MemoryService>>,
    discovery_service_client: Arc<AgentDiscoveryServiceClient>,
}

#[async_trait]
impl AgentRunner for A2AAgentRunner {
    fn name(&self) -> String {
        "a2a_http_runner".to_string()
    }

    /// This function is called by the workflow_runtime when an activity is delegated to an agent.
    async fn invoke(
        &self,
        activity: &Activity,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let preferred_agent_id = activity
            .assigned_agent_id_preference
            .as_ref()
            .ok_or("Missing agent ID preference in activity definition")?;

        let _agent_client = self
            .client_agents
            .get(preferred_agent_id)
            .ok_or(format!("Agent '{}' not found", preferred_agent_id))?;

        // MOCK IMPLEMENTATION: Return a valid JSON object.
        // In a real implementation, you would use the `_agent_client` to make a remote call.
        if activity.id == "fetch_customer_data" {
            let mock_response = json!({
                "result": {
                    "name": "John Doe",
                    "email": "john.doe@example.com",
                    "address": {
                        "street": "123 Main St",
                        "city": "New York"
                    }
                }
            });
            return Ok(mock_response.to_string());
        }

        // Default mock response for other activities
        Ok("\"Mock_Agent_Runner_Default_Response\"".to_string())
    }
}

impl A2AAgentRunner {
    pub async fn new(
        agents_references: Vec<AgentReference>,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service_client: Arc<AgentDiscoveryServiceClient>,
    ) -> anyhow::Result<Self> {
        let client_agents = Self::connect_to_a2a_agents(&agents_references).await?;

        Ok(Self {
            agents_references,
            client_agents,
            evaluation_service,
            memory_service,
            discovery_service_client,
        })
    }

    async fn connect_to_a2a_agents(
        agents_references: &[AgentReference],
    ) -> anyhow::Result<HashMap<String, A2AAgentInteraction>> {
        let mut client_agents = HashMap::new();

        debug!("Connecting to A2A server agents...");
        for agent_reference in agents_references {
            let agent_details = agent_reference.get_agent_reference().await?;

            debug!(
                "Connecting to agent '{}' at {}",
                agent_details.name, agent_details.url
            );

            match A2AAgentInteraction::new(agent_details.name.clone(), agent_details.url.clone())
                .await
            {
                Ok(client) => {
                    debug!(
                        "Successfully connected to agent '{}' at {}",
                        client.id, client.uri
                    );
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    debug!(
                        "Warning: Failed to connect to A2A agent '{}' at {}: {}",
                        agent_details.name, agent_details.url, e
                    );
                }
            }
        }

        if client_agents.is_empty() && !agents_references.is_empty() {
            warn!(
                "Warning: No A2A server agents connected, planner capabilities will be limited."
            );
        }
        Ok(client_agents)
    }
}
