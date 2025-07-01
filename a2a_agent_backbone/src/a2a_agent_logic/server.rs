use a2a_rs::adapter::{
    BearerTokenAuthenticator, DefaultRequestProcessor, HttpServer, InMemoryTaskStorage,
    NoopPushNotificationSender, SimpleAgentInfo,
};
use a2a_rs::port::{AsyncNotificationManager, AsyncTaskManager};

// SQLx storage support (feature-gated)
#[cfg(feature = "sqlx-storage")]
use a2a_rs::adapter::storage::SqlxTaskStorage;

use super::config::{AuthConfig, ServerConfig, StorageConfig};
use super::handler::SimpleAgentHandler;
use crate::a2a_agent_initialization::a2a_agent_config::RuntimeA2aConfigProject;


/// Modern A2A server setup using ReimbursementHandler
//pub struct ReimbursementServer {
pub struct SimpleAgentServer {
    config: ServerConfig,
    runtime_config: RuntimeA2aConfigProject,
}

impl SimpleAgentServer {
    /// Create a new modern reimbursement server with default config
    pub fn new(runtime_config: RuntimeA2aConfigProject) -> Self {
        let config = ServerConfig::new(
            runtime_config.agent_a2a_config.agent_a2a_host.clone(),
            runtime_config
                .agent_a2a_config
                .agent_a2a_http_port
                .clone()
                .parse::<u16>()
                .unwrap(),
            runtime_config
                .agent_a2a_config
                .agent_a2a_ws_port
                .clone()
                .parse::<u16>()
                .unwrap(),
        );

        Self {
            config,
            runtime_config,
        }
    }

    /// Create in-memory storage
    fn create_in_memory_storage(&self) -> InMemoryTaskStorage {
        tracing::info!("Using in-memory storage");
        let push_sender = NoopPushNotificationSender;
        InMemoryTaskStorage::with_push_sender(push_sender)
    }

    #[cfg(feature = "sqlx-storage")]
    /// Create SQLx storage (only available with sqlx feature)
    async fn create_sqlx_storage(
        &self,
        url: &str,
        _max_connections: u32,
        enable_logging: bool,
    ) -> Result<SqlxTaskStorage, Box<dyn std::error::Error>> {
        tracing::info!("Using SQLx storage with URL: {}", url);
        if enable_logging {
            tracing::info!("SQL query logging enabled");
        }

        // Include reimbursement-specific migrations
        let reimbursement_migrations = &[include_str!(
            "../../migrations/001_create_reimbursements.sql"
        )];

        let storage = SqlxTaskStorage::with_migrations(url, reimbursement_migrations)
            .await
            .map_err(|e| format!("Failed to create SQLx storage: {}", e))?;
        Ok(storage)
    }

    /// Start the HTTP server
    pub async fn start_http(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.config.storage {
            StorageConfig::InMemory => {
                let storage = self.create_in_memory_storage();
                self.start_http_server(storage).await
            }
            #[cfg(feature = "sqlx-storage")]
            StorageConfig::Sqlx {
                url,
                max_connections,
                enable_logging,
            } => {
                let storage = self
                    .create_sqlx_storage(url, *max_connections, *enable_logging)
                    .await?;
                self.start_http_server(storage).await
            }
            #[cfg(not(feature = "sqlx-storage"))]
            StorageConfig::Sqlx { .. } => {
                Err("SQLx storage requested but 'sqlx' feature is not enabled.".into())
            }
        }
    }

    /// Start HTTP server
    async fn start_http_server<S>(&self, storage: S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsyncTaskManager + AsyncNotificationManager + Clone + Send + Sync + 'static,
    {
        // This works by re creating an in memory task storage
        // does not use config... should be addressed
        // does not use the one from start_http()
        let storage = self.create_in_memory_storage();

        // Create message handler
        let message_handler =
            SimpleAgentHandler::with_storage(self.runtime_config.clone(), storage.clone());
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

        let agent_a2a_config = self.runtime_config.agent_a2a_config.clone();

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

        //////////////////////////////////////////////////////////////////

        let agent_a2a_config = self.runtime_config.agent_a2a_config.clone();
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

        match &self.config.storage {
            StorageConfig::InMemory => println!("💾 Storage: In-memory (non-persistent)"),
            StorageConfig::Sqlx { url, .. } => println!("💾 Storage: SQLx ({})", url),
        }

        match &self.config.auth {
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
}
