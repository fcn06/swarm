use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    User,
    Agent,
    System,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Agent => write!(f, "Agent"),
            Role::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub role: Role,
    pub content: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogPayload {
    pub conversation_id: String,
    pub role: Role,
    pub content: String,
    pub agent_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub log_entries:Vec<LogEntry>,
}