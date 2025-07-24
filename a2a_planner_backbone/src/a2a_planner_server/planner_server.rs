use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::port::{AsyncNotificationManager, AsyncTaskManager};

use a2a_rs::services::AgentInfoProvider;
use a2a_rs::domain::AgentCard;

use super::planner_handler::SimplePlannerAgentHandler;
use crate::a2a_agent_logic::planner_agent::PlannerAgent;
use configuration::AgentPlannerConfig;

/// Modern A2A server setup using ReimbursementHandler
//pub struct ReimbursementServer {
pub struct SimplePlannerAgentServer {
    agent_planner_config: AgentPlannerConfig,
    planner_agent: PlannerAgent,
}

impl SimplePlannerAgentServer {
    /// Create a new modern reimbursement server with default config
    pub async fn new(agent_planner_config: AgentPlannerConfig) -> anyhow::Result<Self> {

        // Initialize the Planner Agent
        let  planner_agent = PlannerAgent::new(agent_planner_config.clone()).await?;

        Ok(Self {
            agent_planner_config:agent_planner_config,
            planner_agent:planner_agent,
        })
    }

    /// Create in-memory storage
    fn create_in_memory_storage(&self) -> InMemoryTaskStorage {
        tracing::info!("Using in-memory storage");
        let push_sender = NoopPushNotificationSender;
        InMemoryTaskStorage::with_push_sender(push_sender)
    }

    /// Start the HTTP server
    pub async fn start_http(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        let storage = self.create_in_memory_storage();
        self.start_http_server(storage).await
    }

    /// Start HTTP server
    async fn start_http_server<S>(&self, _storage: S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncTaskManager + AsyncNotificationManager + Clone + Send + Sync + 'static,
    {
        // This works by re creating an in memory task storage
        // does not use config... should be addressed
        // does not use the one from start_http()
        let storage = self.create_in_memory_storage();

        // Create message handler
        let message_handler =
            SimplePlannerAgentHandler::with_storage(storage.clone(), self.agent_planner_config.clone(),self.planner_agent.clone());
        self.start_with_handler(message_handler, storage).await
    }

    /// Start HTTP server with specific handler
    async fn start_with_handler<S, H>(
        &self,
        message_handler: H,
        storage: S,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncTaskManager + AsyncNotificationManager + Clone + Send + Sync + 'static,
        H: a2a_rs::port::message_handler::AsyncMessageHandler + Clone + Send + Sync + 'static,
    {
        // Create processor with separate handlers
        let processor = DefaultRequestProcessor::new(
            message_handler,
            storage.clone(), // storage implements AsyncTaskManager
            storage,         // storage also implements AsyncNotificationManager
        );

        //////////////////////////////////////////////////////////////////

        let agent_planner_config = self.agent_planner_config.clone();

        let agent_info = SimpleAgentInfo::new(
            agent_planner_config.agent_planner_name.clone(),
            format!("http://{}:{}", agent_planner_config.agent_planner_host, agent_planner_config.agent_planner_http_port),
        );

        // todo: add skills

        // register the planner agent server
        self.register(agent_info.get_agent_card().await?).await?;

        //////////////////////////////////////////////////////////////////

        // Create HTTP server
        let bind_address = format!("{}:{}", agent_planner_config.agent_planner_host, agent_planner_config.agent_planner_http_port);

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}:{}",
            agent_planner_config.agent_planner_name,agent_planner_config.agent_planner_host, agent_planner_config.agent_planner_http_port
        );
        println!(
            "📋 Agent card: http://{}:{}/agent-card",
            agent_planner_config.agent_planner_host, agent_planner_config.agent_planner_http_port
        );
        println!(
            "🛠️  Skills: http://{}:{}/skills",
            agent_planner_config.agent_planner_host, agent_planner_config.agent_planner_http_port
        );

        println!("💾 Storage: In-memory (non-persistent)");
        println!("🔓 Authentication: None (public access)");
         // Create server without authentication
         let server = HttpServer::new(processor, agent_info, bind_address);
         server
             .start()
             .await
             .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)

    }

    /// Start the WebSocket server (simplified for now)
    pub async fn start_websocket(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔌 WebSocket server not yet implemented with authentication");
        println!("🔌 Use HTTP server for now");
        Err("WebSocket server not yet implemented".into())
    }

    /// Start both HTTP and WebSocket servers (simplified for now)
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting modern reimbursement agent...");
        println!("Note: Starting HTTP server only for now. WebSocket support coming soon.");
        self.start_http().await
    }


    /// Start both HTTP and WebSocket servers (simplified for now)
    pub async fn register(&self, agent_card:AgentCard) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Registering Agent ...");

        let discovery_url=self.agent_planner_config.agent_planner_discovery_url.clone().expect("NO DISCOVERY URL");

        let register_uri=format!("{}/register",discovery_url);

        let agent_registered = reqwest::Client::new()
        .post(register_uri)
        .json(&agent_card)
        .send()
        .await;

        match agent_registered {
            Ok(response) => { println!("Successfully registered server agent: {:?}", response);}
            Err(e) => {
                if e.is_connect() {
                    eprintln!("Connection error: The target server is not up or reachable. Details: {:?}", e);
                } else if e.is_timeout() {
                    eprintln!("Request timed out: {:?}", e);
                } else if e.is_status() {
                    // Handle HTTP status errors (e.g., 404, 500)
                    eprintln!("HTTP status error: {:?}", e.status());
                } else {
                    eprintln!("An unexpected reqwest error occurred: {:?}", e);
                }
                //return Err(e);
            }
        }

        Ok(())
    }

    /// Start both HTTP and WebSocket servers (simplified for now)
    pub async fn list_registered_agents(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 List Registered Agents ...");
        Ok(())
    }


}
