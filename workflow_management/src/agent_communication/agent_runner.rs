use agent_protocol_backbone::planning::plan_definition::TaskDefinition;
use async_trait::async_trait;

#[async_trait]
pub trait AgentRunner: Send + Sync {
    /// A unique name for the runner, used for registration.
    fn name(&self) -> String;

    /// Invokes the agent.
    ///
    /// This method is responsible for:
    /// 1. Discovering the agent's endpoint.
    /// 2. Formatting the request according to the A2A protocol.
    /// 3. Sending the request.
    /// 4. Parsing the response and returning the result.
    async fn invoke(
        &self,
        task: &TaskDefinition,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
