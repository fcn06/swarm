use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response}, // Use IntoResponse for better error handling
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};


use a2a_rs::{
    HttpClient,
    domain::{Message, Part},
    services::AsyncA2AClient,
};

use llm_api::chat::Message as Message_Llm;

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
    info!("Initializing API endpoint...");

    let app = Router::new()
        .route("/", get(root))
        .route("/msg", post(post_msg))
        .with_state(app_state.clone()); // Pass the cloned AppState

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("API Listener bound to 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World! Agent is running."
}

// Updated handler to accept AppState
async fn post_msg(
    State(state): State<AppState>, // Extract the AppState
    Json(payload): Json<Message_Llm>,
) -> Result<impl IntoResponse, ApiError> {
    // Return Result for error handling
    info!("Received message: {:?}", payload);

    let a2a_client = state.a2a_client.clone();

    let task_id = format!("task-{}", uuid::Uuid::new_v4());

    let message_id = uuid::Uuid::new_v4().to_string();

    // todo:to be modified to pass the exact query
    let message = Message::user_text(
        "What is the weather like in Boston ?".to_string(),
        message_id,
    );

    // Send a task message
    println!("Sending message to task...");
    let task = a2a_client
        .send_task_message(&task_id, &message, None, Some(50))
        .await.unwrap();
    println!("Got response with status: {:?}", task.status.state);

    if let Some(response_message) = task.status.message {
        println!("Agent response:");
        for part in response_message.parts {
            match part {
                Part::Text { text, .. } => println!("  {}", text),
                _ => println!("  [Non-text content]"),
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(Message_Llm {
            // Using OK instead of CREATED
            role: "assistant".to_string(), // Use config for role
            content: Some("Done".to_string()),
            tool_call_id: None,
            tool_calls:None
        }),
    ))

   
}
