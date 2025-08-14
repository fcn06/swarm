use dashmap::DashMap;
use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
    http::StatusCode,
};
use tracing::{info,trace};
use std::sync::Arc;
use crate::evaluation_server::judge_agent::{AgentLogData, EvaluatedAgentData};
use crate::evaluation_server::judge_agent::JudgeAgent;

use configuration::AgentConfig;

/// Application state holding evaluation data.
#[derive(Clone)]
pub struct AppState {
    // Storing a vector of evaluations for each agent
    pub db_evaluations: Arc<DashMap<String, Vec<EvaluatedAgentData>>>,
    pub judge_agent: Arc<JudgeAgent>,
}

/// EvaluationServer for handling agent evaluation logs.
pub struct EvaluationServer {
    pub uri: String,
    pub app:Router,
}

impl EvaluationServer {
    pub async fn new(uri:String, agent_config: AgentConfig) -> anyhow::Result<Self> {
        // Initialize with the new data structure
        let db_evaluations: DashMap<String, Vec<EvaluatedAgentData>> = DashMap::new();

        let judge_agent=JudgeAgent::new(agent_config.clone()).await?;

        // Create AppState
        let app_state = AppState {
            db_evaluations: Arc::new(db_evaluations),
            judge_agent: Arc::new(judge_agent),
        };

        let app = Router::new()
            .route("/", get(root))
            .route("/log", post(log_evaluation))
            .route("/evaluations", get(list_evaluations))
            .with_state(app_state);

        Ok(Self {
            uri:uri,
            app,
        })
    }

    /// Start the HTTP server.
    pub async fn start_http(&self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.uri.clone()).await?;
        info!("Evaluation Server started at {}", self.uri);
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
    
    let judge_agent = state.judge_agent.to_owned();

    info!("Received log_evaluation request for agent: {}", log_data.agent_id);

    match judge_agent.evaluate_agent_output(log_data.clone()).await {
        Ok(evaluated_data) => {
            let db_evaluations = state.db_evaluations.clone();
            trace!("Received Agent Evaluation Data : {:?}", evaluated_data);
            
            // Get the entry for the agent_id, or create a new one if it doesn't exist.
            // Then push the new evaluation data into the vector.
            db_evaluations.entry(log_data.agent_id.clone()).or_default().push(evaluated_data);

            Ok(Json("Evaluation logged successfully".to_string()))
        }
        Err(e) => {
            let error_message = format!("Failed to evaluate agent output: {:?}", e);
            trace!("{}", error_message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}


async fn list_evaluations(
    State(state): State<AppState>,
) -> Json<Vec<EvaluatedAgentData>> {
    let db_evaluations = state.db_evaluations.clone();

    // Flatten the vectors of evaluations into a single vector
    let list_evaluations: Vec<EvaluatedAgentData> = db_evaluations
        .iter()
        .flat_map(|entry| entry.value().clone())
        .collect();

    Json(list_evaluations)
}
