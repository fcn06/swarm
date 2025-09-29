use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, warn,info};
use serde_json::Value;
use anyhow::anyhow;

use agent_core::agent_interaction_protocol::agent_interaction::AgentInteraction;
use agent_core::agent_interaction_protocol::a2a_agent_interaction::A2AAgentInteraction;
//use super::a2a_agent_interaction::A2AAgentInteraction;

use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
// The direct client is no longer used here, only the DiscoveryService trait
// use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use configuration::AgentReference;


use workflow_management::agent_communication::agent_invoker::AgentInvoker;
//use workflow_management::graph::graph_definition::Activity;

/// An AgentRunner that communicates using the A2A protocol over HTTP.
#[allow(dead_code)]
pub struct A2AAgentInvoker {
    agents_references: Vec<AgentReference>,
    client_agents: HashMap<String, A2AAgentInteraction>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    memory_service: Option<Arc<dyn MemoryService>>,
    discovery_service_client: Arc<dyn DiscoveryService>,
}

#[async_trait]
impl AgentInvoker for A2AAgentInvoker {

    /// This function is called by the workflow_runtime when an activity is delegated to an agent in order to execute an activity.
    #[allow(unused_variables)]
    async fn interact(&self, agent_id: String, message:  String, skill_to_use: String ) -> anyhow::Result<Value> {
       
        let agent_client = self
            .client_agents
            .get(&agent_id)
            .ok_or(anyhow!("Agent \'{}\' not found", agent_id))?;

        /******************************************************/
        // execute the task by remote agent
        let outcome=agent_client.execute_task(&message, "default_skill").await?;
        
        debug!("A2AAgentInvoker : {}",outcome);

        //Ok(outcome)
        Ok(serde_json::Value::String(outcome))

        // MOCK IMPLEMENTATION: Return a valid JSON object.
        // Ok("\"Mock_Agent_Runner_Default_Response\"".to_string())

    }
}

impl A2AAgentInvoker {
    /// This function instantiate an AgentRunner 
    pub async fn new(
        agents_references: Vec<AgentReference>,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service_client: Arc<dyn DiscoveryService>,
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

    /// This function retrieves a list of clients agents , the list of agents that are referenced
    async fn connect_to_a2a_agents(
        agents_references: &[AgentReference],
    ) -> anyhow::Result<HashMap<String, A2AAgentInteraction>> {
        let mut client_agents = HashMap::new();

        debug!("Connecting to A2A server agents...");
        for agent_reference in agents_references {
            let agent_details = agent_reference.get_agent_reference().await?;

            debug!(
                "Connecting to agent \'{}\' at {}",
                agent_details.name, agent_details.url
            );

            match A2AAgentInteraction::new(agent_details.name.clone(), agent_details.url.clone())
                .await
            {
                Ok(client) => {
                    debug!(
                        "Successfully connected to agent \'{}\' at {}",
                        client.id, client.uri
                    );
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    debug!(
                        "Warning: Failed to connect to A2A agent \'{}\' at {}: {}",
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

    #[allow(dead_code)]
    async fn find_agent_with_skill(&self, skill: &str, _task_id: &str) -> Option<&A2AAgentInteraction> {

        // 1. Try to find the agent with appropriate skill 
        for (agent_id, agent) in &self.client_agents {
            info!("WorkFlow Management: agent_id : \'{}\' with skill \'{}\'",agent_id, skill);
            // Access skills directly from the A2AClient struct
            if agent.has_skill(skill) {
                // Use the has_skill method
                info!("WorkFlow Management: Found agent \'{}\' with skill \'{}\'",agent_id, skill);
                return Some(agent);
            }
        }

         // 2. If no agent with the specific skill is found, try to find the default agent
         warn!("WorkFlow Management: No agent found with skill \'{}\' . Attempting to find default agent.", skill);

         for agent_ref_config in &self.agents_references {
             if agent_ref_config.is_default == Some(true) {
                 // We need to find the A2AClient instance associated with this default SimpleAgentReference
                 // We can do this by matching the name or ID. Assuming client.id is agent_reference.name
                 if let Some(default_agent_client) = self.client_agents.get(&agent_ref_config.name) {
                     info!(
                         "WorkFlow Management: Found default agent \'{}\' as fallback.",
                         default_agent_client.id
                     );
                     return Some(default_agent_client);
                 }
             }
         }
 
         // 3. If no agent with the skill and no default agent are found
         warn!("WorkFlow Management: No suitable agent (skill-matching or default) found for skill \'{}\'", skill);
         None
    }


}


