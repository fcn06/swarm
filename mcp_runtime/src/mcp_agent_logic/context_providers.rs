use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use agent_core::business_logic::context_provider::{ContextProvider, ContextRequest};
use agent_core::business_logic::services::MemoryService;
use agent_models::memory::memory_models::{IdentityContext, MemoryQuery, Role};
use llm_api::chat::Message;

/// Context provider that fetches previous conversation history from MemoryService.
pub struct HistoryContextProvider {
    memory_service: Arc<dyn MemoryService>,
    history_length: usize,
}

impl HistoryContextProvider {
    pub fn new(memory_service: Arc<dyn MemoryService>, history_length: usize) -> Self {
        Self {
            memory_service,
            history_length,
        }
    }
}

#[async_trait]
impl ContextProvider for HistoryContextProvider {
    fn name(&self) -> &str {
        "history"
    }

    async fn provide_context(&self, req: &ContextRequest) -> Result<Vec<Message>> {
        let session_id = match &req.session_id {
            Some(id) if !id.is_empty() => id,
            _ => return Ok(vec![]),
        };

        let entries = self
            .memory_service
            .get_conversation(session_id, Some(self.history_length))
            .await?;

        let messages = entries
            .into_iter()
            .map(|entry| {
                let role = match entry.role {
                    Role::User => "user".to_string(),
                    Role::Agent => "assistant".to_string(),
                    Role::System => "system".to_string(),
                };
                Message {
                    role,
                    content: Some(entry.content),
                    tool_call_id: None,
                    tool_calls: None,
                }
            })
            .collect();

        Ok(messages)
    }
}

/// Context provider that injects structured agent identity (role, objectives, constraints, capabilities).
pub struct IdentityContextProvider {
    identity: IdentityContext,
}

impl IdentityContextProvider {
    pub fn new(identity: IdentityContext) -> Self {
        Self { identity }
    }
}

#[async_trait]
impl ContextProvider for IdentityContextProvider {
    fn name(&self) -> &str {
        "identity"
    }

    async fn provide_context(&self, _req: &ContextRequest) -> Result<Vec<Message>> {
        let mut sections = Vec::new();

        if !self.identity.role.is_empty() {
            sections.push(format!("Role: {}", self.identity.role));
        }
        if !self.identity.objectives.is_empty() {
            let mut obj_str = String::from("Objectives:\n");
            for obj in &self.identity.objectives {
                obj_str.push_str(&format!("- {}\n", obj));
            }
            sections.push(obj_str.trim_end().to_string());
        }
        if !self.identity.constraints.is_empty() {
            let mut c_str = String::from("Constraints:\n");
            for c in &self.identity.constraints {
                c_str.push_str(&format!("- {}\n", c));
            }
            sections.push(c_str.trim_end().to_string());
        }
        if !self.identity.capabilities.is_empty() {
            let mut cap_str = String::from("Capabilities:\n");
            for cap in &self.identity.capabilities {
                cap_str.push_str(&format!("- {}\n", cap));
            }
            sections.push(cap_str.trim_end().to_string());
        }

        if sections.is_empty() {
            return Ok(vec![]);
        }

        let full_text = sections.join("\n\n");
        Ok(vec![Message {
            role: "system".to_string(),
            content: Some(full_text),
            tool_call_id: None,
            tool_calls: None,
        }])
    }
}

/// Context provider that recalls relevant facts from MemoryService based on the user's input.
pub struct SemanticMemoryContextProvider {
    memory_service: Arc<dyn MemoryService>,
    max_facts: usize,
    category: Option<String>,
}

impl SemanticMemoryContextProvider {
    pub fn new(memory_service: Arc<dyn MemoryService>, max_facts: usize, category: Option<String>) -> Self {
        Self {
            memory_service,
            max_facts,
            category,
        }
    }
}

#[async_trait]
impl ContextProvider for SemanticMemoryContextProvider {
    fn name(&self) -> &str {
        "semantic_memory"
    }

    async fn provide_context(&self, req: &ContextRequest) -> Result<Vec<Message>> {
        if req.current_input.is_empty() {
            return Ok(vec![]);
        }

        let query = MemoryQuery {
            query: req.current_input.clone(),
            category: self.category.clone(),
            limit: Some(self.max_facts),
        };

        let facts = self.memory_service.recall_facts(&query).await?;
        if facts.is_empty() {
            return Ok(vec![]);
        }

        let mut text = String::from("Relevant Facts:\n");
        for fact in &facts {
            text.push_str(&format!("- [{}] {}\n", fact.key, fact.content));
        }

        Ok(vec![Message {
            role: "system".to_string(),
            content: Some(text.trim_end().to_string()),
            tool_call_id: None,
            tool_calls: None,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_models::memory::memory_models::{FactItem, LogEntry, Role};

    struct MockMemoryService {
        entries: Vec<LogEntry>,
        facts: Vec<FactItem>,
    }

    #[async_trait]
    impl MemoryService for MockMemoryService {
        async fn log(&self, _conv_id: String, _role: Role, _text: String, _agent: Option<String>) -> Result<()> {
            Ok(())
        }

        async fn get_conversation(&self, _conv_id: &str, limit: Option<usize>) -> Result<Vec<LogEntry>> {
            let mut res = self.entries.clone();
            if let Some(l) = limit {
                if res.len() > l {
                    res = res[res.len() - l..].to_vec();
                }
            }
            Ok(res)
        }

        async fn recall_facts(&self, query: &MemoryQuery) -> Result<Vec<FactItem>> {
            let q_lower = query.query.to_lowercase();
            let matches = self
                .facts
                .iter()
                .filter(|f| f.content.to_lowercase().contains(&q_lower) || f.key.to_lowercase().contains(&q_lower))
                .cloned()
                .collect();
            Ok(matches)
        }
    }

    #[tokio::test]
    async fn test_history_context_provider() {
        let mem = Arc::new(MockMemoryService {
            entries: vec![
                LogEntry { role: Role::User, content: "Hello".to_string(), agent_id: None },
                LogEntry { role: Role::Agent, content: "Hi there!".to_string(), agent_id: None },
            ],
            facts: vec![],
        });

        let provider = HistoryContextProvider::new(mem, 10);

        // Without session_id -> empty
        let req_empty = ContextRequest::default();
        let msgs = provider.provide_context(&req_empty).await.unwrap();
        assert!(msgs.is_empty());

        // With session_id -> formatted messages
        let req_with_session = ContextRequest {
            session_id: Some("session-123".to_string()),
            ..Default::default()
        };
        let msgs = provider.provide_context(&req_with_session).await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[0].content, Some("Hello".to_string()));
        assert_eq!(msgs[1].role, "assistant");
        assert_eq!(msgs[1].content, Some("Hi there!".to_string()));
    }

    #[tokio::test]
    async fn test_identity_context_provider() {
        let identity = IdentityContext {
            role: "Code Reviewer".to_string(),
            objectives: vec!["Ensure code quality".to_string()],
            constraints: vec!["Do not rewrite logic".to_string()],
            capabilities: vec!["rust".to_string(), "python".to_string()],
        };

        let provider = IdentityContextProvider::new(identity);
        let req = ContextRequest::default();
        let msgs = provider.provide_context(&req).await.unwrap();

        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "system");
        let content = msgs[0].content.as_ref().unwrap();
        assert!(content.contains("Role: Code Reviewer"));
        assert!(content.contains("Objectives:\n- Ensure code quality"));
        assert!(content.contains("Constraints:\n- Do not rewrite logic"));
        assert!(content.contains("Capabilities:\n- rust\n- python"));
    }

    #[tokio::test]
    async fn test_semantic_memory_context_provider() {
        let mem = Arc::new(MockMemoryService {
            entries: vec![],
            facts: vec![
                FactItem {
                    key: "user_city".to_string(),
                    content: "User lives in Paris".to_string(),
                    category: Some("profile".to_string()),
                    timestamp: None,
                },
                FactItem {
                    key: "user_os".to_string(),
                    content: "User uses Linux Ubuntu".to_string(),
                    category: Some("system".to_string()),
                    timestamp: None,
                },
            ],
        });

        let provider = SemanticMemoryContextProvider::new(mem, 5, None);

        let req = ContextRequest {
            current_input: "Paris".to_string(),
            ..Default::default()
        };
        let msgs = provider.provide_context(&req).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "system");
        let content = msgs[0].content.as_ref().unwrap();
        assert!(content.contains("[user_city] User lives in Paris"));
        assert!(!content.contains("Linux Ubuntu"));
    }
}
