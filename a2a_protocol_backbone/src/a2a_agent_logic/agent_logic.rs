

// --- AgentLogic Trait ---

/// Defines the core interface for an A2A agent's business logic.
/// Implementations will vary based on whether it's a simple or orchestration agent.
#[async_trait]
pub trait AgentLogic: Send + Sync {
    /// Handles an incoming A2A request and returns an A2A response.
    async fn handle_request(&self, request: A2ARequest) -> Result<A2AResponse, A2AProtocolError>;
}

// --- Common Server Utilities ---

/// A generic A2A server handler that uses an AgentLogic implementation.
pub async fn a2a_server_handler<T: AgentLogic>(
    Json(request): Json<A2ARequest>,
    Extension(agent_logic): Extension<Arc<T>>,
) -> Json<A2AResponse> {
    info!("Received A2A request for agent {}: {:?}", request.agent_id, request.request_id);

    let response = match agent_logic.handle_request(request.clone()).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error handling request {}: {}", request.request_id, e);
            A2AResponse {
                request_id: request.request_id,
                agent_id: request.agent_id,
                payload: serde_json::json!({"error": e.to_string()}),
                status: ResponseStatus::Failure(e.to_string()),
            }
        }
    };
    Json(response)
}


/// Creates a common A2A Server.
pub fn create_a2a_router<T: AgentLogic + 'static>(agent_logic: Arc<T>) -> Router {
    Router::new()
        .route("/a2a/request", post(a2a_server_handler::<T>))
        .layer(Extension(agent_logic))
}