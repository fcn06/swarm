use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
};

use dashmap::DashMap;

use tracing::{info, error, Level};
use tracing_subscriber::{
    prelude::*,
    fmt,
    layer::Layer,
    Registry, filter
};

use clap::Parser;

use serde::{Serialize,Deserialize};
use chrono::Utc;


// create a MemoryService struct, along with dedicated methods
// create a light bin file that will launch server through server.start()


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentData {
    pub agent_id: String,
    pub content: String,
    pub summary: Option<String>,
    pub interaction_type: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub agent_id: String,
    pub timestamp: String,
    pub content: String,
    pub summary: Option<String>,
    pub interaction_type: String,
}


/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub db_memory: Arc<DashMap<String, MemoryEntry>>,
}

/// Command-line arguments for the reimbursement server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "warn")]
    log_level: String,
    #[clap(long, default_value = "0.0.0.0:5000")]
    uri: String,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Parse command-line arguments
    let args = Args::parse();

    /************************************************/
    /* Setting proper log level. Default is INFO    */
    /************************************************/ 
    let log_level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::WARN,
    };

    let subscriber = Registry::default()
    .with(
        // stdout layer, to view everything in the console
        fmt::layer()
            .compact()
            .with_ansi(true)
            .with_filter(filter::LevelFilter::from_level(log_level))
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    /************************************************/
    /* End of Setting proper log level              */
    /************************************************/ 

    /************************************************/
    /* Memory Data Storage                          */
    /************************************************/ 
    let db_memory:DashMap<String, MemoryEntry> = DashMap::new();

    /************************************************/
    /* End declaration Memory Data Storage          */
    /************************************************/ 

    /************************************************/
    /* App State Definition                         */
    /************************************************/ 
       // Create AppState
       let app_state = AppState {
        db_memory:Arc::new(db_memory),
    };
    /************************************************/
    /* End App State Definition                     */
    /************************************************/ 


    /************************************************/
    /* Server Definition                            */
    /************************************************/ 

    // Build our application with a route
        let app = Router::new()
        .route("/", get(root))
        .route("/memory", post(add_memory).get(list_memories))
        .with_state(app_state); 

    // Run our app with hyper
        let listener = tokio::net::TcpListener::bind(args.uri).await?;
        println!("Server started");
        axum::serve(listener, app).await?;
        Ok(())

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

