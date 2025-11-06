use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, warn, info};
use serde_json::{Map,Value, json, from_value};
use anyhow::{anyhow, Context};

use agent_core::agent_interaction_protocol::agent_interaction::AgentInteraction;
use agent_core::agent_interaction_protocol::a2a_agent_interaction::A2AAgentInteraction;

use agent_core::business_logic::services::{EvaluationService, MemoryService, DiscoveryService};
use configuration::{AgentReference, McpRuntimeConfig};

use rmcp::model::CallToolRequestParam;
use mcp_runtime::runtime::mcp_runtime::{McpRuntime};

// Re-export the traits from workflow_management for convenience
pub use workflow_management::agent_communication::agent_invoker::AgentInvoker;
pub use workflow_management::tasks::task_invoker::TaskInvoker;
pub use workflow_management::tools::tool_invoker::ToolInvoker;


use rmcp::model::{ListToolsResult, Tool as RmcpTool}; // Alias for clarity
use llm_api::tools::{FunctionDefinition, FunctionParameters, Tool};


/// An AgentInvoker that communicates using the A2A protocol over HTTP.
#[allow(dead_code)]
pub struct A2AAgentInvoker {
    agents_references: Vec<AgentReference>,
    client_agents: HashMap<String, A2AAgentInteraction>,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    memory_service: Option<Arc<dyn MemoryService>>,
    discovery_service_client: Arc<dyn DiscoveryService>,
}

#[async_trait]
impl AgentInvoker for A2AAgentInvoker {
    /// This function is called by the workflow_runtime when an activity is delegated to an agent in order to execute an activity.
    #[allow(unused_variables)]
    async fn interact(&self, agent_id: String, message:  String, skill_to_use: String ) -> anyhow::Result<Value> {
       
        let agent_client = self
            .client_agents
            .get(&agent_id)
            .ok_or(anyhow!("Agent \'{}\' not found", agent_id))?;

        // execute the task by remote agent
        let outcome = agent_client.execute_task(&message, "default_skill").await?;
        
        debug!("A2AAgentInvoker : {}", outcome);

        Ok(serde_json::Value::String(outcome))
    }
}

impl A2AAgentInvoker {
    /// This function instantiate an A2AAgentInvoker 
    pub async fn new(
        agents_references: Vec<AgentReference>,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service_client: Arc<dyn DiscoveryService>,
    ) -> anyhow::Result<Self> {
        
        let client_agents = Self::connect_to_a2a_agents(&agents_references).await?;

        Ok(Self {
            agents_references,
            client_agents,
            evaluation_service,
            memory_service,
            discovery_service_client,
        })
    }

     /// Instantiates an A2AAgentInvoker by dynamically discovering agents.
    /// This version fetches agent definitions from the `discovery_service_client`
    /// and uses the `agent_endpoint` directly from the `AgentDefinition`.
    pub async fn new_with_discovery(
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
        discovery_service_client: Arc<dyn DiscoveryService>,
    ) -> anyhow::Result<Self> {

        // 1. Discover agent definitions using the DiscoveryService trait method
        let agent_definitions = discovery_service_client.discover_agents().await?;
        info!("Discovered {} agent definitions.", agent_definitions.len());

        // 2. Convert AgentDefinition to AgentReference
        let mut agents_references: Vec<AgentReference> = Vec::new();
        for agent_def in agent_definitions {
            info!("Found agent '{}' at {}", agent_def.name, agent_def.agent_endpoint);
            agents_references.push(AgentReference {
                //name: agent_def.name.clone(),
                name: agent_def.id.clone(),
                url: agent_def.agent_endpoint.clone(),
                is_default: None, // AgentDefinition does not provide 'is_default'
            });
        }

        info!("Converted to {} agent references.", agents_references.len());

        // 3. Connect to the A2A agents using the dynamically created references
        let client_agents = Self::connect_to_a2a_agents(&agents_references).await?;

        Ok(Self {
            agents_references,
            client_agents,
            evaluation_service,
            memory_service,
            discovery_service_client,
        })
    }

    /// This function retrieves a list of clients agents , the list of agents that are referenced
    async fn connect_to_a2a_agents(
        agents_references: &[AgentReference],
    ) -> anyhow::Result<HashMap<String, A2AAgentInteraction>> {
        let mut client_agents = HashMap::new();

        debug!("Connecting to A2A server agents...");
        for agent_reference in agents_references {
            let agent_details = agent_reference.get_agent_reference().await?;

            debug!(
                "Connecting to agent \'{}\' at {}",
                agent_details.name, agent_details.url
            );

            match A2AAgentInteraction::new(agent_details.name.clone(), agent_details.url.clone())
                .await
            {
                Ok(client) => {
                    debug!(
                        "Successfully connected to agent \'{}\' at {}",
                        client.id, client.uri
                    );
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    debug!(
                        "Warning: Failed to connect to A2A agent \'{}\' at {}: {}",
                        agent_details.name, agent_details.url, e
                    );
                }
            }
        }

        if client_agents.is_empty() && !agents_references.is_empty() {
            warn!(
                "Warning: No A2A server agents connected, planner capabilities will be limited."
            );
        }
        Ok(client_agents)
    }

    #[allow(dead_code)]
    pub async fn find_agent_with_skill(&self, skill: &str, _task_id: &str) -> Option<&A2AAgentInteraction> {

        // 1. Try to find the agent with appropriate skill 
        for (agent_id, agent) in &self.client_agents {
            info!("WorkFlow Management: agent_id : \'{}\' with skill \'{}\'",agent_id, skill);
            // Access skills directly from the A2AClient struct
            if agent.has_skill(skill) {
                // Use the has_skill method
                info!("WorkFlow Management: Found agent \'{}\' with skill \'{}\'",agent_id, skill);
                return Some(agent);
            }
        }

         // 2. If no agent with the specific skill is found, try to find the default agent
         warn!("WorkFlow Management: No agent found with skill \'{}\' . Attempting to find default agent.", skill);

         for agent_ref_config in &self.agents_references {
             if agent_ref_config.is_default == Some(true) {
                 // We need to find the A2AClient instance associated with this default SimpleAgentReference
                 // We can do this by matching the name or ID. Assuming client.id is agent_reference.name
                 if let Some(default_agent_client) = self.client_agents.get(&agent_ref_config.name) {
                     info!(
                         "WorkFlow Management: Found default agent \'{}\' as fallback.",
                         default_agent_client.id
                     );
                     return Some(default_agent_client);
                 }
             }
         }
 
         // 3. If no agent with the skill and no default agent are found
         warn!("WorkFlow Management: No suitable agent (skill-matching or default) found for skill \'{}\'", skill);
         None
    }
}


pub struct GreetTask;

#[async_trait]
impl TaskInvoker for GreetTask {
    #[allow(unused_variables)]
    async fn invoke(
        &self,
        task_id: String, 
        params: &Value
    ) -> anyhow::Result<Value> {
        let name = params.get("name").and_then(|value| value.as_str()).unwrap_or("World");
        Ok(json!({"response":format!("Hello, {}!", name)}))
    }
}

impl GreetTask {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self{})
    }
}

pub struct  McpRuntimeToolInvoker  {
    mcp_runtime: Arc<McpRuntime>, // Your client for communicating with the MCP runtime
}

impl McpRuntimeToolInvoker  {
    pub async fn new(mcp_config_path: String) -> anyhow::Result<Self> {
        let mcp_runtime = Arc::new(Self::initialize_mcp_agent(mcp_config_path).await?);
        Ok(Self { mcp_runtime  })
    }

    pub async fn initialize_mcp_agent(mcp_config_path: String) -> anyhow::Result<McpRuntime> {
        let agent_mcp_config = McpRuntimeConfig::load_agent_config(mcp_config_path.as_str())
            .context("Error loading MCP config for planner")?;
        let mcp_runtime = McpRuntime::initialize_mcp_client_v2(agent_mcp_config).await?;
        Ok(mcp_runtime)
    }

    pub async fn get_tools_list_v2(&self) -> anyhow::Result<Vec<Tool>> {
        let list_tools: ListToolsResult = self.mcp_runtime.get_client()?.list_tools(Default::default()).await?;
        let list_tools:Vec<RmcpTool> = list_tools.tools;
        let tools=McpRuntimeToolInvoker::transcode_tools(list_tools);
        Ok(tools?)
    }

    pub fn transcode_tools(rmcp_tools: Vec<RmcpTool>) -> anyhow::Result<Vec<Tool>> {
        rmcp_tools
            .into_iter()
            .map(|tool| {
                let tool_name = tool.name.to_string(); // Get name early for potential error context
                let description = tool
                    .description
                    .ok_or_else(|| {
                        anyhow::anyhow!("Tool description is missing for tool '{}'", tool_name)
                    })?
                    .to_string(); // Convert Arc<str> to String
    
                // Clone the input schema map directly
                let properties_map: Map<String, Value> = tool.input_schema.as_ref().clone();
    
                let properties = properties_map.get("properties");
                //println!("Properties : {:#?}", properties);
    
                Ok(Tool {
                    r#type: "function".to_string(),
                    function: FunctionDefinition {
                        name: tool_name, // Use owned name
                        description,
                        parameters: FunctionParameters {
                            r#type: "object".to_string(),
                            properties: properties
                                .cloned()
                                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
                            required: None, // Keep as None for now
                        },
                    },
                })
            })
            .collect::<anyhow::Result<Vec<Tool>>>()
            .with_context(|| "Failed to define tools from rmcp::model::Tool vector")
    }
}

#[async_trait]
impl ToolInvoker for McpRuntimeToolInvoker  {
    async fn invoke(&self, tool_id:String,params: &Value) -> anyhow::Result<serde_json::Value>  {
        let arguments_map = from_value(params.clone())?;

        let call_tool_request_param = CallToolRequestParam {
            name: tool_id.into(),
            arguments: Some(arguments_map),
        };

        let tool_result = self.mcp_runtime.get_client()?.call_tool(call_tool_request_param).await?;
        
        let tool_result_value = serde_json::to_value(&tool_result.content)?;
        Ok(tool_result_value)
    }
}
