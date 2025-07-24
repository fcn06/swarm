use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub http_port: u16,
    pub ws_port: u16,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
}

impl ServerConfig {
    pub fn new(host: String, http_port: u16, ws_port: u16) -> Self {
        Self {
            host,
            http_port,
            ws_port,
            auth: AuthConfig::None,
            storage: StorageConfig::InMemory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthConfig {
    None,
    BearerToken {
        tokens: Vec<String>,
        format: Option<String>,
    },
    ApiKey {
        keys: Vec<String>,
        location: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageConfig {
    InMemory,
    Sqlx {
        url: String,
        migrations_folder: Option<String>,
    },
}
