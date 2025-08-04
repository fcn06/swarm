use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::port::{AsyncNotificationManager, AsyncTaskManager};

use a2a_rs::services::AgentInfoProvider;
use configuration::DiscoveryServiceInteraction;


use crate::a2a_agent_server::agent_handler::SimpleAgentHandler;
use crate::a2a_agent_logic::simple_agent::SimpleAgent;

use configuration::{AgentA2aConfig};

use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;


/// Modern A2A server setup 
pub struct SimpleAgentServer {
    agent_a2a_config: AgentA2aConfig,
    simple_agent:SimpleAgent,
}


impl SimpleAgentServer {

    pub async fn new(agent_a2a_config: AgentA2aConfig) -> anyhow::Result<Self> {

        let simple_agent= SimpleAgent::new(agent_a2a_config.clone()).await?;

        Ok(Self {
            agent_a2a_config,
            simple_agent,
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
                SimpleAgentHandler::with_storage(self.simple_agent.clone(),storage.clone());
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

        let agent_a2a_config = self.agent_a2a_config.clone();

        let agent_info = SimpleAgentInfo::new(
            agent_a2a_config.agent_a2a_name,
            format!("http://{}:{}", agent_a2a_config.agent_a2a_host, agent_a2a_config.agent_a2a_http_port),
        )
        .with_description(agent_a2a_config.agent_a2a_description)
        //.with_provider(
        //    "Example Organization".to_string(),
        //    "https://example.org".to_string(),
        //)
        .with_documentation_url(agent_a2a_config.agent_a2a_doc_url.expect("NO DOC URL"))
        .with_streaming()
        .add_comprehensive_skill(
            agent_a2a_config.agent_a2a_skill_id,
            agent_a2a_config.agent_a2a_skill_name,
            Some(agent_a2a_config.agent_a2a_skill_description),
            Some(agent_a2a_config.agent_a2a_tags),
            Some(agent_a2a_config.agent_a2a_examples),
            Some(vec!["text".to_string(), "data".to_string()]),
            Some(vec!["text".to_string(), "data".to_string()]),
        );

         let agent_a2a_config = self.agent_a2a_config.clone();

         agent_a2a_config.register(agent_info.get_agent_card().await?).await?;

        // Make it more resilient, especially if the discovery service is not up
        // let agent_discovery_client=AgentDiscoveryServiceClient::new(agent_a2a_config.agent_a2a_discovery_url.clone().expect("NO DISCOVERY URL"));
        // agent_discovery_client.register(agent_info.get_agent_card().await?).await?;


        //////////////////////////////////////////////////////////////////

        let agent_a2a_config = self.agent_a2a_config.clone();
        // Create HTTP server
        let bind_address = format!("{}:{}", agent_a2a_config.agent_a2a_host, agent_a2a_config.agent_a2a_http_port);

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}:{}",
            agent_a2a_config.agent_a2a_name,agent_a2a_config.agent_a2a_host, agent_a2a_config.agent_a2a_http_port
        );
        println!(
            "📋 Agent card: http://{}:{}/agent-card",
            agent_a2a_config.agent_a2a_host, agent_a2a_config.agent_a2a_http_port
        );
        println!(
            "🛠️  Skills: http://{}:{}/skills",
            agent_a2a_config.agent_a2a_host, agent_a2a_config.agent_a2a_http_port
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
