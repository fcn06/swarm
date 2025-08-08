use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::port::{AsyncNotificationManager, AsyncTaskManager};
use a2a_rs::services::AgentInfoProvider;

use configuration::AgentFullConfig;

use crate::a2a_full_server::full_handler::FullAgentHandler;
use crate::a2a_full_agent_logic::full_agent::FullAgent;

//use configuration::DiscoveryServiceInteraction;
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;


/// Modern A2A server setup using ReimbursementHandler
//pub struct ReimbursementServer {
pub struct FullAgentServer {
    agent_full_config: AgentFullConfig,
    full_agent:FullAgent,
}

impl FullAgentServer {
    /// Create a new full agent server with default config
    pub async fn new(agent_full_config: AgentFullConfig) -> anyhow::Result<Self> {

        let full_agent= FullAgent::new(agent_full_config.clone()).await?;

        Ok(Self {
            agent_full_config,
            full_agent,
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
            FullAgentHandler::with_storage(storage.clone(), self.agent_full_config.clone(),self.full_agent.clone());
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

        let agent_full_config = self.agent_full_config.clone();

        let agent_info = SimpleAgentInfo::new(
            agent_full_config.agent_full_name.clone(),
            format!("http://{}:{}", agent_full_config.agent_full_host, agent_full_config.agent_full_http_port),
        )
        .with_description(agent_full_config.agent_full_description)
        //.with_provider(
        //    "Example Organization".to_string(),
        //    "https://example.org".to_string(),
        //)
        .with_documentation_url(agent_full_config.agent_full_doc_url.expect("NO DOC URL"))
        .with_streaming()
        .add_comprehensive_skill(
            agent_full_config.agent_full_skill_id,
            agent_full_config.agent_full_skill_name,
            Some(agent_full_config.agent_full_skill_description),
            Some(agent_full_config.agent_full_tags),
            Some(agent_full_config.agent_full_examples),
            Some(vec!["text".to_string(), "data".to_string()]),
            Some(vec!["text".to_string(), "data".to_string()]),
        );
        
        //////////////////////////////////////////////////////////////////
        // Register agent at launch
        let agent_discovery_client = AgentDiscoveryServiceClient::new(agent_full_config.agent_full_discovery_url.clone().expect("NO DISCOVERY URL"));
        
        let max_retries = 2;
        let mut retries = 0;
        let mut delay = 1; // seconds

        loop {
            match agent_discovery_client.register(agent_info.get_agent_card().await?).await {
                Ok(_) => {
                    tracing::info!("Agent successfully registered with discovery service.");
                    break;
                },
                Err(e) => {
                    retries += 1;
                    if retries < max_retries {
                        tracing::warn!("Failed to register with discovery service, attempt {}/{}. Error: {}. Retrying in {} seconds...", retries, max_retries, e, delay);
                        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                        delay *= 2; // Exponential backoff
                    } else {
                        tracing::error!("Failed to register with discovery service after {} attempts. Error: {}", max_retries, e);
                        break;
                    }
                }
            }
        }

        //////////////////////////////////////////////////////////////////


        let agent_full_config = self.agent_full_config.clone();
        // Create HTTP server
        let bind_address = format!("{}:{}", agent_full_config.agent_full_host, agent_full_config.agent_full_http_port);

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}:{}",
            agent_full_config.agent_full_name,agent_full_config.agent_full_host, agent_full_config.agent_full_http_port
        );
        println!(
            "📋 Agent card: http://{}:{}/agent-card",
            agent_full_config.agent_full_host, agent_full_config.agent_full_http_port
        );
        println!(
            "🛠️  Skills: http://{}:{}/skills",
            agent_full_config.agent_full_host, agent_full_config.agent_full_http_port
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



}
