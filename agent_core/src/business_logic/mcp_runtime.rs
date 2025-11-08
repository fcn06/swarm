use configuration::McpRuntimeConfig;

#[derive(Debug, Clone)]
pub struct McpRuntimeDetails {
    pub config: McpRuntimeConfig,
    pub api_key: String,
}
