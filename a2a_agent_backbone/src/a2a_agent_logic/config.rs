use serde::{Deserialize, Serialize};
use std::env;

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    /// In-memory storage (default)
    InMemory,
    /// SQLx-based persistent storage
    Sqlx {
        /// Database URL (e.g., sqlite:tasks.db, postgres://localhost/a2a)
        url: String,
        /// Maximum number of connections in the pool
        #[serde(default = "default_max_connections")]
        max_connections: u32,
        /// Enable SQL query logging
        #[serde(default)]
        enable_logging: bool,
    },
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::InMemory
    }
}

impl StorageConfig {
    /// Create storage config from environment variables
    pub fn from_env() -> Self {
        if let Ok(url) = env::var("DATABASE_URL") {
            Self::Sqlx {
                url,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_max_connections),
                enable_logging: env::var("DATABASE_ENABLE_LOGGING")
                    .ok()
                    .map(|s| s.to_lowercase() == "true" || s == "1")
                    .unwrap_or(false),
            }
        } else {
            Self::InMemory
        }
    }
}

fn default_max_connections() -> u32 {
    10
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port for HTTP server
    pub http_port: u16,
    /// Port for WebSocket server
    pub ws_port: u16,
    /// Storage backend configuration
    #[serde(default)]
    pub storage: StorageConfig,
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
}

impl ServerConfig {
    /// Create config from environment variables
    pub fn new(host: String, http_port: u16, ws_port: u16) -> Self {
        Self {
            host: host,
            http_port: http_port,
            ws_port: ws_port,
            storage: StorageConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    /// No authentication (default for development)
    None,
    /// Bearer token authentication
    BearerToken {
        /// List of valid tokens
        tokens: Vec<String>,
        /// Optional bearer format description (e.g., "JWT")
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
    },
    /// API Key authentication
    ApiKey {
        /// Valid API keys
        keys: Vec<String>,
        /// Location of the API key: "header", "query", or "cookie"
        #[serde(default = "default_api_key_location")]
        location: String,
        /// Name of the header/query param/cookie
        #[serde(default = "default_api_key_name")]
        name: String,
    },
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::None
    }
}

impl AuthConfig {
    /// Create auth config from environment variables
    pub fn from_env() -> Self {
        // Check for bearer tokens first
        if let Ok(tokens_str) = env::var("AUTH_BEARER_TOKENS") {
            let tokens: Vec<String> = tokens_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !tokens.is_empty() {
                return Self::BearerToken {
                    tokens,
                    format: env::var("AUTH_BEARER_FORMAT").ok(),
                };
            }
        }

        // Check for API keys
        if let Ok(keys_str) = env::var("AUTH_API_KEYS") {
            let keys: Vec<String> = keys_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !keys.is_empty() {
                return Self::ApiKey {
                    keys,
                    location: env::var("AUTH_API_KEY_LOCATION")
                        .unwrap_or_else(|_| default_api_key_location()),
                    name: env::var("AUTH_API_KEY_NAME").unwrap_or_else(|_| default_api_key_name()),
                };
            }
        }

        // Default to no authentication
        Self::None
    }
}

fn default_api_key_location() -> String {
    "header".to_string()
}

fn default_api_key_name() -> String {
    "X-API-Key".to_string()
}
