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
use crate::evaluation_server::judge_agent::{AgentLogData, EvaluatedAgentData};
use crate::evaluation_server::judge_agent::JudgeAgent;

use agent_protocol_backbone::config::agent_config::AgentConfig;

/// Application state holding evaluation data.
#[derive(Clone)]
pub struct AppState {
    pub db_evaluations: Arc<DashMap<String, EvaluatedAgentData>>,
    pub judge_agent: Arc<JudgeAgent>,
}

/// EvaluationServer for handling agent evaluation logs.
pub struct EvaluationServer {
    pub uri: String,
    pub app:Router,
}

impl EvaluationServer {
    pub async fn new(agent_config: AgentConfig) -> anyhow::Result<Self> {
        let db_evaluations: DashMap<String, EvaluatedAgentData> = DashMap::new();

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
            uri:format!("http://{}:{}", agent_config.agent_host(), agent_config.agent_http_port()),
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
    
    let judge_agent = state.judge_agent.to_owned();

    info!("Received log_evaluation request for agent: {}", log_data.agent_id);

    match judge_agent.evaluate_agent_output(log_data.clone()).await {
        Ok(evaluated_data) => {

            let db_evaluations = state.db_evaluations.clone();
            info!("Received Agent Evaluation Data : {:?}", evaluated_data);
            db_evaluations.insert(log_data.agent_id.clone(), evaluated_data);
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
) -> Json<Vec<EvaluatedAgentData>> {
    let db_evaluations = state.db_evaluations.to_owned();

    let mut list_evaluations: Vec<EvaluatedAgentData> = Vec::new();

    for entry in db_evaluations.iter() {
        list_evaluations.push(entry.value().clone());
    }

    Json(list_evaluations)
}

