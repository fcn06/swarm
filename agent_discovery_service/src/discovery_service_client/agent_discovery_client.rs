
use reqwest::{Client, Error};



use a2a_rs::domain::AgentCard;

#[derive(Debug)]
pub struct AgentDiscoveryServiceClient {
    discovery_service_url: String,
    client: Client,
}

impl AgentDiscoveryServiceClient {
    pub fn new(discovery_service_url: String) -> Self {
        AgentDiscoveryServiceClient {
            discovery_service_url,
            client: Client::new(),
        }
    }

    pub async fn register(&self, agent_card: AgentCard) -> Result<String, Error> {

        let url = format!("{}/register", self.discovery_service_url);
        
        let response = self.client.post(&url)
            .json(&agent_card)
            .send()
            .await?;

        response.json::<String>().await
    }


    pub async fn list_agents(&self) -> Result<Vec<AgentCard>, Error> {
        let url = format!("{}/agents", self.discovery_service_url);
        let response = self.client.get(&url)
            .send()
            .await?;

        response.json::<Vec<AgentCard>>().await

    }


    // List all the agents except the one with mentioned name
    pub async fn list_agents_v2(&self, agent_name_to_filter_out:String) -> Result<Vec<AgentCard>, Error> {
        let url = format!("{}/agents", self.discovery_service_url);
        let response = self.client.get(&url)
            .send()
            .await?;

        let list_agents = response.json::<Vec<AgentCard>>().await?;

        let filtered_agents: Vec<AgentCard> = list_agents
            .into_iter()
            .filter(|agent_card| agent_card.name != agent_name_to_filter_out)
            .collect();

        Ok(filtered_agents)
    }


}


/*

/// Card describing an agent's capabilities, metadata, and available skills.\n///\n/// The AgentCard is the primary descriptor for an agent, containing all the\n/// information needed for clients to understand what the agent can do and\n/// how to interact with it. This includes basic metadata like name and version,\n/// capabilities like streaming support, available skills, and security requirements.\n///\n/// # Example\n/// ```rust\n/// use a2a_rs::{AgentCard, AgentCapabilities, AgentSkill};\n/// \n/// let card = AgentCard::builder()\n///     .name(\"My Agent\".to_string())\n///     .description(\"A helpful AI agent\".to_string())\n///     .url(\"https://agent.example.com\".to_string())\n///     .version(\"1.0.0\".to_string())\n///     .capabilities(AgentCapabilities::default())\n///     .build();\n/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "documentationUrl")]
    pub documentation_url: Option<String>,
    pub capabilities: AgentCapabilities,
    #[serde(skip_serializing_if = "Option::is_none", rename = "securitySchemes")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    #[serde(default = "default_input_modes", rename = "defaultInputModes")]
    pub default_input_modes: Vec<String>,
    #[serde(default = "default_output_modes", rename = "defaultOutputModes")]
    pub default_output_modes: Vec<String>,
    pub skills: Vec<AgentSkill>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "supportsAuthenticatedExtendedCard"
    )]
    pub supports_authenticated_extended_card: Option<bool>,
}

*/