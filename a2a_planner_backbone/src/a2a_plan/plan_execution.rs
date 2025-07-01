use a2a_rs::{
    HttpClient,
    domain::{ Message, Part,AgentSkill},
    services::AsyncA2AClient,
};
use anyhow::Result;

use std::sync::Arc;

/////////////////////////////////////////////////////////
// Client to connect to a2a server
/////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct A2AClient {
    pub id: String,
    pub uri: String,
    //pub skills: Vec<String>, // Skills this agent offers - Old version
    pub skills:Vec<AgentSkill>, // Skills this agent offers
    // In a real implementation, this would hold the actual HTTP client or connection pool
    client: Arc<HttpClient>, // Assuming HttpClient is part of a2a_rs or defined elsewhere
}

impl A2AClient {
    // Connect to an A2A server agent and potentially fetch its skills.
    pub async fn connect(id: String, uri: String) -> Result<Self> {
        // create a client to remopte agent
        let client = HttpClient::new(uri.clone());

        // check formatting, for testing purpose
        //println!("{}",format!("{}/skills",uri));

        // Get skills from remote agents
        let http_client = reqwest::Client::new();
        let response = http_client
        .get(format!("{}/skills",uri))
        .send()
        .await
        .expect("Failed to fetch skills");

        // THIS NEEDS TO BE REWORKED
        // We should check on the full list of skills 
        // Old version returning String for skills instead of AgentSkill
        //let skills_json: Vec<Value> = response.json().await.expect("Failed to parse skills");
        //let skills = vec![serde_json::to_string(&(skills_json[0]["id"])).unwrap()];

        let skills: Vec<AgentSkill> = response.json().await.expect("Failed to parse skills");
       
        // In a real scenario, you might add a check here if the connection was successful
        // For now, we assume it is.

        Ok(A2AClient {
            id: id.clone(),
            uri: uri.to_string(),
            skills: skills,
            client: Arc::new(client),
        })
    }

    /// Execute a task on the A2A server agent.
    /// In a real scenario, this would make an API call to the agent.
    pub async fn execute_task(&self, task_description: &str, _skill_to_use: &str) -> Result<String> {
        ////////////////////////////////////////////////////////////////////////////////
        // EXAMPLE OF REAL WORLD TASK EXECUTION

        // Generate a task ID
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        println!("Created task with ID: {}", task_id);

        // Create a message
        let message_id = uuid::Uuid::new_v4().to_string();
        let message = Message::agent_text(task_description.to_string(), message_id);

        // Send a task message
        println!("Sending message to task...");
        let task = self
            .client
            .send_task_message(&task_id, &message, None, Some(50))
            .await?;
        // Response of send_task_message is  :Result<Task, A2AError>;
        // Simulate potential failure for demonstration
        // if task_description.contains("fail") {
        //     bail!("Simulated task failure");
        // }
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

        println!("Received response: {:?}", response);

        Ok(response)

        ////////////////////////////////////////////////////////////////////////////////
    }


    pub fn get_skills(&self) -> &[AgentSkill] {
        &self.skills
    }

    /// Check if the agent has a specific skill.
    /// THIS NEEDS TO BE REWORKED
    pub fn has_skill(&self, skill_name: &str) -> bool {
        println!("Checking if agent has skill: {}, out of skills : {:?}", skill_name,self.skills[0]);
        self.skills[0].id.contains(&skill_name.to_string())
    }

    /// Get the agent's ID.
    pub fn id(&self) -> &str {
        &self.id
    }
}
