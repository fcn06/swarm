use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response}, // Use IntoResponse for better error handling
    routing::{get, post},
};
use llm_api::chat::Message;

use tracing::{info, error,warn};

use serde::Serialize;
use std::sync::Arc; // Use log

use crate::AppState;


// Define a custom error type for API responses
#[derive(Serialize)]
struct ApiError {
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

pub async fn run_endpoint(app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    // Accept AppState
    // initialize tracing - This should ideally be done in main.rs
    // tracing_subscriber::fmt::init();
    info!("Initializing API endpoint...");

    let app = Router::new()
        .route("/", get(root))
        .route("/msg", post(post_msg))
        .with_state(app_state.clone()); // Pass the cloned AppState

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("API Listener bound to 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    // --- MCP Client Cancellation Logic ---
    // This logic might be better placed in main.rs after the server stops,
    // especially if AppState holds the only Arc reference when the server exits.
    info!("Shutting down endpoint. Attempting to cancel MCP client...");
    let mcp_client = app_state.mcp_agent.mcp_client; // Get client from state

    // Attempt to get exclusive ownership to cancel.
    match Arc::try_unwrap(mcp_client) {
        Ok(mcp_client_owned) => {
            info!("Successfully obtained exclusive ownership of MCP client. Cancelling...");
            mcp_client_owned.cancel().await?;
            info!("MCP client cancelled successfully.");
        }
        Err(_original_arc) => {
            // This error is expected if the state was cloned elsewhere or held onto.
            error!("Cannot cancel MCP client: still shared. Was the server shut down gracefully?");
            // Return an error or handle as appropriate for your application shutdown.
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "MCP client core still shared, cannot cancel upon endpoint exit",
            )));
        }
    }
    // --- End MCP Client Cancellation Logic ---

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World! Agent is running."
}

// Updated handler to accept AppState
async fn post_msg(
    State(mut state): State<AppState>, // Extract the AppState
    Json(payload): Json<Message>,
) -> Result<impl IntoResponse, ApiError> {
    // Return Result for error handling
    info!("Received message: {:?}", payload);

    // Call run_agent, passing both project and agent configs
    match state.mcp_agent.run_agent_internal(
        payload,
    )
    .await
    {
        Ok(Some(msg)) => {
            info!("Agent returned response: {:?}", msg);
            let _= state.mcp_agent.reset_messages();
            Ok((StatusCode::CREATED, Json(msg)))
        }
        Ok(None) => {
            warn!("Agent finished without a final message.");
            // Return a specific no-content message or an error
            let _= state.mcp_agent.reset_messages();
            Ok((
                StatusCode::OK,
                Json(Message {
                    // Using OK instead of CREATED
                    role: state.agent_mcp_config.agent_mcp_role_assistant.clone(), // Use config for role
                    content: Some("Agent finished processing, but no specific response was generated."
                        .to_string()),
                    tool_call_id: None,
                    tool_calls:None
                }),
            ))
        }
        Err(e) => {
            error!("Error running agent: {}", e);
            // Return a structured error response
            Err(ApiError {
                message: format!("Agent execution failed: {}", e),
            })
        }
    }
}


