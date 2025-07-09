//! Simple agent handler for examples and testing
//!
//! This provides a complete agent implementation that bundles all business capabilities
//! (message handling, task management, notifications, and streaming) with in-memory storage.
//!
//! For production agents, you typically want to implement your own message handler
//! and compose it with the storage implementations directly.

use std::sync::{Arc};

use async_trait::async_trait;

use a2a_rs::{
    adapter::storage::InMemoryTaskStorage,
    domain::{
        A2AError, Message, Part as MessagePart, Task, TaskArtifactUpdateEvent,
        TaskPushNotificationConfig, TaskState, TaskStatusUpdateEvent,
    },
    port::{
        AsyncMessageHandler, AsyncNotificationManager, AsyncStreamingHandler, AsyncTaskManager,
        streaming_handler::Subscriber,
    },
};

use llm_api::chat::Message as Message_Llm;
use mcp_agent_backbone::mcp_agent_logic::agent::run_agent;

use crate::a2a_agent_initialization::a2a_agent_config::RuntimeA2aConfigProject;

/// Simple agent handler that coordinates all business capability traits
/// by delegating to InMemoryTaskStorage which implements the actual functionality.
///
/// This is useful for:
/// - Quick prototyping
/// - Simple echo/test agents
/// - Examples and demos
/// - Agents that don't need custom message processing
///
/// For production agents with custom business logic, implement your own
/// `AsyncMessageHandler` and compose it with storage using `DefaultRequestProcessor`.
///
/// Todo : alter SimpleAgentHandler definition to add appropriate runtine entities to connect to AI, MCP, etc...
///
#[derive(Clone)]
pub struct SimpleAgentHandler {
    pub a2a_runtime_config_project: RuntimeA2aConfigProject,
    /// Task storage that implements all the business capabilities
    storage: Arc<InMemoryTaskStorage>,
}

impl SimpleAgentHandler {
    /// Create a new simple agent handler
    pub fn new(a2a_runtime_config_project: RuntimeA2aConfigProject) -> Self {
        println!("Creating SimpleAgentHandler");
        Self {
            a2a_runtime_config_project: a2a_runtime_config_project,
            storage: Arc::new(InMemoryTaskStorage::new()),
        }
    }

    /// Create with a custom storage implementation
    pub fn with_storage(
        a2a_runtime_config_project: RuntimeA2aConfigProject,
        storage: InMemoryTaskStorage,
    ) -> Self {
        Self {
            a2a_runtime_config_project: a2a_runtime_config_project,
            storage: Arc::new(storage),
        }
    }

    /// Get a reference to the underlying storage
    #[allow(dead_code)]
    pub fn storage(&self) -> &Arc<InMemoryTaskStorage> {
        &self.storage
    }

    fn a2a_message_to_llm_message(&self, a2a_message: &Message) -> Result<Message_Llm, A2AError> {
        // Extract user query
        let user_query = a2a_message
            .parts
            .iter()
            .filter_map(|part| match part {
                MessagePart::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Convert A2A Message into LLM Message
        let llm_msg = Message_Llm {
            role: "user".to_string(),
            content: Some(user_query.to_string()),
            tool_call_id: None,
            tool_calls:None
        };

        Ok(llm_msg)
    }

    // other specific functions, like Validate Content, etc...
    fn llm_message_to_a2a_message(&self, llm_message: Message_Llm) -> Result<Message, A2AError> {
        // Convert LLM Message into A2A
        // todo use agent_text or user_text depending on role
        let message_id = uuid::Uuid::new_v4().to_string();
        let llm_msg = Message::agent_text(llm_message.content.expect("Empty Message"), message_id);
        Ok(llm_msg)
    }
}

// Asynchronous trait implementations - delegate to storage

#[async_trait]
impl AsyncMessageHandler for SimpleAgentHandler {
    async fn process_message<'a>(
        &self,
        task_id: &'a str,
        message: &'a Message,
        session_id: Option<&'a str>,
    ) -> Result<Task, A2AError> {
        // This is where we need to process custom code for message handling
        // including communication with mcp
        // see agent_swarm for details
        // replace the below default message handler

        // Transform a2a message into llm message
        let llm_msg = self.a2a_message_to_llm_message(&message)?;

        // Create or get the session ID
        //let _session_id = session_id.unwrap_or("default_session").to_string();

        // Create a new task
        //let mut task = self.create_task(task_id,"context_task").await?;
        let _task = self.create_task(task_id, "context_task").await?;

        //println!("Task Created : {:#?}",task.clone());

        // Invoke the agent, this is where business logic needs to be implemented
        let response = run_agent(
            self.a2a_runtime_config_project
                .mcp_runtime_config
                .clone()
                .expect("No runtime for mcp"),
            self.a2a_runtime_config_project
                .agent_mcp_config
                .clone()
                .expect("No config for mcp"),
            llm_msg.clone(),
        )
        .await
        .unwrap();

        // Convert the message Back to A2A Message
        // todo : make it resilient and remove unwrap()
        let response_message = self.llm_message_to_a2a_message(response.unwrap())?;

        // Add the message to the task and update status
        let task = self
            .update_task_status(task_id, TaskState::Completed, Some(response_message))
            .await?;
        //println!("Task Update : {:#?}",task.clone());

        // for testing purpose
        //let task_retrieved=self.get_task(task_id, Some(10)).await?;
        //println!("Task Retrieved : {:#?}",task_retrieved);

        // to do : find a way to dump storage

        // this is where you need to implement a loop and have a way to identify if the run is final
        // Determine if this is a completed response or requires input
        //let task_state = if response.contains("TTTTT") {
        //    TaskState::InputRequired
        //} else {
        //    TaskState::Completed
        //};

        Ok(task)
    }
}

// below are all default boilerplate

#[async_trait]
impl AsyncTaskManager for SimpleAgentHandler {
    async fn create_task<'a>(
        &self,
        task_id: &'a str,
        context_id: &'a str,
    ) -> Result<Task, A2AError> {
        self.storage.create_task(task_id, context_id).await
    }

    async fn get_task<'a>(
        &self,
        task_id: &'a str,
        history_length: Option<u32>,
    ) -> Result<Task, A2AError> {
        self.storage.get_task(task_id, history_length).await
    }

    async fn update_task_status<'a>(
        &self,
        task_id: &'a str,
        state: TaskState,
        message: Option<Message>,
    ) -> Result<Task, A2AError> {
        self.storage
            .update_task_status(task_id, state, message)
            .await
    }

    async fn cancel_task<'a>(&self, task_id: &'a str) -> Result<Task, A2AError> {
        self.storage.cancel_task(task_id).await
    }

    async fn task_exists<'a>(&self, task_id: &'a str) -> Result<bool, A2AError> {
        self.storage.task_exists(task_id).await
    }
}

#[async_trait]
impl AsyncNotificationManager for SimpleAgentHandler {
    async fn set_task_notification<'a>(
        &self,
        config: &'a TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.storage.set_task_notification(config).await
    }

    async fn get_task_notification<'a>(
        &self,
        task_id: &'a str,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.storage.get_task_notification(task_id).await
    }

    async fn remove_task_notification<'a>(&self, task_id: &'a str) -> Result<(), A2AError> {
        self.storage.remove_task_notification(task_id).await
    }
}

#[async_trait]
impl AsyncStreamingHandler for SimpleAgentHandler {
    async fn add_status_subscriber<'a>(
        &self,
        task_id: &'a str,
        subscriber: Box<dyn Subscriber<TaskStatusUpdateEvent> + Send + Sync>,
    ) -> Result<String, A2AError> {
        self.storage
            .add_status_subscriber(task_id, subscriber)
            .await
    }

    async fn add_artifact_subscriber<'a>(
        &self,
        task_id: &'a str,
        subscriber: Box<dyn Subscriber<TaskArtifactUpdateEvent> + Send + Sync>,
    ) -> Result<String, A2AError> {
        self.storage
            .add_artifact_subscriber(task_id, subscriber)
            .await
    }

    async fn remove_subscription<'a>(&self, subscription_id: &'a str) -> Result<(), A2AError> {
        self.storage.remove_subscription(subscription_id).await
    }

    async fn remove_task_subscribers<'a>(&self, task_id: &'a str) -> Result<(), A2AError> {
        self.storage.remove_task_subscribers(task_id).await
    }

    async fn get_subscriber_count<'a>(&self, task_id: &'a str) -> Result<usize, A2AError> {
        self.storage.get_subscriber_count(task_id).await
    }

    async fn broadcast_status_update<'a>(
        &self,
        task_id: &'a str,
        update: TaskStatusUpdateEvent,
    ) -> Result<(), A2AError> {
        self.storage.broadcast_status_update(task_id, update).await
    }

    async fn broadcast_artifact_update<'a>(
        &self,
        task_id: &'a str,
        update: TaskArtifactUpdateEvent,
    ) -> Result<(), A2AError> {
        self.storage
            .broadcast_artifact_update(task_id, update)
            .await
    }

    async fn status_update_stream<'a>(
        &self,
        task_id: &'a str,
    ) -> Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Result<TaskStatusUpdateEvent, A2AError>> + Send>,
        >,
        A2AError,
    > {
        self.storage.status_update_stream(task_id).await
    }

    async fn artifact_update_stream<'a>(
        &self,
        task_id: &'a str,
    ) -> Result<
        std::pin::Pin<
            Box<dyn futures::Stream<Item = Result<TaskArtifactUpdateEvent, A2AError>> + Send>,
        >,
        A2AError,
    > {
        self.storage.artifact_update_stream(task_id).await
    }

    async fn combined_update_stream<'a>(
        &self,
        task_id: &'a str,
    ) -> Result<
        std::pin::Pin<
            Box<
                dyn futures::Stream<
                        Item = Result<a2a_rs::port::streaming_handler::UpdateEvent, A2AError>,
                    > + Send,
            >,
        >,
        A2AError,
    > {
        self.storage.combined_update_stream(task_id).await
    }
}
