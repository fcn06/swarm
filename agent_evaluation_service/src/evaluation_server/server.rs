use dashmap::DashMap;
use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
    http::StatusCode,
};
use tracing::info;
use std::sync::Arc;
use crate::evaluation_server::llm_judge::{AgentLogData, EvaluatedAgentData, evaluate_agent_output};

/// Application state holding evaluation data.
#[derive(Clone)]
pub struct AppState {
    pub db_evaluations: Arc<DashMap<String, Vec<EvaluatedAgentData>>>,
}

/// EvaluationServer for handling agent evaluation logs.
pub struct EvaluationServer {
    pub uri: String,
    pub app:Router,
}

impl EvaluationServer {
    pub async fn new(uri:String) -> anyhow::Result<Self> {
        let db_evaluations: DashMap<String, Vec<EvaluatedAgentData>> = DashMap::new();

        // Create AppState
        let app_state = AppState {
            db_evaluations: Arc::new(db_evaluations),
        };

        let app = Router::new()
            .route("/", get(root))
            .route("/log", post(log_evaluation))
            .route("/evaluations", get(list_evaluations))
            .with_state(app_state);

        Ok(Self {
            uri,
            app,
        })
    }

    /// Start the HTTP server.
    pub async fn start_http(&self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.uri.clone()).await?;
        println!("Evaluation Server started at {}", self.uri);
        axum::serve(listener, self.app.clone()).await?;
        Ok(())
    }
}

async fn root() -> &'static str {
    "Hello, Swarm Evaluation Service!"
}

async fn log_evaluation(
    State(state): State<AppState>,
    Json(log_data): Json<AgentLogData>,
) -> Result<Json<String>, (StatusCode, String)> {
    info!("Received log_evaluation request for agent: {}", log_data.agent_id);

    match evaluate_agent_output(log_data.clone()).await {
        Ok(evaluated_data) => {
            let db_evaluations = state.db_evaluations.clone();
            let mut entry = db_evaluations.entry(log_data.agent_id.clone()).or_insert_with(Vec::new);
            entry.push(evaluated_data);
            Ok(Json("Evaluation logged successfully".to_string()))
        }
        Err(e) => {
            let error_message = format!("Failed to evaluate agent output: {:?}", e);
            eprintln!("{}", error_message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}

async fn list_evaluations(
    State(state): State<AppState>,
) -> Json<DashMap<String, Vec<EvaluatedAgentData>>> {
    let db_evaluations = state.db_evaluations.clone();
    Json(db_evaluations.clone())
}