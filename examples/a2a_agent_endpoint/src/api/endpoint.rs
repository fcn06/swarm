
use axum::{
    Json,
    Router,
    extract::{State,Form},
    http::StatusCode,
    response::{IntoResponse, Response,Html}, // Use IntoResponse for better error handling
    routing::{get, post},
};

use serde::{Deserialize, Serialize};
use tracing::{info, error,debug};


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

pub async fn run_endpoint(app_state: AppState, uri: String) -> Result<(), Box<dyn std::error::Error>> {
    
    //let uri="0.0.0.0:3000".to_string();
    info!("Initializing API endpoint...");

    let app = Router::new()
        .route("/", get(root))
        .route("/msg", post(post_msg))
        .with_state(app_state.clone()); // Pass the cloned AppState

    let listener = tokio::net::TcpListener::bind(uri).await?;
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
    debug!("Received message: {:?}", payload);

    let a2a_client = state.a2a_client.clone();

    // retrieving the message from user query
    // Sample call : 
    // curl -d '{"role":"user", "content":"What are the details of customer_id 1234 ?"}' -H "Content-Type: application/json" -X POST http://localhost:3000/msg
    let message_id = uuid::Uuid::new_v4().to_string();
    let message = Message::user_text(
        payload.content.unwrap(),
        message_id,
    );

    // Send a task message
    debug!("Sending message to task...");
    let task_id = format!("task-{}", uuid::Uuid::new_v4());
    let task = a2a_client
        .send_task_message(&task_id, &message, None, Some(50))
        .await
        .unwrap();
    

    debug!("Got response with status: {:?}", task.status.state);

    if let Some(response_message) = task.status.message {
        
        let assistant_response=extract_text_from_message(&response_message).await;

        Ok((
            StatusCode::OK,
            Json(Message_Llm {
                // Using OK instead of CREATED
                role: "assistant".to_string(), // Use config for role
                content: Some(assistant_response),
                tool_call_id: None,
                tool_calls:None
            }),
        ))

    } else {
        error!("No response message received");
        Err(ApiError {
            message: "No response message received".to_string(),
        })
    }

   
}

// Helper function to extract text from a Message
async fn extract_text_from_message( message: &Message) -> String {
    message
        .parts
        .iter()
        .filter_map(|part| {
            if let Part::Text { text, metadata: _ } = part {
                Some(text.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

#[allow(dead_code)]
async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post">
                    <label for="name">
                        Enter your name:
                        <input type="text" name="name">
                    </label>

                    <label>
                        Enter your email:
                        <input type="text" name="email">
                    </label>

                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}
