use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
    routing::{get, post},
};
use tracing::info;
use std::sync::Arc;
use a2a_rs::domain::AgentCard;

/// Application state holding configurations and in-memory data.
#[derive(Clone)]
pub struct AppState {
    /// In-memory database for registered agents. Key: agent_name, Value: AgentCard.
    pub db_agents: Arc<DashMap<String, AgentCard>>,
    /// In-memory index for agent skills. Key: skill, Value: Set of agent_names.
    pub skills_index: Arc<DashMap<String, HashSet<String>>>,
}

/// The discovery server, responsible for agent registration and search.
pub struct DiscoveryServer {
    pub uri: String,
    pub app: Router,
}

impl DiscoveryServer {
    pub async fn new(uri: String) -> anyhow::Result<Self> {
        // Initialize the in-memory stores
        let db_agents = DashMap::new();
        let skills_index = DashMap::new();

        // Create the application state
        let app_state = AppState {
            db_agents: Arc::new(db_agents),
            skills_index: Arc::new(skills_index),
        };

        // Configure the API routes
        let app = Router::new()
            .route("/", get(root))
            .route("/register", post(register_agent))
            .route("/deregister", post(deregister_agent))
            .route("/agents", get(list_agents))
            .route("/agents/search", get(search_agents_by_skill))
            .with_state(app_state);

        Ok(Self { uri, app })
    }

    /// Start the HTTP server.
    pub async fn start_http(&self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.uri).await?;
        info!("Discovery Server started at {}", self.uri);
        axum::serve(listener, self.app.clone()).await?;
        Ok(())
    }
}

/// Root endpoint for basic health checks.
async fn root() -> &'static str {
    "Hello, Swarm Discovery Service!"
}

/// Registers an agent and indexes its skills.
async fn register_agent(
    State(state): State<AppState>,
    Json(agent_card): Json<AgentCard>,
) -> impl IntoResponse {
    info!("Received register request for agent: {}", agent_card.name);
    let agent_name = agent_card.name.clone();
    let agent_skills=agent_card.skills.clone();

    // Index the agent's skills
    // Assumes `agent_card.skills` is a `Vec<String>`.
    // This part needs to be adjusted if the `skills` field in `AgentCard` is different.
    if let Some(skills) = Some(agent_skills.into_iter().map(|x | x.name)) {
        for skill in skills {
            state
                .skills_index
                .entry(skill.to_lowercase())
                .or_default()
                .insert(agent_name.clone());
        }
    }

    // Store the agent card
    state.db_agents.insert(agent_name, agent_card);

    (StatusCode::CREATED, "Agent registered successfully")
}

/// Deregisters an agent and removes it from the skills index.
async fn deregister_agent(
    State(state): State<AppState>,
    Json(agent_card): Json<AgentCard>,
) -> impl IntoResponse {
    info!("Received deregister request for agent: {}", agent_card.name);
    let agent_name = &agent_card.name;
    let agent_skills=agent_card.skills.clone();

    // Remove the agent from the skills index
    if let Some(skills) = Some(agent_skills.into_iter().map(|x | x.name)) {
        for skill in skills {
            let skill_key = skill.to_lowercase();
            if let Some(mut agents_with_skill) = state.skills_index.get_mut(&skill_key) {
                agents_with_skill.remove(agent_name);
                // Clean up the skill entry if no agents are left
                if agents_with_skill.is_empty() {
                    state.skills_index.remove(&skill_key);
                }
            }
        }
    }

    // Remove the agent from the main database
    state.db_agents.remove(agent_name);

    (StatusCode::OK, "Agent deregistered successfully")
}


/// Lists all currently registered agents.
async fn list_agents(State(state): State<AppState>) -> Json<Vec<AgentCard>> {
    let list_agents: Vec<AgentCard> = state.db_agents.iter().map(|e| e.value().clone()).collect();
    Json(list_agents)
}

// todo: Fix the bug. This search does not work properly
/// Searches for agents possessing a specific skill.
/// The skill is provided as a query parameter, e.g., /agents/search?skill=math
async fn search_agents_by_skill(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<AgentCard>>, StatusCode> {
    // Get the skill from the query parameters
    let skill = match params.get("skill") {
        Some(s) => s.to_lowercase(),
        None => {
            info!("Search request failed: 'skill' query parameter is missing");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    info!("Received search request for skill: {}", skill);

    let mut found_agents = Vec::new();
    // Look up the skill in the index
    if let Some(agent_names) = state.skills_index.get(&skill) {
        // Retrieve the full AgentCard for each matching agent name
        for name in agent_names.iter() {
            if let Some(agent_card) = state.db_agents.get(name) {
                found_agents.push(agent_card.value().clone());
            }
        }
    }

    info!("Found {} agents with skill '{}'", found_agents.len(), skill);
    Ok(Json(found_agents))
}
