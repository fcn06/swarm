use dashmap::DashMap;

use crate::{MemoryEntry, AgentData};
use serde::{Serialize,Deserialize};
use chrono::Utc;
use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
};

use tracing::info;
use std::sync::Arc;

/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub db_memory: Arc<DashMap<String, MemoryEntry>>,
}

/// Memory_server 
pub struct MemoryServer {
    pub uri: String,
    pub app:Router,
}

impl MemoryServer {

    pub async fn new(uri:String) -> anyhow::Result<Self> {
        
        let db_memory:DashMap<String, MemoryEntry> = DashMap::new();

        // Create AppState
        let app_state = AppState {
            db_memory:Arc::new(db_memory),
        };

        let app = Router::new()
        .route("/", get(root))
        .route("/memory", post(add_memory).get(list_memories))
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
    "Hello, Swarm Memory Service!"
}

async fn add_memory(
    State(state): State<AppState>, // Extract the AppState
    Json(payload_agent_data): Json<AgentData>,
) -> Json<MemoryEntry> {

    info!("Received add_memory request: {:?}", payload_agent_data);

    let db_memory = state.db_memory.to_owned();
   
    let memory_entry_updated=MemoryEntry{
        id:uuid::Uuid::new_v4().to_string(),
        agent_id:payload_agent_data.agent_id.clone(),
        timestamp:Utc::now().to_rfc3339(),
        content:payload_agent_data.content.clone(),
        summary:None,
        interaction_type:payload_agent_data.interaction_type.clone(),
    };

    db_memory.insert(memory_entry_updated.id.clone(), memory_entry_updated.clone());

    Json(memory_entry_updated)
   
}

async fn list_memories(
    State(state): State<AppState>, // Extract the AppState
) -> Json<Vec<MemoryEntry>> {

    info!("Received list_memories request");

    let db_memory = state.db_memory.to_owned();

    let mut memories: Vec<MemoryEntry> = Vec::new();

    for entry in db_memory.iter() {
        memories.push(entry.value().clone());
    }

    Json(memories)

}
