use a2a_rs::client::A2aClient as ExternalA2aClient;
use a2a_rs::domain::{Message, Part, Skill, TaskState};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use url::Url;

/// A wrapper around the external A2aClient to provide simplified interactions
#[derive(Clone)]
pub struct A2AClient {
    pub id: String,
    pub uri: String,
    client: ExternalA2aClient,
    skills: Vec<Skill>,
}

impl A2AClient {
    /// Connects to an A2A agent server and fetches its skills.
    pub async fn connect(id: String, uri: String) -> Result<Self> {
        let client = ExternalA2aClient::connect(Url::parse(&uri)?)?; // Using the external client
        let skills = client.get_skills().await?;
        Ok(Self { id, uri, client, skills })
    }

    /// Executes a task on the connected A2A agent.
    pub async fn execute_task(&self, task_description: &str, skill_id: &str) -> Result<String> {
        tracing::info!(
            "A2AClient: Executing task '{}' with skill '{}' on agent '{}'",
            task_description, skill_id, self.id
        );

        let message_id = uuid::Uuid::new_v4().to_string();
        let request_message = Message::user_text(task_description.to_string(), message_id.clone());

        match self.client.send_message(request_message).await {
            Ok(response_message) => {
                tracing::debug!("A2AClient: Received response from agent '{}'.", self.id);
                // Process the response message to extract the relevant part
                self.extract_text_from_message(&response_message).await
            }
            Err(e) => {
                tracing::error!("A2AClient: Failed to send message to agent '{}': {:?}", self.id, e);
                Err(anyhow!("Failed to execute task on agent '{}': {:?}", self.id, e))
            }
        }
    }

    /// Returns the skills provided by this A2A agent.
    pub fn get_skills(&self) -> &Vec<Skill> {
        &self.skills
    }

    /// Checks if the agent has a specific skill.
    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.skills.iter().any(|s| s.id == skill_id)
    }

    // Helper function to extract text from a Message
    async fn extract_text_from_message(&self, message: &Message) -> Result<String> {
        message
            .parts
            .iter()
            .filter_map(|part| {
                if let Part::Text { text, metadata: _ } = part {
                    Some(text.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("")
            .pipe(Ok)
    }
}

// A simple trait to allow piping values, similar to .into() but more flexible
trait Pipe<T> {
    fn pipe(self, f: impl FnOnce(Self) -> T) -> T;
}

impl<T> Pipe<T> for String {
    fn pipe(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
