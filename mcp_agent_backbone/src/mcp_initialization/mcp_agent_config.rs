use reqwest::Client;

use dotenv::dotenv;

use std::sync::Arc;

use llm_api::chat::Message;
use llm_api::tools::Tool;

use configuration::AgentMcpConfig;

use crate::mcp_client::mcp_client::get_tools_list_v2;
use crate::mcp_client::mcp_client::initialize_mcp_client_v2;
use crate::mcp_tools::tools::define_all_tools;

use rmcp::RoleClient;
use rmcp::model::InitializeRequestParam;
use rmcp::service::RunningService;

#[derive(Debug, Clone)]
pub struct RuntimeMcpConfigProject {
    pub http_client: Client,
    pub mcp_client: Arc<RunningService<RoleClient, InitializeRequestParam>>,
    pub init_messages: Vec<Message>,
    pub llm_all_tool: Vec<Tool>,
    pub model_id: String,
}

// https://github.com/modelcontextprotocol/rust-sdk/blob/main/examples/simple-chat-client/src/chat.rs
// make some tests to see if this can handle recursive calls

// Define available tools
pub async fn setup_project_mcp(
    agent_mcp_config: AgentMcpConfig,
) -> anyhow::Result<RuntimeMcpConfigProject> {
    // Load .env file if it exists
    dotenv().ok();

    // Set model to be used
    let model_id = agent_mcp_config.agent_mcp_model_id.clone();

    // Set model to be used
    let system_message = agent_mcp_config.agent_mcp_system_message.clone();

    // Initial configuration
    // Activate tracing and define mcp_server_url
    let mcp_client = Arc::new(initialize_mcp_client_v2(agent_mcp_config).await?);

    // connect mcp_server and retrieve tools
    let list_tools = get_tools_list_v2(mcp_client.clone()).await?;

    // Hard coded tool for testing purpose
    let llm_all_tool = define_all_tools(list_tools)?;

    // Conversation history
    let init_messages = vec![Message {
        role: "system".to_string(),
        content: system_message,
        tool_call_id: None,
    }];

    // Create a reqwest client
    let http_client = Client::new();

    Ok(RuntimeMcpConfigProject {
        http_client: http_client,
        mcp_client: mcp_client,
        init_messages: init_messages,
        llm_all_tool: llm_all_tool,
        model_id: model_id,
    })
}
