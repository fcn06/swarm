use dashmap::DashMap;



use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
};

use tracing::info;
use std::sync::Arc;
use a2a_rs::domain::AgentCard;

/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub db_agents: Arc<DashMap<String, AgentCard>>,
}

/// Memory_server 
pub struct DiscoveryServer {
    pub uri: String,
    pub app:Router,
}

impl DiscoveryServer {

    pub async fn new(uri:String) -> anyhow::Result<Self> {
        
        let db_agents:DashMap<String, AgentCard> = DashMap::new();

        // Create AppState
        let app_state = AppState {
            db_agents:Arc::new(db_agents),
        };

        let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register_agent))
        .route("/agents", get(list_agents))
        .with_state(app_state);

        Ok(Self {
            uri,
            app,
        })

    }

    /// Start the HTTP server
    pub async fn start_http(&self) -> anyhow::Result<()> {

            // Run our app with hyper
            let listener = tokio::net::TcpListener::bind(self.uri.clone()).await?;
            println!("Server started");
            axum::serve(listener, self.app.clone()).await?;

            Ok(())

    }

}

async fn root() -> &'static str {
    "Hello, Swarm Discovery Service!"
}

async fn register_agent(
    State(state): State<AppState>, // Extract the AppState
    Json(agent_card): Json<AgentCard>,
) -> Json<String> {

    let db_agents = state.db_agents.to_owned();
    info!("Received register_agent request: {:?}", agent_card);
    db_agents.insert(agent_card.name.clone(), agent_card);
    Json("Agent registered successfully".to_string())
}

async fn list_agents(
    State(state): State<AppState>, // Extract the AppState
) -> Json<Vec<AgentCard>> {

    let db_agents = state.db_agents.to_owned();

    let mut list_agents: Vec<AgentCard> = Vec::new();

    for entry in db_agents.iter() {
        list_agents.push(entry.value().clone());
    }

    Json(list_agents)

}


