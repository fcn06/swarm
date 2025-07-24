use a2a_rs::adapter::{
    BearerTokenAuthenticator, DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::port::{AsyncNotificationManager, AsyncTaskManager};

use a2a_rs::services::AgentInfoProvider;
use a2a_rs::domain::AgentCard;

use super::server_config::{AuthConfig, ServerConfig, StorageConfig};
use super::handler::BidirectionalAgentHandler;
use super::planner_agent::PlannerAgent;
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;
use configuration::{AgentBidirectionalConfig, AgentMcpConfig};
use llm_api::chat::ChatLlmInteraction;
use std::env;

/// Modern A2A server setup 
pub struct BidirectionalAgentServer {
    server_config: ServerConfig,
    llm_interaction: ChatLlmInteraction,
    agent_bidirectional_config: AgentBidirectionalConfig,
    mcp_agent:Option<McpAgent>,
    planner_agent: Option<PlannerAgent>,
}



impl BidirectionalAgentServer {
    /// Create a new modern reimbursement server with default config
    pub async fn new(agent_bidirectional_config: AgentBidirectionalConfig,) -> anyhow::Result<Self> {
        
        let server_config = ServerConfig::new(
            agent_bidirectional_config.agent_bidirectional_host.clone(),
            agent_bidirectional_config.agent_bidirectional_http_port.clone().parse::<u16>().unwrap(),
            agent_bidirectional_config.agent_bidirectional_ws_port.clone().parse::<u16>().unwrap(),
            );

             // Set model to be used
        let model_id = agent_bidirectional_config.agent_bidirectional_model_id.clone();

        // Set model to be used
        let _system_message = agent_bidirectional_config.agent_bidirectional_system_prompt.clone();

        // Set API key for LLM
        let llm_bidirectional_api_key = env::var("LLM_BIDIRECTIONAL_API_KEY").expect("LLM_BIDIRECTIONAL_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
            agent_bidirectional_config.agent_bidirectional_llm_url.clone(),
            model_id,
            llm_bidirectional_api_key,
        );

        // load mcp agent agent if it exists
        let mcp_agent = match agent_bidirectional_config.agent_bidirectional_mcp_config_path.clone() {
            None => None,
            Some(path) => {
                let agent_mcp_config=AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading agent config");
                let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                Some(mcp_agent)},
            
        };

        // load planner agent if it exists
        let planner_agent = match agent_bidirectional_config.agent_bidirectional_planner_config_path.clone() {
            None => None,
            Some(path) => {
                let agent_planner_config = configuration::AgentPlannerConfig::load_agent_config(path.as_str()).expect("Error loading planner config");
                let planner_agent = PlannerAgent::new(agent_planner_config).await?;
                Some(planner_agent)
            },
        };

        Ok(Self {
            server_config:server_config,
            llm_interaction:llm_interaction,
            agent_bidirectional_config:agent_bidirectional_config,
            mcp_agent:mcp_agent,
            planner_agent: planner_agent,
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
        match &self.server_config.storage {
            StorageConfig::InMemory => {
                let storage = self.create_in_memory_storage();
                self.start_http_server(storage).await
            }
            StorageConfig::Sqlx { .. } => {
                Err("SQLx storage requested but 'sqlx' feature is not enabled.".into())
            }
        }
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
            BidirectionalAgentHandler::with_storage(self.llm_interaction.clone(), self.mcp_agent.clone(), self.planner_agent.clone(), storage.clone());
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

        let agent_bidirectional_config = self.agent_bidirectional_config.clone();

        let agent_info = SimpleAgentInfo::new(
            agent_bidirectional_config.agent_bidirectional_name,
            format!("http://{}:{}", agent_bidirectional_config.agent_bidirectional_host, agent_bidirectional_config.agent_bidirectional_http_port),
        )
        .with_description(agent_bidirectional_config.agent_bidirectional_description)
        //.with_provider(
        //    "Example Organization".to_string(),
        //    "https://example.org".to_string(),
        //)
        .with_documentation_url(agent_bidirectional_config.agent_bidirectional_doc_url.expect("NO DOC URL"))
        .with_streaming()
        .add_comprehensive_skill(
            agent_bidirectional_config.agent_bidirectional_skill_id,
            agent_bidirectional_config.agent_bidirectional_skill_name,
            Some(agent_bidirectional_config.agent_bidirectional_skill_description),
            Some(agent_bidirectional_config.agent_bidirectional_tags),
            Some(agent_bidirectional_config.agent_bidirectional_examples),
            Some(vec!["text".to_string(), "data".to_string()]),
            Some(vec!["text".to_string(), "data".to_string()]),
        );

        self.register(agent_info.get_agent_card().await?).await?;

        //////////////////////////////////////////////////////////////////

        let agent_bidirectional_config = self.agent_bidirectional_config.clone();
        // Create HTTP server
        let bind_address = format!("{}:{}", agent_bidirectional_config.agent_bidirectional_host, agent_bidirectional_config.agent_bidirectional_http_port);

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}:{}",
            agent_bidirectional_config.agent_bidirectional_name,agent_bidirectional_config.agent_bidirectional_host, agent_bidirectional_config.agent_bidirectional_http_port
        );
        println!(
            "📋 Agent card: http://{}:{}/agent-card",
            agent_bidirectional_config.agent_bidirectional_host, agent_bidirectional_config.agent_bidirectional_http_port
        );
        println!(
            "🛠️  Skills: http://{}:{}/skills",
            agent_bidirectional_config.agent_bidirectional_host, agent_bidirectional_config.agent_bidirectional_http_port
        );

        match &self.server_config.storage {
            StorageConfig::InMemory => println!("💾 Storage: In-memory (non-persistent)"),
            StorageConfig::Sqlx { url, .. } => println!("💾 Storage: SQLx ({})", url),
        }

        match &self.server_config.auth {
            AuthConfig::None => {
                println!("🔓 Authentication: None (public access)");

                // Create server without authentication
                let server = HttpServer::new(processor, agent_info, bind_address);
                server
                    .start()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            AuthConfig::BearerToken { tokens, format } => {
                println!(
                    "🔐 Authentication: Bearer token ({} token(s){})",
                    tokens.len(),
                    format
                        .as_ref()
                        .map(|f| format!(", format: {}", f))
                        .unwrap_or_default()
                );

                let authenticator = BearerTokenAuthenticator::new(tokens.clone());
                let server =
                    HttpServer::with_auth(processor, agent_info, bind_address, authenticator);
                server
                    .start()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            AuthConfig::ApiKey {
                keys,
                location,
                name,
            } => {
                println!(
                    "🔐 Authentication: API key ({} {}, {} key(s))",
                    location,
                    name,
                    keys.len()
                );
                println!("⚠️  API key authentication not yet supported, using no authentication");

                // Create server without authentication
                let server = HttpServer::new(processor, agent_info, bind_address);
                server
                    .start()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        }
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

        let discovery_url=self.agent_bidirectional_config.agent_bidirectional_discovery_url.clone().expect("NO DISCOVERY URL");

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
