#[allow(unused_imports)]
use a2a_rs::{
    HttpClient,
    domain::{ AgentSkill},
};

use std::sync::Arc;
use std::collections::HashMap;

use async_trait::async_trait;

use tracing::{warn,debug,info};

use crate::agent_communication::agent_runner::AgentRunner;
use crate::agent_communication::a2a_agent_interaction::A2AAgentInteraction;
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;

use agent_core::business_logic::services::EvaluationService;
use agent_core::business_logic::services::MemoryService;
// Removed: use agent_core::planning::plan_definition::TaskDefinition;
use agent_core::agent_interaction_protocol::agent_interaction::AgentInteraction;

use configuration::AgentReference;
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

    /// function that is called by the workflow_runtime when an activity is requested to an agent
    async fn invoke(
        &self,
        activity: &Activity,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        
        // retrieve preferred_agent_id from the activity
        let preferred_agent_id = activity.assigned_agent_id_preference.as_ref().ok_or("Missing agent ID preference in activity definition")?;

        // retrieve the http client of the remote agent , based on preferred_agent_id
        let agent_client = self.client_agents.get(preferred_agent_id).ok_or(format!("Agent '{}' not found", preferred_agent_id))?;

        // execute the task by remote agent
        //let outcome=agent_client.execute_task(&activity.description, &activity.skill_to_use.clone().unwrap_or_else(|| "default_skill".to_string())).await?;
        //info!("{}",outcome);

        // We need to ensure we pass proper task request for the agent to execute

        Ok("Mock_Agent_Runner_Invoke_Response".to_string())

    }
}


impl A2AAgentRunner {
    pub async fn new( 
        agents_references: Vec<AgentReference>,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service_client: Arc<AgentDiscoveryServiceClient>) -> anyhow::Result<Self> {
       
        // Get all clients for each agent defined in config
        let client_agents = Self::connect_to_a2a_agents(&agents_references).await?;   

        Ok(Self { 
            agents_references: agents_references, 
            client_agents: client_agents, 
            evaluation_service: evaluation_service, 
            memory_service: memory_service,
            discovery_service_client:discovery_service_client, 
        })
    
    }

    async fn connect_to_a2a_agents(agents_references: &[AgentReference]) -> anyhow::Result<HashMap<String, A2AAgentInteraction>> {
        let mut client_agents = HashMap::new();

        debug!("Full Agent: Connecting to A2a server agents...");
        for agent_reference in agents_references {
            // Use agent_info (which implements AgentInfoProvider) to get details for connection
            let agent_reference_details = agent_reference.get_agent_reference().await?;

            debug!(
                "FullAgent: Connecting to agent '{}' at {}",
                agent_reference_details.name, agent_reference_details.url
            );

            match A2AAgentInteraction::new(agent_reference_details.name.clone(), agent_reference_details.url.clone())
                .await
            {
                Ok(client) => {
                    debug!(
                        "FullAgent: Successfully connected to agent '{}' at {}",
                        client.id, client.uri
                    );
                    // Use the connected client's ID as the key
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    // Use details from agent_info for error reporting
                    debug!(
                        "FullAgent: Warning: Failed to connect to A2a agent '{}' at {}: {}",
                        agent_reference_details.name, agent_reference_details.url, e
                    );
                }
            }
        }

        if client_agents.is_empty() && !agents_references.is_empty() {
            warn!(
                "FullAgent: Warning: No A2a server agents connected, planner capabilities will be limited to direct LLM interaction if any."
            );
        }
        Ok(client_agents)
    }




}


