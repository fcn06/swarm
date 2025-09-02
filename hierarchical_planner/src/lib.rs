use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agent_core::graph::graph_definition::{Activity, ActivityType, AgentConfigInput, ToolConfigInput, TaskConfigInput, Dependency, WorkflowPlanInput};

// --- Traits for Abstraction ---

/// Trait for an LLM client that can generate high-level plans.
#[async_trait]
pub trait LLMClient {
    async fn generate_high_level_plan(&self, user_request: &str) -> Result<Vec<String>, String>;
}

/// Trait for an embedding model that can convert text to vectors.
#[async_trait]
pub trait EmbeddingModel {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String>;
}

// --- Data Structures ---

/// A simplified representation of an Activity, used for high-level planning suggestions.
/// This contains just enough information to identify the suggested resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleActivity {
    pub id: String,
    pub description: String,
    pub activity_type: ActivityType,
    pub r#type: Option<String>,
    pub skill_to_use: Option<String>,
    pub tool_to_use: Option<String>,
    pub tasks_to_use: Option<Vec<TaskConfigInput>>, // Renamed from 'tasks' to 'tasks_to_use' to avoid conflict
}

/// Represents an abstract step in the high-level plan, with a suggested concrete activity.
#[derive(Debug, Clone)]
pub struct HighLevelStep {
    pub step_number: usize,
    pub description: String, // The abstract description from the LLM
    pub suggested_activity: Option<SimpleActivity>, // The simplified concrete Activity suggested by semantic search
    pub suggestion_similarity_score: Option<f32>,
}

/// Represents the overall hierarchical plan, composed of high-level steps.
#[derive(Debug, Clone)]
pub struct HierarchicalPlan {
    pub user_request: String,
    pub steps: Vec<HighLevelStep>,
}

// --- Concrete Implementations (Simulated/Basic) ---

/// A simulated LLM client for demonstration purposes.
pub struct SimulatedLLMClient;

#[async_trait]
impl LLMClient for SimulatedLLMClient {
    async fn generate_high_level_plan(&self, user_request: &str) -> Result<Vec<String>, String> {
        println!("Simulating LLM call for request: '{}'", user_request);
        if user_request.contains("customer feedback") {
            Ok(vec![
                "Fetch customer feedback data.".to_string(),
                "Analyze feedback for sentiment and key themes.".to_string(),
                "Generate a summary report.".to_string(),
                "Send the report to the marketing team.".to_string(),
            ])
        } else if user_request.contains("user data") && user_request.contains("anomalies") {
             Ok(vec![
                "Fetch user data.".to_string(),
                "Analyze data for anomalies.".to_string(),
                "Generate a report on anomalies.".to_string(),
                "Notify relevant team about anomalies.".to_string(),
            ])
        }
        else if user_request.contains("flight") && user_request.contains("book") {
            Ok(vec![
                "Search for available flights.".to_string(),
                "Select the best flight option.".to_string(),
                "Book the chosen flight.".to_string(),
                "Confirm booking with user.".to_string(),
            ])
        }
        else {
            Ok(vec![format!("Perform high-level task: {}", user_request)])
        }
    }
}

/// A simulated embedding model for demonstration purposes.
pub struct SimulatedEmbeddingModel;

#[async_trait]
impl EmbeddingModel for SimulatedEmbeddingModel {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let words: Vec<String> = text.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut embedding_map = HashMap::new();
        for word in words {
            *embedding_map.entry(word).or_insert(0) += 1;
        }

        let mut sorted_keys: Vec<&String> = embedding_map.keys().collect();
        sorted_keys.sort();

        let embedding: Vec<f32> = sorted_keys.into_iter()
            .map(|key| *embedding_map.get(key).unwrap() as f32)
            .collect();

        Ok(embedding)
    }
}

/// A simple in-memory vector store for activities.
pub struct ActivityVectorStore {
    activities: Vec<Activity>, // Still store full Activities for their detailed descriptions
    activity_embeddings: HashMap<String, Vec<f32>>, // Activity ID -> embedding
}

impl ActivityVectorStore {
    pub async fn new(
        activities: Vec<Activity>,
        embedding_model: &dyn EmbeddingModel,
    ) -> Result<Self, String> {
        let mut activity_embeddings = HashMap::new();
        for activity in &activities {
            let embedding = embedding_model.generate_embedding(&activity.description).await?;
            activity_embeddings.insert(activity.id.clone(), embedding);
        }
        Ok(Self {
            activities,
            activity_embeddings,
        })
    }

    /// Finds the most semantically similar activity for a given plan step description
    /// and returns a SimpleActivity representation.
    pub async fn find_most_similar_activity(
        &self,
        step_description: &str,
        embedding_model: &dyn EmbeddingModel,
    ) -> Result<Option<(SimpleActivity, f32)>, String> {
        let step_embedding = embedding_model.generate_embedding(step_description).await?;

        if step_embedding.is_empty() {
            return Ok(None);
        }

        let mut best_match: Option<(&Activity, f32)> = None;
        let mut max_similarity = -1.0;

        for activity in &self.activities {
            if let Some(activity_embedding) = self.activity_embeddings.get(&activity.id) {
                if activity_embedding.is_empty() {
                    continue;
                }
                let similarity = cosine_similarity(&step_embedding, activity_embedding);
                if similarity > max_similarity {
                    max_similarity = similarity;
                    best_match = Some((activity, similarity));
                }
            }
        }

        Ok(best_match.map(|(activity, similarity)| {
            // Convert the full Activity to a SimpleActivity for the output
            let simple_activity = SimpleActivity {
                id: activity.id.clone(),
                description: activity.description.clone(),
                activity_type: activity.activity_type.clone(),
                r#type: activity.r#type.clone(),
                skill_to_use: activity.skill_to_use.clone(),
                tool_to_use: activity.tool_to_use.clone(),
                tasks_to_use: activity.tasks.clone(),
            };
            (simple_activity, similarity)
        }))
    }
}

/// Calculates the cosine similarity between two vectors.
fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(&a, &b)| a * b).sum();
    let magnitude1: f32 = vec1.iter().map(|&a| a * a).sum::<f32>().sqrt();
    let magnitude2: f32 = vec2.iter().map(|&a| a * a).sum::<f32>().sqrt();

    if magnitude1 == 0.0 || magnitude2 == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude1 * magnitude2)
}

// --- Hierarchical Planner Core Logic ---

pub struct HierarchicalPlanner {
    llm_client: Box<dyn LLMClient + Send + Sync>,
    embedding_model: Box<dyn EmbeddingModel + Send + Sync>,
    activity_store: ActivityVectorStore,
}

impl HierarchicalPlanner {
    pub async fn new(
        llm_client: Box<dyn LLMClient + Send + Sync>,
        embedding_model: Box<dyn EmbeddingModel + Send + Sync>,
        available_activities: Vec<Activity>,
    ) -> Result<Self, String> {
        let activity_store = ActivityVectorStore::new(available_activities, embedding_model.as_ref()).await?;
        Ok(Self {
            llm_client,
            embedding_model,
            activity_store,
        })
    }

    /// Generates a hierarchical plan: first high-level steps, then suggests concrete activities.
    pub async fn generate_plan(&self, user_request: &str) -> Result<HierarchicalPlan, String> {
        // Step 1: High-Level Plan Generation
        let high_level_descriptions = self.llm_client.generate_high_level_plan(user_request).await?;

        // Step 2: Suggest concrete activities for each high-level step
        let mut high_level_steps: Vec<HighLevelStep> = Vec::new();

        for (i, description) in high_level_descriptions.into_iter().enumerate() {
            let step_number = i + 1;

            println!("Attempting to find activity for step {}: '{}'", step_number, description);
            let (suggested_activity, suggestion_similarity_score) = match self.activity_store.find_most_similar_activity(&description, self.embedding_model.as_ref()).await {
                Ok(Some((simple_activity, similarity))) => {
                    println!("  -> Suggested Activity: {} (Similarity: {:.2})", simple_activity.id, similarity);
                    (Some(simple_activity), Some(similarity))
                }
                Ok(None) => {
                    println!("  -> No suitable activity found for step {}", step_number);
                    (None, None)
                }
                Err(e) => {
                    eprintln!("Error during activity suggestion for step {}: {}", step_number, e);
                    return Err(format!("Failed to suggest activity for step {}: {}", step_number, e));
                }
            };

            high_level_steps.push(HighLevelStep {
                step_number,
                description,
                suggested_activity,
                suggestion_similarity_score,
            });
        }

        Ok(HierarchicalPlan {
            user_request: user_request.to_string(),
            steps: high_level_steps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // A mock LLMClient for testing purposes
    pub struct MockLLMClient {
        plans: HashMap<String, Vec<String>>,
    }

    impl MockLLMClient {
        pub fn new() -> Self {
            let mut plans = HashMap::new();
            plans.insert(
                "Analyze customer feedback and generate a summary report, then notify the marketing team.".to_string(),
                vec![
                    "Fetch customer feedback data.".to_string(),
                    "Analyze feedback for sentiment and key themes.".to_string(),
                    "Generate a summary report.".to_string(),
                    "Send the report to the marketing team.".to_string(),
                ],
            );
            plans.insert(
                "Process new user registrations and send a welcome email.".to_string(),
                vec![
                    "Receive new user registration event.".to_string(),
                    "Store user details in database.".to_string(),
                    "Generate welcome email content.".to_string(),
                    "Send welcome email to user.".to_string(),
                ],
            );
             plans.insert(
                "Book a flight from London to New York for tomorrow.".to_string(),
                vec![
                    "Search for available flights.".to_string(),
                    "Select the best flight option.".to_string(),
                    "Book the chosen flight.".to_string(),
                    "Confirm booking with user.".to_string(),
                ],
            );
            Self { plans }
        }
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate_high_level_plan(&self, user_request: &str) -> Result<Vec<String>, String> {
            self.plans.get(user_request)
                .cloned()
                .ok_or_else(|| "No mock plan for this request".to_string())
        }
    }

    // A mock EmbeddingModel for testing purposes
    pub struct MockEmbeddingModel;

    #[async_trait]
    impl EmbeddingModel for MockEmbeddingModel {
        async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
            let mut embedding = vec![0.0; 26]; // For a-z
            for char_code in text.to_lowercase().chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c as u32) {
                if char_code >= 97 && char_code <= 122 {
                    embedding[(char_code - 97) as usize] += 1.0;
                }
            }
            Ok(embedding)
        }
    }

    // Helper function to create a basic Activity for testing
    fn create_activity(
        id: &str,
        description: &str,
        activity_type: ActivityType,
        r#type: &str,
        skill_to_use: Option<&str>,
        tool_to_use: Option<&str>,
        tool_parameters: Option<serde_json::Value>,
        tasks: Option<Vec<TaskConfigInput>>,
    ) -> Activity {
        Activity {
            activity_type,
            id: id.to_string(),
            description: description.to_string(),
            r#type: Some(r#type.to_string()),
            skill_to_use: skill_to_use.map(|s| s.to_string()),
            assigned_agent_id_preference: None,
            agent_context: None,
            tool_to_use: tool_to_use.map(|s| s.to_string()),
            tool_parameters,
            tasks,
            dependencies: Vec::new(),
            expected_outcome: Some(format!("Outcome of {}", id)),
            activity_output: None,
        }
    }

    #[tokio::test]
    async fn test_hierarchical_planner_generates_high_level_plan_with_simple_suggestions() {
        let llm_client = Box::new(MockLLMClient::new());
        let embedding_model = Box::new(MockEmbeddingModel);

        let available_activities = vec![
            create_activity("DataFetcher", "Fetches data from various data sources.", ActivityType::DirectToolUse, "data_tool", None, Some("DataFetcherTool"), None, None),
            create_activity("SentimentAnalyzer", "Analyzes text for sentiment and extracts key entities.", ActivityType::DirectToolUse, "nlp_tool", None, Some("SentimentAnalyzerTool"), None, None),
            create_activity("ReportGenerator", "Compiles data into a structured report.", ActivityType::DirectToolUse, "reporting_tool", None, Some("ReportGeneratorTool"), None, None),
            create_activity("EmailSenderAgent", "Sends emails to specified recipients.", ActivityType::DelegationAgent, "communication_agent", Some("email_sending_skill"), None, None, None),
            create_activity("DatabaseWriter", "Writes or stores data into a database.", ActivityType::DirectToolUse, "db_tool", None, Some("DatabaseWriterTool"), None, None),
            create_activity("FlightSearchTool", "Searches for flights based on origin, destination, and date.", ActivityType::DirectToolUse, "flight_search", None, Some("FlightSearchTool"), Some(json!({"origin": "string", "destination": "string", "date": "string"})), None),
            create_activity("FlightBookingTool", "Books a flight given flight details.", ActivityType::DirectToolUse, "flight_booking", None, Some("FlightBookingTool"), Some(json!({"flight_id": "string", "passenger_details": "object"})), None),
            create_activity("BookingConfirmationAgent", "Confirms booking details with the user and handles notifications.", ActivityType::DelegationAgent, "confirmation_agent", Some("booking_confirmation_skill"), None, None, None),
        ];

        let planner = HierarchicalPlanner::new(llm_client, embedding_model, available_activities)
            .await
            .expect("Failed to create planner");

        let user_request = "Analyze customer feedback and generate a summary report, then notify the marketing team.";
        let high_level_plan = planner.generate_plan(user_request).await.expect("Failed to generate high-level plan");

        assert_eq!(high_level_plan.user_request, user_request);
        assert_eq!(high_level_plan.steps.len(), 4);

        // Verify first step
        let step1 = &high_level_plan.steps[0];
        assert_eq!(step1.description, "Fetch customer feedback data.");
        assert!(step1.suggested_activity.is_some());
        assert_eq!(step1.suggested_activity.as_ref().unwrap().id, "DataFetcher");
        assert_eq!(step1.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DirectToolUse);
        assert!(step1.suggested_activity.as_ref().unwrap().tool_to_use.is_some());
        assert_eq!(step1.suggested_activity.as_ref().unwrap().tool_to_use.as_ref().unwrap(), "DataFetcherTool");
        assert!(step1.suggestion_similarity_score.is_some());

        // Verify second step
        let step2 = &high_level_plan.steps[1];
        assert_eq!(step2.description, "Analyze feedback for sentiment and key themes.");
        assert!(step2.suggested_activity.is_some());
        assert_eq!(step2.suggested_activity.as_ref().unwrap().id, "SentimentAnalyzer");
        assert_eq!(step2.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DirectToolUse);
        assert!(step2.suggested_activity.as_ref().unwrap().tool_to_use.is_some());
        assert_eq!(step2.suggested_activity.as_ref().unwrap().tool_to_use.as_ref().unwrap(), "SentimentAnalyzerTool");
        assert!(step2.suggestion_similarity_score.is_some());

        // Verify third step
        let step3 = &high_level_plan.steps[2];
        assert_eq!(step3.description, "Generate a summary report.");
        assert!(step3.suggested_activity.is_some());
        assert_eq!(step3.suggested_activity.as_ref().unwrap().id, "ReportGenerator");
        assert_eq!(step3.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DirectToolUse);
        assert!(step3.suggested_activity.as_ref().unwrap().tool_to_use.is_some());
        assert_eq!(step3.suggested_activity.as_ref().unwrap().tool_to_use.as_ref().unwrap(), "ReportGeneratorTool");
        assert!(step3.suggestion_similarity_score.is_some());

        // Verify fourth step (Agent type)
        let step4 = &high_level_plan.steps[3];
        assert_eq!(step4.description, "Send the report to the marketing team.");
        assert!(step4.suggested_activity.is_some());
        assert_eq!(step4.suggested_activity.as_ref().unwrap().id, "EmailSenderAgent");
        assert_eq!(step4.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DelegationAgent);
        assert!(step4.suggested_activity.as_ref().unwrap().skill_to_use.is_some());
        assert_eq!(step4.suggested_activity.as_ref().unwrap().skill_to_use.as_ref().unwrap(), "email_sending_skill");
        assert!(step4.suggestion_similarity_score.is_some());
    }

    #[tokio::test]
    async fn test_hierarchical_planner_handles_no_matching_simple_activity() {
        let llm_client = Box::new(MockLLMClient::new());
        let embedding_model = Box::new(MockEmbeddingModel);

        let available_activities = vec![
            create_activity("EmailSenderAgent", "Sends emails.", ActivityType::DelegationAgent, "communication_agent", Some("email_sending_skill"), None, None, None),
        ];

        let planner = HierarchicalPlanner::new(llm_client, embedding_model, available_activities)
            .await
            .expect("Failed to create planner");

        let user_request = "Process new user registrations and send a welcome email.";
        let high_level_plan = planner.generate_plan(user_request).await.expect("Failed to generate high-level plan");

        assert_eq!(high_level_plan.user_request, user_request);
        assert_eq!(high_level_plan.steps.len(), 4);

        // Expect no suggested activity for steps that don't match EmailSenderAgent
        assert!(high_level_plan.steps[0].suggested_activity.is_none()); // Receive new user registration event.
        assert!(high_level_plan.steps[1].suggested_activity.is_none()); // Store user details in database.
        assert!(high_level_plan.steps[2].suggested_activity.is_none()); // Generate welcome email content.

        // Expect EmailSenderAgent for the last step
        let step4 = &high_level_plan.steps[3];
        assert_eq!(step4.description, "Send welcome email to user.");
        assert!(step4.suggested_activity.is_some());
        assert_eq!(step4.suggested_activity.as_ref().unwrap().id, "EmailSenderAgent");
    }

     #[tokio::test]
    async fn test_hierarchical_planner_with_flight_booking_and_simple_activities() {
        let llm_client = Box::new(MockLLMClient::new());
        let embedding_model = Box::new(MockEmbeddingModel);

        let available_activities = vec![
            create_activity("FlightSearchTool", "Searches for flights based on origin, destination, and date.", ActivityType::DirectToolUse, "flight_search", None, Some("FlightSearchTool"), Some(json!({"origin": "string", "destination": "string", "date": "string"})), None),
            create_activity("FlightBookingTool", "Books a flight given flight details.", ActivityType::DirectToolUse, "flight_booking", None, Some("FlightBookingTool"), Some(json!({"flight_id": "string", "passenger_details": "object"})), None),
            create_activity("BookingConfirmationAgent", "Confirms booking details with the user and handles notifications.", ActivityType::DelegationAgent, "confirmation_agent", Some("booking_confirmation_skill"), None, None, None),
        ];

        let planner = HierarchicalPlanner::new(llm_client, embedding_model, available_activities)
            .await
            .expect("Failed to create planner");

        let user_request = "Book a flight from London to New York for tomorrow.";
        let high_level_plan = planner.generate_plan(user_request).await.expect("Failed to generate high-level plan");

        assert_eq!(high_level_plan.steps.len(), 4);

        let step1 = &high_level_plan.steps[0];
        assert_eq!(step1.description, "Search for available flights.");
        assert!(step1.suggested_activity.is_some());
        assert_eq!(step1.suggested_activity.as_ref().unwrap().id, "FlightSearchTool");
        assert_eq!(step1.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DirectToolUse);

        let step2 = &high_level_plan.steps[1];
        assert_eq!(step2.description, "Select the best flight option.");
        assert!(step2.suggested_activity.is_none()); // This step might not find a perfect activity in this simple mock

        let step3 = &high_level_plan.steps[2];
        assert_eq!(step3.description, "Book the chosen flight.");
        assert!(step3.suggested_activity.is_some());
        assert_eq!(step3.suggested_activity.as_ref().unwrap().id, "FlightBookingTool");
        assert_eq!(step3.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DirectToolUse);

        let step4 = &high_level_plan.steps[3];
        assert_eq!(step4.description, "Confirm booking with user.");
        assert!(step4.suggested_activity.is_some());
        assert_eq!(step4.suggested_activity.as_ref().unwrap().id, "BookingConfirmationAgent");
        assert_eq!(step4.suggested_activity.as_ref().unwrap().activity_type, ActivityType::DelegationAgent);
    }
}
