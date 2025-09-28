
use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
//use a2a_rs::services::AgentInfoProvider;

use crate::business_logic::agent::{Agent};

use configuration::AgentConfig;

use crate::server::agent_handler::AgentHandler;
use std::sync::Arc;
use crate::business_logic::services::DiscoveryService;
use agent_discovery_service::discovery_service_client::agent_discovery_client::AgentDiscoveryServiceClient;
use anyhow::Result;

use uuid::Uuid;


//use agent_discovery_service::model::models::AgentSkill;
//use agent_discovery_service::model::models::AgentDefinition;
use agent_models::registry::registry_models::{AgentDefinition,AgentSkill};


pub struct AgentServer<T:Agent> {
    config: AgentConfig,
    agent:T,
    discovery_service: Option<Arc<dyn DiscoveryService>>,
}

impl<T:Agent> AgentServer<T> {
    pub async fn new(agent_config: AgentConfig, agent: T, discovery_service: Option<Arc<dyn DiscoveryService>>) -> anyhow::Result<Self> {
        Ok(Self { config:agent_config,agent:agent,discovery_service:discovery_service })
    }

    /// Create in-memory storage
    fn create_in_memory_storage(&self) -> InMemoryTaskStorage {
        tracing::info!("Using in-memory storage");
        let push_sender = NoopPushNotificationSender;
        InMemoryTaskStorage::with_push_sender(push_sender)
    }

    async fn register_with_discovery_service(&self, agent_definition: &AgentDefinition) -> Result<()> {
        let max_retries = 2;
        let mut retries = 0;
        let mut delay = 1; // seconds

        loop {
            let registration_result = if let Some(ds) = &self.discovery_service {
                // Use injected discovery service
                ds.register_agent(&agent_definition).await
            } else if let Some(discovery_url) = self.config.agent_discovery_url() {
                // Fallback to creating a client if URL is provided in config
                let client = AgentDiscoveryServiceClient::new(&discovery_url);
                client.register_agent_definition(&agent_definition)
                    .await
                    .map(|_| ()) // Map Ok(String) to Ok(())
                    .map_err(|e| e.into()) // Convert reqwest::Error to anyhow::Error
            } else {
                tracing::warn!("Discovery service not configured. Skipping registration.");
                return Ok(()); // No discovery service to register with
            };

            match registration_result {
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
                        tracing::error!("Failed to register with discovery service after {} attempts. Error: {}. Proceeding without discovery service registration.", max_retries, e);
                        // Allow the agent to start even if registration fails
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn start_http(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        let storage = self.create_in_memory_storage();

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

        let agent_definition=AgentDefinition{
            id:Uuid::new_v4().to_string(),
            name:self.config.agent_name(),
            description:self.config.agent_description(),
            skills:vec![AgentSkill{
                name:self.config.agent_skill_name(),
                description:self.config.agent_skill_description(),
                parameters:serde_json::Value::Null,
                output:serde_json::Value::Null,
            }]
        };


        if let Some(true) = self.config.agent_discoverable() {
            self.register_with_discovery_service(&agent_definition).await?;
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
