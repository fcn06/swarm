use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AgentInfo {
    id: String,
    address: String,
    skills: Vec<String>,
}

#[tokio::main]
async fn main() {
    let agents: Arc<Mutex<HashMap<String, AgentInfo>>> = Arc::new(Mutex::new(HashMap::new()));

    let agents_filter = warp::any().map(move || Arc::clone(&agents));

    let register_agent = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(agents_filter.clone())
        .map(|agent_info: AgentInfo, agents: Arc<Mutex<HashMap<String, AgentInfo>>>| {
            let mut agents = agents.lock().unwrap();
            println!("Registering agent: {:?}", agent_info);
            agents.insert(agent_info.id.clone(), agent_info);
            warp::reply::json(&"Agent registered successfully")
        });

    let routes = register_agent;

    println!("Discovery service started on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
