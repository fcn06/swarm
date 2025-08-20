




/// Agent that that can interact with other available agents, and also embed MCP runtime if needed
#[derive(Clone)]
pub struct WorkflowAgent {
    agent_config: Arc<AgentConfig>, // Use Arc for cheaper cloning
    agents_references: Vec<AgentReference>,
    llm_interaction: ChatLlmInteraction,
    evaluation_service: Option<Arc<dyn EvaluationService>>,
    memory_service: Option<Arc<dyn MemoryService>>,
    discovery_service_client: Arc<AgentDiscoveryServiceClient>,
    workflow_registries : Arc<WorkflowRegistries>,
}



#[async_trait]
impl Agent for WorkFlowAgent {

    async fn new(
        agent_config: AgentConfig,
        evaluation_service: Option<Arc<dyn EvaluationService>>,
        memory_service: Option<Arc<dyn MemoryService>>,
    ) -> anyhow::Result<Self> {

        // Set model to be used
        let model_id = agent_config.agent_model_id();

        // Set llm_url to be used
        let llm_url =  agent_config.agent_llm_url();

        // Set API key for LLM
        let llm_full_api_key = env::var("LLM_FULL_API_KEY").expect("LLM_FULL_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
        llm_url,
        model_id,
        llm_full_api_key,
        );

        // List available agents from config
        let agents_references =  agent_config.agent_agents_references().unwrap_or_default();
       
        // Get all clients for each agent defined in config
        let client_agents = Self::connect_to_a2a_agents(&agents_references).await?;
  
          // Load MCP agent if specified in planner config
          let mcp_agent = Self::initialize_mcp_agent(&agent_config).await?;
  
          Ok(Self {
            agent_config: Arc::new(agent_config),
            agents_references,
            llm_interaction,
            client_agents,
            mcp_agent,
            evaluation_service,
            memory_service,
          })

    }

    async fn handle_request(&self, request: LlmMessage) ->anyhow::Result<ExecutionResult> { 
    
        let request_id = Uuid::new_v4().to_string();
        let conversation_id = Uuid::new_v4().to_string();
        let user_query = request.content.clone().unwrap_or_default();
        info!("---Full: Starting to handle user request --  Query: '{:?}'---",user_query);

        match self.process_plan_creation(user_query.clone(), &request_id).await {
            Ok(mut plan) => {
                let execution_outcome = self.execute_and_summarize_plan(&mut plan, &request_id, &conversation_id, &user_query).await;
                
                // Log evaluation and memory data asynchronously
                self.log_evaluation_data(&request_id, &user_query, &execution_outcome).await;
                self.log_memory_data(&conversation_id, &user_query, &plan, &execution_outcome).await;

                execution_outcome
            },
            Err(e) => {
                let error_msg = format!(
                    "FullAgent: Failed to create plan for request ID {}: {}",
                    request_id, e
                );
                trace!("{}", error_msg);
                Ok(ExecutionResult {
                    request_id,
                    conversation_id,
                    success: false,
                    output: error_msg,
                    plan_details: None,
                })
            }
        }
    

    }
}