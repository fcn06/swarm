//use dashmap::DashMap;
use axum::{
    Json,
    Router,
    extract::{State},
    routing::{get, post},
    http::StatusCode,
};
use tracing::{info,trace};
use std::sync::Arc;
use crate::evaluation_server::judge_agent::{AgentEvaluationLogData, JudgeEvaluation};
use crate::evaluation_server::judge_agent::JudgeAgent;

use configuration::AgentConfig;

/// Application state holding evaluation data.
#[derive(Clone)]
pub struct AppState {
    pub judge_agent: Arc<JudgeAgent>,
}

/// EvaluationServer for handling agent evaluation logs.
pub struct EvaluationServer {
    pub uri: String,
    pub app:Router,
}

impl EvaluationServer {
    pub async fn new(uri:String, agent_config: AgentConfig) -> anyhow::Result<Self> {
        let judge_agent=JudgeAgent::new(agent_config.clone()).await?;

        // Create AppState
        let app_state = AppState {
            judge_agent: Arc::new(judge_agent),
        };

        let app = Router::new()
            .route("/", get(root))
            .route("/log", post(log_evaluation))
            .with_state(app_state);

        Ok(Self {
            uri:uri,
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
    Json(log_data): Json<AgentEvaluationLogData>,
) -> Result<Json<JudgeEvaluation>, (StatusCode, String)> {
    
    let judge_agent = state.judge_agent.to_owned();

    info!("Received log_evaluation request for agent: {}", log_data.agent_id);

    match judge_agent.evaluate_agent_output(log_data.clone()).await {
        Ok(judge_evaluation) => {
            trace!("Received Agent Evaluation Data : {:?}", judge_evaluation);
            Ok(Json(judge_evaluation))
        }
        Err(e) => {
            let error_message = format!("Failed to evaluate agent output: {:?}", e);
            trace!("{}", error_message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}
