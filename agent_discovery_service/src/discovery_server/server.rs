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

//use crate::model::models::{AgentDefinition, TaskDefinition, ToolDefinition};
use agent_models::registry::registry_models::{AgentDefinition, TaskDefinition, ToolDefinition};

// This is a sample and simple implementation

// todo: we should add a notion of context, that would be used to segment resources for different context

/// Application state holding configurations and in-memory data.
#[derive(Clone)]
pub struct AppState {
    /// In-memory database for registered agents. Key: agent_id, Value: AgentDefinition.
    pub db_agents: Arc<DashMap<String, AgentDefinition>>,
    /// In-memory index for agent skills. Key: skill_name, Value: Set of agent_ids.
    pub skills_index: Arc<DashMap<String, HashSet<String>>>,
    /// In-memory database for registered tasks. Key: task_id, Value: TaskDefinition.
    pub db_tasks: Arc<DashMap<String, TaskDefinition>>,
    /// In-memory database for registered tools. Key: tool_id, Value: ToolDefinition.
    pub db_tools: Arc<DashMap<String, ToolDefinition>>,
}

/// The discovery server, responsible for agent, task, and tool registration and search.
pub struct DiscoveryServer {
    pub uri: String,
    pub app: Router,
}

impl DiscoveryServer {
    pub async fn new(uri: String) -> anyhow::Result<Self> {
        // Initialize the in-memory stores
        let db_agents = DashMap::new();
        let skills_index = DashMap::new();
        let db_tasks = DashMap::new();
        let db_tools = DashMap::new();

        // Create the application state
        let app_state = AppState {
            db_agents: Arc::new(db_agents),
            skills_index: Arc::new(skills_index),
            db_tasks: Arc::new(db_tasks),
            db_tools: Arc::new(db_tools),
        };

        // Configure the API routes
        let app = Router::new()
            .route("/", get(root))
            // Agent Definition Routes
            .route("/agents/register", post(register_agent_definition))
            .route("/agents/deregister", post(deregister_agent_definition))
            .route("/agents", get(list_agent_definitions))
            .route("/agents/search", get(search_agents_by_skill))
            // Task Definition Routes
            .route("/tasks/register", post(register_task_definition))
            .route("/tasks", get(list_task_definitions))
            // Tool Definition Routes
            .route("/tools/register", post(register_tool_definition))
            .route("/tools", get(list_tool_definitions))
            // All resources
            .route("/resources", get(list_available_resources))
            .with_state(app_state);

        Ok(Self { uri, app })
    }

    /// Start the HTTP server.
    pub async fn start_http(&self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.uri).await?;
        println!("Discovery Server started on {}", self.uri);
        axum::serve(listener, self.app.clone()).await?;
        Ok(())
    }
}

/// Root endpoint for basic health checks.
async fn root() -> &'static str {
    "Hello, Swarm Discovery Service!"
}

/// Registers an AgentDefinition and indexes its skills.
async fn register_agent_definition(
    State(state): State<AppState>,
    Json(agent_def): Json<AgentDefinition>,
) -> impl IntoResponse {
    info!("Received register request for agent: {}", agent_def.name);
    let agent_id = agent_def.id.clone();
    let agent_skills = agent_def.skills.clone();

    // Index the agent's skills
    for skill in agent_skills {
        state
            .skills_index
            .entry(skill.name.to_lowercase())
            .or_default()
            .insert(agent_id.clone());
    }

    // Store the agent definition
    state.db_agents.insert(agent_id, agent_def);

    (StatusCode::CREATED, "Agent registered successfully")
}

/// Deregisters an AgentDefinition and removes it from the skills index.
async fn deregister_agent_definition(
    State(state): State<AppState>,
    Json(agent_def): Json<AgentDefinition>,
) -> impl IntoResponse {
    info!("Received deregister request for agent: {}", agent_def.name);
    let agent_id = &agent_def.id;
    let agent_skills = agent_def.skills.clone();

    // Remove the agent from the skills index
    for skill in agent_skills {
        let skill_key = skill.name.to_lowercase();
        if let Some(mut agents_with_skill) = state.skills_index.get_mut(&skill_key) {
            agents_with_skill.remove(agent_id);
            // Clean up the skill entry if no agents are left
            if agents_with_skill.is_empty() {
                state.skills_index.remove(&skill_key);
            }
        }
    }

    // Remove the agent from the main database
    state.db_agents.remove(agent_id);

    (StatusCode::OK, "Agent deregistered successfully")
}

/// Lists all currently registered AgentDefinitions.
async fn list_agent_definitions(State(state): State<AppState>) -> Json<Vec<AgentDefinition>> {
    let list_agents: Vec<AgentDefinition> = state.db_agents.iter().map(|e| e.value().clone()).collect();
    Json(list_agents)
}

/// Searches for agents possessing a specific skill.
/// The skill is provided as a query parameter, e.g., /agents/search?skill=math
async fn search_agents_by_skill(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<AgentDefinition>>, StatusCode> {
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
    if let Some(agent_ids) = state.skills_index.get(&skill) {
        // Retrieve the full AgentDefinition for each matching agent ID
        for id in agent_ids.iter() {
            if let Some(agent_def) = state.db_agents.get(id) {
                found_agents.push(agent_def.value().clone());
            }
        }
    }

    info!("Found {} agents with skill '{}'", found_agents.len(), skill);
    Ok(Json(found_agents))
}

/// Registers a TaskDefinition.
async fn register_task_definition(
    State(state): State<AppState>,
    Json(task_def): Json<TaskDefinition>,
) -> impl IntoResponse {
    info!("Received register request for task: {}", task_def.name);
    state.db_tasks.insert(task_def.id.clone(), task_def);
    (StatusCode::CREATED, "Task registered successfully")
}

/// Lists all currently registered TaskDefinitions.
async fn list_task_definitions(State(state): State<AppState>) -> Json<Vec<TaskDefinition>> {
    let list_tasks: Vec<TaskDefinition> = state.db_tasks.iter().map(|e| e.value().clone()).collect();
    Json(list_tasks)
}

/// Registers a ToolDefinition.
async fn register_tool_definition(
    State(state): State<AppState>,
    Json(tool_def): Json<ToolDefinition>,
) -> impl IntoResponse {
    info!("Received register request for tool: {}", tool_def.name);
    state.db_tools.insert(tool_def.id.clone(), tool_def);
    (StatusCode::CREATED, "Tool registered successfully")
}

/// Lists all currently registered ToolDefinitions.
async fn list_tool_definitions(State(state): State<AppState>) -> Json<Vec<ToolDefinition>> {
    let list_tools: Vec<ToolDefinition> = state.db_tools.iter().map(|e| e.value().clone()).collect();
    Json(list_tools)
}

async fn list_available_resources(State(state): State<AppState>) -> impl IntoResponse {
    let mut available_resources = String::new();

    let list_agents: Vec<AgentDefinition> = state.db_agents.iter().map(|e| e.value().clone()).collect();
    let list_tools: Vec<ToolDefinition> = state.db_tools.iter().map(|e| e.value().clone()).collect();
    let list_tasks: Vec<TaskDefinition> = state.db_tasks.iter().map(|e| e.value().clone()).collect();

    if !list_tools.is_empty() {
        let tool_details = list_tools.iter()
            .map(|tool| format!("* tool_id : {} -- description : {} -- arguments : {}", tool.id, tool.description,tool.input_schema))
            .collect::<Vec<String>>()
            .join("\n");
        available_resources.push_str(&tool_details);
        available_resources.push('\n');
    }

    if !list_tasks.is_empty() {
        let task_details = list_tasks.iter()
            .map(|task| format!("* task_id :  {} -- description : {}", task.id, task.description))
            .collect::<Vec<String>>()
            .join("\n");
        available_resources.push_str(&task_details);
        available_resources.push('\n');
    }

    if !list_agents.is_empty() {
        let agent_details = list_agents.iter()
            .map(|agent| format!("* agent_id : {} -- description :{} -- ", agent.id, agent.description))
            .collect::<Vec<String>>()
            .join("\n");
        available_resources.push_str(&agent_details);
        available_resources.push('\n');
    }

    (StatusCode::OK, Json(available_resources))
}