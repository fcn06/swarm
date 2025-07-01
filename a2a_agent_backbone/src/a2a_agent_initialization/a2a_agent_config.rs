use configuration::{AgentA2aConfig, AgentMcpConfig};
use dotenv::dotenv;
use mcp_agent_backbone::mcp_initialization::mcp_agent_config::{
    RuntimeMcpConfigProject, setup_project_mcp,
};

#[derive(Debug, Clone)]
pub struct RuntimeA2aConfigProject {
    pub agent_a2a_config: AgentA2aConfig,
    pub agent_mcp_config: Option<AgentMcpConfig>,
    pub mcp_runtime_config: Option<RuntimeMcpConfigProject>,
}

// Define available tools
pub async fn setup_project_a2a(
    agent_a2a_config: AgentA2aConfig,
) -> anyhow::Result<RuntimeA2aConfigProject> {
    // Load .env file if it exists
    dotenv().ok();

    // load mcp config agent if it exists
    let mcp_config_agent = match agent_a2a_config.agent_a2a_mcp_config_path.clone() {
        None => None,
        Some(path) => Some(
            AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading agent config"),
        ),
    };

    let mcp_runtime_config = match mcp_config_agent.clone() {
        None => None,
        Some(mcp_config_agent) => Some(setup_project_mcp(mcp_config_agent).await?),
    };

    Ok(RuntimeA2aConfigProject {
        agent_a2a_config: agent_a2a_config,
        agent_mcp_config: mcp_config_agent,
        mcp_runtime_config: mcp_runtime_config,
    })
}

