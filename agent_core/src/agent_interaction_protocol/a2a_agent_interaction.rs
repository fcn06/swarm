use a2a_rs::{
    HttpClient,
    domain::{ Message, Part,AgentSkill},
    services::AsyncA2AClient,
};

use async_trait::async_trait;
use tracing::{info,warn,debug,error};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use super::agent_interaction::AgentInteraction;

// implementation of the AgentInteraction with an A2A agent
// Protocol to interact with a single agent


/// This structure enable Interaction with an A2A enabled single Agent
#[derive(Clone)]
pub struct A2AAgentInteraction {
    /// The Id of the agent
    pub id: String,
    /// The uri of the agent
    pub uri: String,
    /// The Skills of the Agent
    pub skills:Vec<AgentSkill>, // Skills this agent offers
    /// An Http Client to communicate with the agent
    client: Arc<HttpClient>, // Assuming HttpClient is part of a2a_rs or defined elsewhere
}


#[async_trait]
impl AgentInteraction for A2AAgentInteraction {
    // Connect to an A2A server agent and potentially fetch its skills.
    async fn new(id: String, uri: String) -> Result<Self> {
        // create a client to remopte agent
        let client = HttpClient::new(uri.clone());

        // Get skills from remote agents
        let http_client = reqwest::Client::new();
        let response = http_client
        .get(format!("{}/skills",uri))
        .send()
        .await
        .expect("Failed to fetch skills");

        let skills: Vec<AgentSkill> = response.json().await.expect("Failed to parse skills");
       

        Ok(A2AAgentInteraction {
            id: id.clone(),
            uri: uri.to_string(),
            skills: skills,
            client: Arc::new(client),
        })
    }


    /// Execute a task on the A2A server agent.
    async fn execute_task(&self, task_description: &str, _skill_to_use: &str) -> Result<String> {
        
        ////////////////////////////////////////////////////////////////////////////////
        // EXAMPLE OF REAL WORLD TASK EXECUTION

        // Generate a task ID
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        info!("Created task with ID: {}", task_id);

        // Create a message
        let message_id = uuid::Uuid::new_v4().to_string();
        let message = Message::agent_text(task_description.to_string(), message_id);

        // Exponential re start in case of rate limiting
        let mut retries = 0;
        let max_retries = 3; // You can adjust this
        let mut delay = Duration::from_secs(1); // Starting delay

        let task = loop {
            info!("Sending message to task...");
            match self
                .client
                .send_task_message(&task_id, &message, None, Some(50))
                .await
            {
                Ok(t) => break t,
                Err(e) => {
                    retries += 1;
                    if retries > max_retries {
                        error!("Failed to send task message after {} retries: {}", max_retries, e);
                        return Err(e.into()); // Return the error if max retries reached
                    }
                    warn!("Failed to send task message. Retrying in {:?}... (Retry {}/{})\nError: {}", delay, retries, max_retries, e);
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        };

        // Response of send_task_message is  :Result<Task, A2AError>;
        let response = task
            .status
            .message
            .unwrap()
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        debug!("Received response: {:?}", response);

        Ok(response)

        ////////////////////////////////////////////////////////////////////////////////
    }


    fn get_skills(&self) -> &[AgentSkill] {
        &self.skills
    }

    /// Check if the agent has a specific skill.
    fn has_skill(&self, skill_name: &str) -> bool {
        for skill in &self.skills {
            if skill.id .contains(skill_name) ||
               skill.name.contains(skill_name) ||
               skill.description.contains(skill_name)  {
                debug!("Agent {} has skill: {}", self.id, skill_name);
                return true;
            }
        }
        debug!("Agent {} does NOT have skill: {}", self.id, skill_name);
        false
    }


    /// Get the agent's ID.
    fn agent_id(&self) -> String {self.id.clone()}
    /// Get the agent's uri.
    fn agent_uri(&self) -> String {self.uri.clone()}
    /// Get the agent's skills.
    fn agent_skills(&self) -> Vec<AgentSkill>  {self.skills.clone()}
    /// Get the agent's remote endpoint
    fn agent_remote_http_client(&self) -> Arc<HttpClient> {self.client.clone()}

}