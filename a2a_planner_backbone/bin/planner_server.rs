//First draft of planner server to achieve recursivity
/* 
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{
    error,
    info,
};

use configuration::planner_agent_config::PlannerAgentDefinition;
use a2a_planner_backbone::a2a_agent_logic::planner_agent::PlannerAgent;
use a2a_planner_backbone::a2a_plan::plan_definition::ExecutionResult;
use a2a_rs::domain::Message;

/// State for the Axum server, holding a thread-safe reference to the PlannerAgent.
#[derive(Clone)]
struct AppState {
    planner_agent: Arc<Mutex<PlannerAgent>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Loading planner agent configuration...");
    // Load PlannerAgent configuration
    let config = PlannerAgentDefinition::load_from_default_path().await?;
    info!("Planner agent configuration loaded.");

    info!("Initializing PlannerAgent...");
    let planner_agent = PlannerAgent::new(config).await?;
    info!("PlannerAgent initialized.");

    // Wrap the PlannerAgent in Arc and Mutex for thread-safe sharing
    let shared_planner_agent = Arc::new(Mutex::new(planner_agent));

    // Create the application state
    let app_state = AppState {
        planner_agent: shared_planner_agent,
    };

    // Build our application with a single route
    let app = Router::new()
        .route("/plan", post(handle_plan_request))
        .with_state(app_state);

    // Run the server
    let addr = "0.0.0.0:8000".parse().unwrap();
    info!("Planner server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to start axum server")?;

    Ok(())
}

/// HTTP handler for the /plan endpoint.
/// It receives a Message, processes it with the PlannerAgent, and returns an ExecutionResult.
async fn handle_plan_request(
    State(app_state): State<AppState>,
    Json(request_message): Json<Message>,
) -> Result<Json<ExecutionResult>, (StatusCode, String)> {
    info!("Received new plan request.");

    let mut planner = app_state.planner_agent.lock().await;

    match planner.handle_user_request(request_message).await {
        Ok(result) => {
            info!("Plan request processed successfully.");
            Ok(Json(result))
        }
        Err(e) => {
            error!("Error processing plan request: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to process plan request: {}", e),
            ))
        }
    }
}
*/