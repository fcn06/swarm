
use a2a_rs::adapter::{
    DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};

use crate::business_logic::agent::{Agent};

use configuration::AgentConfig;

use crate::server::agent_handler::AgentHandler;
use std::sync::Arc;
use crate::business_logic::services::DiscoveryService;

use anyhow::Result;

use uuid::Uuid;


use agent_models::registry::registry_models::{AgentDefinition,AgentSkillDefinition};


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

        if let Some(ds) = &self.discovery_service {
            loop {
                let registration_result = ds.register_agent(&agent_definition).await;

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
        } else {
            tracing::warn!("Discovery service not configured. Skipping registration.");
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

        let agent_http_endpoint= format!("{}", self.config.agent_http_endpoint());
        let _agent_ws_endpoint= format!("{}", self.config.agent_ws_endpoint());

        let agent_info = SimpleAgentInfo::new(
            self.config.agent_name(),
            agent_http_endpoint.clone(),
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
            agent_endpoint:  agent_http_endpoint.clone(),
            skills:vec![AgentSkillDefinition{
                name:self.config.agent_skill_name(),
                description:self.config.agent_skill_description(),
                parameters:serde_json::Value::Null,
                output:serde_json::Value::Null,
            }]
        };


        if let Some(true) = self.config.agent_discoverable() {
            self.register_with_discovery_service(&agent_definition).await?;
        }

        // bind address is on format  0.0.0.0:0000
        let bind_address = agent_http_endpoint.clone().replace("http://","");

        println!(
            "🌐 Starting HTTP a2a agent server {} on {}",
            self.config.agent_name(), self.config.agent_http_endpoint()
        );
        println!(
            "📋 Agent card: {}/agent-card",
            self.config.agent_http_endpoint(),
        );
        println!(
            "🛠️  Skills: {}/skills",
            self.config.agent_http_endpoint()
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
