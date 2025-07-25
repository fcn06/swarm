use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

// Assuming llm_api crate is available and has these
use llm_api::chat::{ChatLlmInteraction};

use crate::a2a_plan::plan_definition::{ExecutionResult, Plan, PlanResponse, PlanStatus};
use crate::a2a_plan::plan_execution::A2AClient;

use a2a_rs::domain::{Message, Part, TaskState, AgentCard};

use std::env;
use configuration::{AgentPlannerConfig, AgentMcpConfig};
use mcp_agent_backbone::mcp_agent_logic::agent::McpAgent;

use llm_api::tools::Tool;
use configuration::SimpleAgentReference;

use rmcp::model::{CallToolRequestParam, CallToolResult, Annotated, RawContent};

use tracing::{error,warn,info,debug,trace};

/// Agent that will be in charge of the planning definition and execution
/// He will have access to various a2a resources for this purpose
#[derive(Clone)]
pub struct PlannerAgent {
    planner_agent_definition: PlannerAgentDefinition,
    llm_interaction: ChatLlmInteraction,
    client_agents: HashMap<String, A2AClient>,
    mcp_agent: Option<McpAgent>,
}


#[derive(Clone)]
pub struct PlannerAgentDefinition {
    pub agent_configs: Vec<SimpleAgentReference>, // Info to connect to agents
}


impl PlannerAgent {
    pub async fn new(
        agent_planner_config: AgentPlannerConfig) -> Result<Self> {

        // Set model to be used
        let model_id = agent_planner_config.agent_planner_model_id.clone();
        // Set llm_url to be used
        let llm_url = agent_planner_config.agent_planner_llm_url.clone();

        // Set API key for LLM
        let llm_planner_api_key = env::var("LLM_BIDIRECTIONAL_API_KEY").expect("LLM_BIDIRECTIONAL_API_KEY must be set");

        let llm_interaction= ChatLlmInteraction::new(
        llm_url,
        model_id,
        llm_planner_api_key,
        );

        // Set model to be used
        let agents_references = agent_planner_config.agent_planner_agents_references.clone();

        let planner_agent_definition = PlannerAgentDefinition {
        agent_configs: agents_references,
        };

        let mut client_agents = HashMap::new();

        debug!("PlannerAgent: Connecting to A2a server agents...");

        // Query discovery service for available agents
        let discovery_url = agent_planner_config.agent_planner_discovery_url.clone().expect("Discovery URL must be set for PlannerAgent");
        let agents_from_discovery = Self::list_registered_agents(discovery_url.as_str()).await?;

        for agent_card in agents_from_discovery {
            debug!(
                "PlannerAgent: Connecting to discovered agent '{}' at {}",
                agent_card.name, agent_card.url
            );

            match A2AClient::connect(agent_card.name.clone(), agent_card.url.clone())
                .await
            {
                Ok(client) => {
                    debug!(
                        "PlannerAgent: Successfully connected to agent '{}' at {}",
                        client.id, client.uri
                    );
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    debug!(
                        "PlannerAgent: Warning: Failed to connect to discovered A2a agent '{}' at {}: {}",
                        agent_card.name, agent_card.url, e
                    );
                }
            }
        }

        if client_agents.is_empty() && !planner_agent_definition.agent_configs.is_empty() {
            warn!(
                "PlannerAgent: Warning: No A2a server agents connected, planner capabilities will be limited to direct LLM interaction if any."
            );
        }

        // Load MCP agent if specified in planner config
        let mcp_agent = match agent_planner_config.agent_planner_mcp_config_path.clone() {
            None => None,
            Some(path) => {
                let agent_mcp_config = AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading MCP config for planner");
                let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                Some(mcp_agent)
            },
        };

        Ok(Self {
            planner_agent_definition,
            llm_interaction,
            client_agents,
            mcp_agent,
        })
    }

    async fn list_registered_agents(discovery_url: &str) -> Result<Vec<AgentCard>> {
        let list_uri = format!("{}/list", discovery_url);
        let response = reqwest::Client::new()
            .get(list_uri)
            .send()
            .await?
            .json::<Vec<AgentCard>>()
            .await?;
        Ok(response)
    }

    async fn get_available_skills_and_tools_description(&self) -> String {
        let mut description = "Available agent skills: 
".to_string();
        if self.client_agents.is_empty() {
            description.push_str("- No A2a agents connected.
");
        } else {
            for (name, agent) in &self.client_agents {
                description.push_str(&format!("* Agent_id : '{}' -- ", name));
                let skills = agent.get_skills();
                
                if skills.is_empty() {
                    description.push_str(" No specific skills listed.
");
                } else {
                    for skill in skills {
                        description.push_str(&format!(" skill.id : '{}' -- skill.description : '{}' 
", skill.id,skill.description.clone()));
                    }
                }
            }
        }

        // Add MCP tools description if MCP agent is present
        if let Some(mcp) = &self.mcp_agent {
            description.push_str("
Available MCP Tools: 
");
            let tools = mcp.get_available_tools();
            if tools.is_empty() {
                description.push_str("- No MCP tools available.
");
            } else {
                for tool in tools {
                    description.push_str(&format!("* Tool Name: '{}' -- Description: '{}' -- Arguments: '{}'
", tool.function.name, tool.function.description, serde_json::to_string(&tool.function.parameters).unwrap_or_else(|_| "{}".to_string())));
                }
            }
        }
        
        description
    }

    pub async fn handle_user_request(&mut self, user_request: Message) -> ExecutionResult {
        let request_id = Uuid::new_v4().to_string();

        // Extracting text from message
        let user_query = self.extract_text_from_message(&user_request).await;

        info!("---PlannerAgent: Starting to handle user request --  Query: '{}'---",user_query);

        match self.create_plan(&user_request).await {
            Ok(mut plan) => {
                trace!(
                    "PlannerAgent: Plan created successfully for request ID: {}. Plan ID: {}",
                    request_id, plan.id
                );

                // Attempt to execute the plan
                let _execution_outcome = self.execute_plan(&mut plan).await;

                // Attempt to summarize results regardless of execution outcome
                match self.summarize_results(&mut plan).await {
                    Ok(summary) => {
                        trace!(
                            "PlannerAgent: Final summary generated for request ID {}.",
                            request_id
                        );
                        ExecutionResult {
                            request_id,
                            success: plan.status == PlanStatus::Completed,
                            output: summary,
                            plan_details: Some(plan),
                        }
                    }
                    Err(e) => {
                        trace!(
                            "PlannerAgent: Failed to summarize results for request ID {}: {}",
                            request_id, e
                        );
                        let output_on_summary_fail = format!(
                            "Plan processing finished with status {:?}, but summarization failed: {}",
                            plan.status, e
                        );
                        ExecutionResult {
                            request_id,
                            success: false, // Mark as not fully successful if summarization fails
                            output: output_on_summary_fail,
                            plan_details: Some(plan),
                        }
                    }
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "PlannerAgent: Failed to create plan for request ID {}: {}",
                    request_id, e
                );
                trace!("{}", error_msg);
                ExecutionResult {
                    request_id,
                    success: false,
                    output: error_msg,
                    plan_details: None,
                }
            }
        }
    }

    async fn create_plan(&self, request: &Message) -> Result<Plan> {
        info!(
            "PlannerAgent: Creating plan for request ID: {}",
            Uuid::new_v4().to_string()
        ); 

        let skills_and_tools_description = self.get_available_skills_and_tools_description().await;

        debug!("{}",skills_and_tools_description );

        let prompt = format!(
            "You are a planner agent that creates execution plans for user requests.

            You have access to the following agent skills and MCP tools:
            {}

            User request: {}

            Based on the user request and available skills and tools, create a step-by-step plan to fulfill it.

            The plan should be a JSON object with 'plan_summary' (a brief description of the overall plan) and 'tasks' (an array of task objects).

            Each task object must have the following fields:

            - 'id': A unique string ID for the task (e.g., 'task_1', 'task_web_search').

            - 'description': A clear, concise description of what the task should achieve.

            - 'skill_to_use': (Optional) The specific skill ID required from an A2A agent (e.g., 'skill_search_web', 'skill_calculate'). If a tool is to be used, this should be null.

            - 'tool_to_use': (Optional) The name of the specific MCP tool to use (e.g., 'search_tool', 'calculator'). If a skill is to be used, this should be null.

            - 'assigned_agent_id_preference': (Optional) If a specific skill is mentioned, suggest the ID of an agent that provides this skill (e.g., 'agent_search'). This is just a preference, the executor will find a suitable agent.

            - 'tool_parameters': (Optional) If a tool is to be used, a JSON object containing the parameters for the tool call. Example: {{ \"query\": \"weather in London\" }}.

            - 'dependencies': (Optional) An array of task IDs that must be completed before this task can start. If no dependencies, use an empty array or omit.

            - 'expected_outcome': (Optional) A brief description of the expected result of the task.

            Example Plan:

            {{
              \"plan_summary\": \"Search for information and summarize.\",
              \"tasks\": [
                {{
                  \"id\": \"search_web\",
                  \"description\": \"Search the web for information about the user request.\",
                  \"skill_to_use\": \"skill_search_web\",
                  \"tool_to_use\": null,
                  \"assigned_agent_id_preference\": null,
                  \"tool_parameters\": null,
                  \"dependencies\": [],
                  \"expected_outcome\": \"Relevant search results.\"
                }},
                {{
                  \"id\": \"calculate_sum\",
                  \"description\": \"Calculate the sum of two numbers.\",
                  \"skill_to_use\": null,
                  \"tool_to_use\": \"calculator\",
                  \"assigned_agent_id_preference\": null,
                  \"tool_parameters\": {{
                    \"a\": 10,
                    \"b\": 20
                  }},                  
                  \"dependencies\": [],
                  \"expected_outcome\": \"The sum of the numbers.\"
                }},
                {{
                  \"id\": \"summarize_info\",
                  \"description\": \"Summarize the information found from the web search.\",
                  \"skill_to_use\": null,
                  \"tool_to_use\": null,
                  \"assigned_agent_id_preference\": null,
                  \"tool_parameters\": null,
                  \"dependencies\": [\"search_web\"],
                  \"expected_outcome\": \"A concise summary.\"
                }}]
            }}

            RETURN ONLY THE SIMPLE JSON REPRESENTING THE PLAN ON THE SAME FORMAT AS ABOVE.",

            skills_and_tools_description, self.extract_text_from_message(request).await
        );

        // This api returns raw text from llm
        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(),prompt.to_string()).await?;

        info!(
            "PlannerAgent: LLM responded with plan content:{:?}",
            response_content
        );

        let llm_plan_data: PlanResponse =
            match serde_json::from_str(&response_content.clone().expect("REASON")) {
                Ok(data) => data,
                Err(e) => {
                    warn!(
                        "PlannerAgent: Failed to parse LLM plan response as JSON: {}",
                        e
                    );
                    warn!("PlannerAgent: LLM Raw Response: {:?}", response_content);
                    bail!(
                        "LLM returned invalid plan format: {:?}. Raw: {:?}",
                        e,
                        response_content
                    );
                }
            };

        // Create the Plan struct from the parsed LLM response
        let plan = Plan::new(
            Uuid::new_v4().to_string(),
            self.extract_text_from_message(request).await,
            llm_plan_data.plan_summary,
            llm_plan_data.tasks,
        ); 

        Ok(plan)
    }

    // to be fine tuned and better tested
    async fn execute_plan(&mut self, plan: &mut Plan) -> Result<()> {
        trace!(
            "PlannerAgent: Starting plan execution for request ID: {}",
            plan.request_id
        );
        plan.status = PlanStatus::InProgress;
        plan.updated_at = Some(Utc::now());

        let mut completed_tasks: HashSet<String> = HashSet::new();
        let mut task_queue: VecDeque<String> = VecDeque::new(); // Tasks ready to execute
        let  _executing_plans: HashMap<String, String> = HashMap::new();

        // Initial population of the queue with tasks that have no dependencies
        for task_def in &plan.tasks_definition {
            if task_def.dependencies.is_empty() {
                task_queue.push_back(task_def.id.clone());
            }
        }

        // Process tasks
        while !task_queue.is_empty() {
            let task_id = task_queue.pop_front().unwrap();

            // Check if task is already completed 
            if completed_tasks.contains(&task_id) {
                continue; 
            }

            let task_def_index = plan
                .tasks_definition
                .iter()
                .position(|t| t.id == task_id)
                .context(format!("Task definition {} not found", task_id))?;
            let task_def = &plan.tasks_definition[task_def_index];

            // Check if all dependencies are met
            let dependencies_met = task_def
                .dependencies
                .iter()
                .all(|dep_id| completed_tasks.contains(dep_id));

            if dependencies_met {
                debug!(
                    "PlannerAgent: Submitting task '{}': {}",
                    task_id, task_def.description
                );

                // Construct task description with results of dependencies
                let mut full_task_description = task_def.description.clone();
                if !task_def.dependencies.is_empty() {
                    full_task_description.push_str("Context from previous tasks:
");
                    for dep_id in &task_def.dependencies {
                        if let Some(result) = plan.task_results.get(dep_id) {
                            full_task_description.push_str(&format!(
                                "- Result of task '{}': {}
",
                                dep_id, result
                            ));
                        }
                    }
                }

                let task_result: Result<String>;

                if let Some(skill) = &task_def.skill_to_use {
                    let agent_client = self.find_agent_with_skill(skill, &task_id).await;
                    match agent_client {
                        Some(client) => {
                            task_result = client
                                .execute_task(&full_task_description, skill)
                                .await
                                .map(|r| r);
                        }
                        None => {
                            task_result = Err(anyhow::anyhow!(
                                "No A2A agent found with skill '{}' for task '{}'",
                                skill,
                                task_id
                            ));
                        }
                    }
                } else if let Some(tool_name) = &task_def.tool_to_use {
                    if let Some(mcp) = &self.mcp_agent {
                        let tool_parameters = task_def.tool_parameters.clone().unwrap_or_default();
                        let arguments_map = if tool_parameters.is_object() {
                            Some(tool_parameters.as_object().unwrap().clone())
                        } else {
                            None
                        };
                        let call_tool_request_param = CallToolRequestParam { 
                            name: tool_name.to_string().into(), 
                            arguments: arguments_map,
                        };
                        task_result = mcp.mcp_client.call_tool(call_tool_request_param).await.map(|r: CallToolResult| {
                            r.content.into_iter().filter_map(|annotated_content| {
                                if let RawContent::Text(text) = annotated_content.raw {
                                    Some(text)
                                } else {
                                    None
                                }
                            }).collect::<Vec<String>>().join("")
                        }).map_err(|e| anyhow::anyhow!(e));
                    } else {
                        task_result = Err(anyhow::anyhow!(
                            "MCP agent not initialized, but tool '{}' was requested for task '{}'",
                            tool_name,
                            task_id
                        ));
                    }
                } else {
                    // Task requires no specific skill or tool, potentially an LLM reflection task
                    task_result = Ok(self.llm_interaction.call_api_simple_v2("user".to_string(),full_task_description.to_string()).await?.expect("Improper task description"));
                }

                // Process the task result immediately
                match task_result {
                    Ok(result_content) => {
                        
                        debug!(
                            "PlannerAgent: Task '{}' completed successfully. Result : {}",
                            task_id, result_content
                        );

                        completed_tasks.insert(task_id.clone());
                        plan.task_results
                            .insert(task_id.clone(), result_content.clone()); 

                        if let Some(task_def_mut) =
                            plan.tasks_definition.get_mut(task_def_index)
                        {
                            task_def_mut.task_output = Some(result_content);
                        }

                        // Add dependent tasks to the queue
                        for dep_task_def in &plan.tasks_definition {
                            if dep_task_def.dependencies.contains(&task_id) && !completed_tasks.contains(&dep_task_def.id) {
                                task_queue.push_back(dep_task_def.id.clone());
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Task '{}' failed: {}", task_id, e);
                        error!("PlannerAgent: {}", error_msg);
                        plan.status =
                            PlanStatus::Failed(format!("Execution failed at task {}", task_id));
                        plan.updated_at = Some(Utc::now());
                        return Err(anyhow::anyhow!(error_msg)); 
                    }
                }
            } else {
                // Dependencies not met, push back to queue for later
                task_queue.push_back(task_id.clone());
            }
        }

        let all_tasks_completed = completed_tasks.len() == plan.tasks_definition.len(); 

        if all_tasks_completed {
            plan.status = PlanStatus::Completed;
            plan.updated_at = Some(Utc::now());
            debug!(
                "PlannerAgent: Plan execution completed successfully for request ID: {}",
                plan.request_id
            );
        } else if matches!(plan.status, PlanStatus::InProgress) {
            let unfinished_tasks: Vec<_> = plan
                .tasks_definition
                .iter()
                .filter(|t| !completed_tasks.contains(&t.id))
                .map(|t| t.id.clone())
                .collect();
            let failure_reason = format!(
                "Plan execution finished, but not all tasks completed. Unfinished: {:?}",
                unfinished_tasks
            );
            warn!("PlannerAgent: {}", failure_reason);
            plan.status = PlanStatus::Failed(failure_reason);
            plan.updated_at = Some(Utc::now());
        }

        Ok(())
    }

    async fn find_agent_with_skill(&self, skill: &str, _task_id: &str) -> Option<&A2AClient> {

        for (agent_id, agent) in &self.client_agents {
            info!("PlannerAgent: agent_id : '{}' with skill '{}'.",agent_id, skill);
            if agent.has_skill(skill) {
                info!(
                    "PlannerAgent: Found agent '{}' with skill '{}'.",
                    agent_id, skill
                );
                return Some(agent);
            }
        }

         warn!("PlannerAgent: No agent found with skill '{}'.", skill);
         None
    }

    async fn summarize_results(&self, plan: &mut Plan) -> Result<String> {

        info!("PlannerAgent: Summarizing results for plan ID: {}", plan.id);
        let mut context = format!("User's initial request: {}
", plan.user_query);
        context.push_str(&format!(
            "Plan ID: {}
Overall Plan Summary by LLM: {}
Plan Status: {:?}
Tasks executed:
",
            plan.id, plan.plan_summary, plan.status
        ));

        let  _sorted_tasks_defs = plan.tasks_definition.clone();

        for task_def in &plan.tasks_definition {

            let task_execution_status = match plan.status {
                PlanStatus::Completed => TaskState::Completed, 
                PlanStatus::Failed(_) => TaskState::Failed, 
                _ => TaskState::Working, 
            };

            context.push_str(&format!(
                "- Task ID: {}, Description: {}, Status: {:?}, Skill: {:?}, Tool: {:?}",
                task_def.id,
                task_def.description,
                task_execution_status,
                task_def.skill_to_use.as_deref().unwrap_or("N/A"),
                task_def.tool_to_use.as_deref().unwrap_or("N/A")
            ));

            if let Some(output) = plan.task_results.get(&task_def.id) {
                context.push_str(&format!(", Output: "{}"", output)); 
            }
        }

        if plan.status == PlanStatus::Completed {
            context.push_str("All tasks completed successfully. Please provide a concise summary of the overall outcome for the user based on the initial request and the plan summary.");
        } else if let PlanStatus::Failed(reason) = &plan.status {
            context.push_str(&format!("The plan failed. Reason: {}. Please provide a summary for the user of what was attempted and why it failed, based on the initial request and the plan details.", reason));
        } else {
            context.push_str("The plan is still in progress. Provide a brief update based on the plan summary and tasks.");
        }

        let summary = self.llm_interaction.call_api_simple_v2("user".to_string(),context.to_string()).await?.expect("Improper Summary");
        
        plan.final_summary = Some(summary.clone());
        plan.updated_at = Some(Utc::now());
        debug!("PlannerAgent: Summary generated.");

        Ok(summary)
    }

    // Helper function to extract text from a Message
    async fn extract_text_from_message(&self, message: &Message) -> String {
        message
            .parts
            .iter()
            .filter_map(|part| {
                if let Part::Text { text, metadata: _ } = part {
                    Some(text.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }


        pub async fn submit_user_text(&mut self, user_query: String) -> ExecutionResult {
            let message_id = Uuid::new_v4().to_string();
            let user_req= Message::user_text(user_query, message_id.clone());
            let execution_result = self.handle_user_request(user_req).await;
            execution_result
        }



}
