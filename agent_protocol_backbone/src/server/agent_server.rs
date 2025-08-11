
use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::services::AgentInfoProvider;



use crate::business_logic::agent::{Agent, AgentConfig};
use crate::server::agent_handler::AgentHandler;



pub struct AgentServer<T:Agent,C: AgentConfig> {
    config: C,
    agent:T,
}

impl<T:Agent,C: AgentConfig> AgentServer<T,C> {
    pub async fn new(agent_config: C) -> anyhow::Result<Self> {
        // todo: remove unwrap()
        let agent= Agent::new(agent_config.clone()).await?;
        Ok(Self { config:agent_config,agent:agent })
    }

    /// Create in-memory storage
    fn create_in_memory_storage(&self) -> InMemoryTaskStorage {
        tracing::info!("Using in-memory storage");
        let push_sender = NoopPushNotificationSender;
        InMemoryTaskStorage::with_push_sender(push_sender)
    }

    pub async fn start_http(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        let storage = self.create_in_memory_storage();

        //let message_handler = AgentHandler::<T>::new(self.agent.clone());
        let message_handler = AgentHandler::<T>::with_storage(self.agent.clone(),storage.clone());

        let processor = DefaultRequestProcessor::new(
            message_handler,
            storage.clone(),
            storage,
        );

        let agent_info = SimpleAgentInfo::new(
            self.config.agent_name(),
            format!("http://{}:{}", self.config.agent_host(), self.config.agent_http_port()),
        )
        .with_description(self.config.agent_description())
        .with_documentation_url(self.config.agent_doc_url().expect("NO DOC URL PROVIDED IN CONFIG"))
        .with_streaming()
        .add_comprehensive_skill(
            self.config.agent_skill_id(),
            self.config.agent_skill_name(),
            Some(self.config.agent_skill_description()),
            Some(self.config.agent_tags()),
            Some(self.config.agent_examples()),
            Some(vec!["text".to_string(), "data".to_string()]),
            Some(vec!["text".to_string(), "data".to_string()]),
        );

        // Agent discovery registration (optional, but good practice)
        if let Some(discovery_url) = self.config.agent_discovery_url() {
            let agent_discovery_client = agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient::new(discovery_url);
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
                            // Optionally, return an error here if registration is critical
                            break;
                        }
                    }
                }
            }
        }

        let bind_address = format!("{}:{}", self.config.agent_host(), self.config.agent_http_port());

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}:{}",
            self.config.agent_name(), self.config.agent_host(), self.config.agent_http_port()
        );
        println!(
            "📋 Agent card: http://{}:{}/agent-card",
            self.config.agent_host(), self.config.agent_http_port()
        );
        println!(
            "🛠️  Skills: http://{}:{}/skills",
            self.config.agent_host(), self.config.agent_http_port()
        );
        println!("💾 Storage: In-memory (non-persistent)");
        println!("🔓 Authentication: None (public access)");

        let server = HttpServer::new(processor, agent_info, bind_address);
        server
            .start()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
