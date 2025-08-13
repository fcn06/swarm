use dashmap::DashMap;

use crate::models::{LogEntry, LogPayload};

use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tracing::info;

/// Application state holding configurations
#[derive(Clone)] // AppState needs to be Clone to be used as Axum state
pub struct AppState {
    pub db_memory: Arc<DashMap<String, Vec<LogEntry>>>,
}

/// Memory_server
pub struct MemoryServer {
    pub uri: String,
    pub app: Router,
}

impl MemoryServer {
    pub async fn new(uri: String) -> anyhow::Result<Self> {
        let db_memory: DashMap<String, Vec<LogEntry>> = DashMap::new();

        // Create AppState
        let app_state = AppState {
            db_memory: Arc::new(db_memory),
        };

        let app = Router::new()
            .route("/", get(root))
            .route("/log", post(log_message))
            .route("/conversation/:conversation_id", get(get_conversation))
            .with_state(app_state);

        Ok(Self { uri, app })
    }

    /// Start the HTTP server
    pub async fn start_http(&self) -> anyhow::Result<()> {
        // Run our app with hyper
        let listener = tokio::net::TcpListener::bind(self.uri.clone()).await?;
        println!("Server started on {}", self.uri);
        axum::serve(listener, self.app.clone()).await?;

        Ok(())
    }
}

async fn root() -> &'static str {
    "Hello, Swarm Memory Service!"
}

async fn log_message(
    State(state): State<AppState>, // Extract the AppState
    Json(payload): Json<LogPayload>,
) -> Json<Vec<LogEntry>> {
    info!("Received log_message request: {:?}", payload);

    let db_memory = state.db_memory.to_owned();

    let new_entry = LogEntry {
        role: payload.role,
        content: payload.content,
        agent_id: payload.agent_id,
    };

    let mut conversation = db_memory
        .entry(payload.conversation_id)
        .or_insert_with(Vec::new);
    conversation.push(new_entry);

    Json(conversation.clone())
}

async fn get_conversation(
    State(state): State<AppState>, // Extract the AppState
    Path(conversation_id): Path<String>,
) -> Json<Option<Vec<LogEntry>>> {
    info!("Received get_conversation request for id: {}", conversation_id);

    let db_memory = state.db_memory.to_owned();

    let conversation = db_memory.get(&conversation_id).map(|entry| entry.clone());

    Json(conversation)
}
