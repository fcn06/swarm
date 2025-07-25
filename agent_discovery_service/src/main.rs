use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    Json,
    Router,
    extract::{State,Form},
    http::StatusCode,
    response::{IntoResponse, Response,Html}, // Use IntoResponse for better error handling
    routing::{get, post},
};

use a2a_rs::domain::AgentCard;




// https://github.com/tokio-rs/axum/blob/main/examples/form/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agents: Arc<Mutex<HashMap<String, AgentCard>>> = Arc::new(Mutex::new(HashMap::new()));

    // Build our application with a route
    let app = Router::new()
        .route("/register", post(register_agent))
        .route("/agents", get(list_agents))
        .with_state(agents);

    // Run our app with hyper
        let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await?;
        println!("Server started");
        axum::serve(listener, app).await?;
        Ok(())

}

async fn register_agent(
    State(agents): State<Arc<Mutex<HashMap<String, AgentCard>>>>,
    Json(agent_card): Json<AgentCard>,
) -> Json<String> {
    let mut agents = agents.lock().unwrap();
    //println!("Registering agent: {}", agent_card.name);
    agents.insert(agent_card.name.clone(), agent_card);
    Json("Agent registered successfully".to_string())
}

async fn list_agents(
    State(agents): State<Arc<Mutex<HashMap<String, AgentCard>>>>,
) -> Json<Vec<AgentCard>> {
    let agents = agents.lock().unwrap();
    let agent_card_list: Vec<AgentCard> = agents.values().cloned().collect();
    println!("Listing agents: {:?}", agent_card_list);
    Json(agent_card_list)
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