use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

// Assuming llm_api crate is available and has these
use llm_api::chat::{ChatLlmInteraction};

use crate::plan::plan_execution::A2AClient;

use rmcp::model::{CallToolRequestParam};


use a2a_rs::domain::{Message, Part, TaskState};


use tracing::{error,warn,info,debug,trace};

use configuration::AgentMcpConfig;

use mcp_runtime::mcp_agent_logic::agent::McpAgent;
use llm_api::chat::Message as LlmMessage;
use std::env;
use agent_protocol_backbone::business_logic::agent::{Agent};
use agent_protocol_backbone::config::agent_config::{AgentConfig,AgentReference};

use agent_protocol_backbone::planning::plan_definition::{
    ExecutionResult, Plan, PlanResponse, PlanStatus, 
};

use async_trait::async_trait;
use agent_evaluation_service::evaluation_service_client::agent_evaluation_client::AgentEvaluationServiceClient;
use agent_evaluation_service::evaluation_server::judge_agent::AgentLogData;


/// Agent that that can interact with other available agents, and also embed MCP runtime if needed
#[derive(Clone)]
pub struct OrchestrationAgent {
    agent_config: AgentConfig, // possible future use
    agents_references: Vec<AgentReference>,
    llm_interaction: ChatLlmInteraction,
    client_agents: HashMap<String, A2AClient>,
    mcp_agent: Option<McpAgent>,
}


#[async_trait]
impl Agent for OrchestrationAgent {

    async fn new(
        agent_config: AgentConfig) -> anyhow::Result<Self> {

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
        //todo:make it resilient
        let agents_references =  agent_config.agent_agents_references().unwrap();
        
        // todo:make this search dynamic
        
        // retrieve the agents available from config
          let mut client_agents = HashMap::new();
  
          debug!("Full Agent: Connecting to A2a server agents...");
          for agent_reference in &agents_references {
              // Use agent_info (which implements AgentInfoProvider) to get details for connection
              let agent_reference = agent_reference.get_agent_reference().await?;
  
              debug!(
                  "FullAgent: Connecting to agent '{}' at {}",
                  agent_reference.name, agent_reference.url
              );
  
  
              match A2AClient::connect(agent_reference.name.clone(), agent_reference.url.clone())
                  .await
              {
                  Ok(client) => {
                      debug!(
                          "FullAgent: Successfully connected to agent '{}' at {}",
                          client.id, client.uri
                      );
                      // Use the connected client's ID as the key
                      client_agents.insert(client.id.clone(), client);
  
                  }
                  Err(e) => {
                      // Use details from agent_info for error reporting
                      debug!(
                          "FullAgent: Warning: Failed to connect to A2a agent '{}' at {}: {}",
                          agent_reference.name, agent_reference.url, e
                      );
                  }
              }
          }
  
          if client_agents.is_empty() && !agents_references.is_empty() {
              warn!(
                  "FullAgent: Warning: No A2a server agents connected, planner capabilities will be limited to direct LLM interaction if any."
              );
              // Depending on requirements, you might return an error here:
              // bail!("Critical: Failed to connect to any A2a server agents.");
          }
  
          // Load MCP agent if specified in planner config
          let mcp_agent = match agent_config.agent_mcp_config_path() {
              None => None,
              Some(path) => {
                  let agent_mcp_config = AgentMcpConfig::load_agent_config(path.as_str()).expect("Error loading MCP config for planner");
                  let mcp_agent = McpAgent::new(agent_mcp_config).await?;
                  Some(mcp_agent)
              },
          };
  
          Ok(Self {
            agent_config:agent_config,
            agents_references: agents_references,
            llm_interaction,
            client_agents,
            mcp_agent,
          })

    }

    async fn handle_request(&self, request: LlmMessage) ->anyhow::Result<ExecutionResult> {
    
        let request_id = Uuid::new_v4().to_string();

        // To be instantiated at launch
        let evaluation_service=AgentEvaluationServiceClient::new("http://127.0.0.1:7000".to_string());

        // Extracting text from message
        // todo:make resilient
        let user_query = request.content.unwrap();

        info!("---Full: Starting to handle user request --  Query: '{:?}'---",user_query);

        match self.create_plan(user_query.clone()).await {
            Ok(mut plan) => {
                trace!(
                    "FullAgent: Plan created successfully for request ID: {}. Plan ID: {}",
                    request_id, plan.id
                );

                // Attempt to execute the plan
                let _execution_outcome = self.execute_plan(&mut plan).await;

                // Attempt to summarize results regardless of execution outcome
                match self.summarize_results(&mut plan).await {
                    Ok(summary) => {
                        trace!(
                            "FullAgent: Final summary generated for request ID {}.",
                            request_id
                        );

                        // To be improved
                        evaluation_service.log_evaluation(AgentLogData {
                            agent_id: self.agent_config.agent_name().to_string(),
                            request_id: request_id.to_string(),
                            step_id: "".to_string(),
                            original_user_query: user_query.clone().to_string(),
                            agent_input: user_query.clone().to_string(),
                            agent_output: summary.to_string(),
                            context_snapshot: None,
                            success_criteria: None,
                        }).await?;


                        Ok(ExecutionResult {
                            request_id,
                            success: plan.status == PlanStatus::Completed,
                            output: summary,
                            plan_details: Some(plan),
                        })
                    }
                    Err(e) => {
                        trace!(
                            "FullAgent: Failed to summarize results for request ID {}: {}",
                            request_id, e
                        );
                        let output_on_summary_fail = format!(
                            "Plan processing finished with status {:?}, but summarization failed: {}",
                            plan.status, e
                        );
                        Ok(ExecutionResult {
                            request_id,
                            success: false, // Mark as not fully successful if summarization fails
                            output: output_on_summary_fail,
                            plan_details: Some(plan),
                        })
                    }
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "FullAgent: Failed to create plan for request ID {}: {}",
                    request_id, e
                );
                trace!("{}", error_msg);
                Ok(ExecutionResult {
                    request_id,
                    success: false,
                    output: error_msg,
                    plan_details: None,
                })
            }
        }
    }

}

impl OrchestrationAgent {

    

    async fn get_available_skills_and_tools_description(&self) -> String {
        let mut description = "Available agent skills: ".to_string();
        if self.client_agents.is_empty() {
            description.push_str("- No A2a agents connected.");
        } else {
            for (name, agent) in &self.client_agents {
                description.push_str(&format!("* Agent_id : '{}' -- ", name));
                let skills = agent.get_skills();
                
                if skills.is_empty() {
                    description.push_str(" No specific skills listed.");
                } else {
                    for skill in skills {
                        description.push_str(&format!(" skill.id : '{}' -- skill.description : '{}' ", skill.id,skill.description.clone()));
                    }
                }
            }
        }

        // Add MCP tools description if MCP agent is present
        if let Some(mcp) = &self.mcp_agent {
            description.push_str("Available MCP Tools: ");
            let tools = mcp.get_available_tools();
            if tools.is_empty() {
                description.push_str("- No MCP tools available.");
            } else {
                for tool in tools {
                    description.push_str(&format!("* Tool Name: '{}' -- Description: '{}' -- Arguments: '{}'", tool.function.name, tool.function.description, serde_json::to_string(&tool.function.parameters).unwrap_or_else(|_| "{}".to_string())));
                }
            }
        }
        
        description
    }

    async fn find_agent_with_skill(&self, skill: &str, _task_id: &str) -> Option<&A2AClient> {

        // 1. Try to find the agent with appropriate skill 
        for (agent_id, agent) in &self.client_agents {
            info!("FullAgent: agent_id : '{}' with skill '{}'.",agent_id, skill);
            // Access skills directly from the A2AClient struct
            if agent.has_skill(skill) {
                // Use the has_skill method
                info!("FullAgent: Found agent '{}' with skill '{}'.",agent_id, skill);
                return Some(agent);
            }
        }

         // 2. If no agent with the specific skill is found, try to find the default agent
         warn!("PlannerAgent: No agent found with skill '{}'. Attempting to find default agent.", skill);

         for agent_ref_config in &self.agents_references {
             if agent_ref_config.is_default == Some(true) {
                 // We need to find the A2AClient instance associated with this default SimpleAgentReference
                 // We can do this by matching the name or ID. Assuming client.id is agent_reference.name
                 if let Some(default_agent_client) = self.client_agents.get(&agent_ref_config.name) {
                     info!(
                         "FullAgent: Found default agent '{}' as fallback.",
                         default_agent_client.id
                     );
                     return Some(default_agent_client);
                 }
             }
         }
 
         // 3. If no agent with the skill and no default agent are found
         warn!("FullAgent: No suitable agent (skill-matching or default) found for skill '{}'.", skill);
         None
    }


    async fn create_plan(&self, user_request: String) -> Result<Plan> {
        info!(
            "PlannerAgent: Creating plan for request ID: {}",
            Uuid::new_v4().to_string()
        ); 

        let skills_and_tools_description = self.get_available_skills_and_tools_description().await;

        debug!("{}",skills_and_tools_description );

        let prompt = format!(
            "You are a planner agent that creates execution plans for user requests.

            On top of your own skills, you have access to the following agent skills and MCP tools:
            {}

            As a last resort, if no specific skills or tools are applicable to the user's request, create a task that leverages your general knowledge to answer the user's query.


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

                  \"id\": \"general_knowledge_response\",

                  \"description\": \"Answer the user's query using general knowledge.\",

                  \"skill_to_use\": null,

                  \"tool_to_use\": null,

                  \"assigned_agent_id_preference\": null,

                  \"tool_parameters\": null,

                  \"dependencies\": [],

                  \"expected_outcome\": \"A direct answer to the user's query based on general knowledge.\"

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

            skills_and_tools_description, user_request
        );

        // This api returns raw text from llm
        let response_content = self.llm_interaction.call_api_simple_v2("user".to_string(),prompt.to_string()).await?;

        info!(
            "FullAgent: LLM responded with plan content:{:?}",
            response_content
        );

        let llm_plan_data: PlanResponse =
            match serde_json::from_str(&response_content.clone().expect("REASON")) {
                Ok(data) => data,
                Err(e) => {
                    warn!(
                        "FullAgent: Failed to parse LLM plan response as JSON: {}",e);
                    warn!("FullAgent: LLM Raw Response: {:?}", response_content);
                    bail!(
                        "LLM returned invalid plan format: {:?}. Raw: {:?}",e,response_content);
                }
            };

        // Create the Plan struct from the parsed LLM response
        let plan = Plan::new(
            Uuid::new_v4().to_string(),
            user_request,
            llm_plan_data.plan_summary,
            llm_plan_data.tasks,
        ); 

        Ok(plan)
    }

     // to be fine tuned and better tested
        async fn execute_plan(&self, plan: &mut Plan) -> Result<()> {
        trace!(
            "FullAgent: Starting plan execution for request ID: {}",plan.request_id);
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
                    "FullAgent: Submitting task '{}': {}",
                    task_id, task_def.description
                );

                // Construct task description with results of dependencies
                let mut full_task_description = task_def.description.clone();
                if !task_def.dependencies.is_empty() {
                    full_task_description.push_str("Context from previous tasks:");
                    for dep_id in &task_def.dependencies {
                        if let Some(result) = plan.task_results.get(dep_id) {
                            full_task_description.push_str(&format!(
                                "- Result of task '{}': {}",dep_id, result));
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
                                "No A2A agent found with skill '{}' for task '{}'",skill,task_id));
                        }
                    }
                } else if let Some(tool_name) = &task_def.tool_to_use {
                    if let Some(mcp) = &self.mcp_agent {
                        let tool_parameters = task_def.tool_parameters.clone().unwrap_or_default();
                        let arguments_map = tool_parameters.as_object().cloned();
                        
                        let call_tool_request_param = CallToolRequestParam { 
                            name: tool_name.to_string().into(), 
                            arguments: arguments_map,
                        };

                        
                        let tool_result = mcp.mcp_client.call_tool(call_tool_request_param).await.unwrap();
                        task_result =
                            serde_json::to_string(&tool_result.content).map_err(|e| {
                                anyhow::anyhow!(
                                    "MCP deserialization error : '{}' ",e)});
                    } else {
                        task_result = Err(anyhow::anyhow!(
                            "MCP agent not initialized, but tool '{}' was requested for task '{}'",tool_name,task_id));
                    }
                } else {
                    // Task requires no specific skill or tool, potentially an LLM reflection task
                    // Use the original user query for general knowledge tasks
                    debug!("FullAgent: Executing general knowledge task for user query: '{}'", plan.user_query);
                    let llm_response = self.llm_interaction.call_api_simple_v2("user".to_string(),plan.user_query.to_string()).await?;
                    debug!("FullAgent: Raw LLM response for general knowledge task: {:?}", llm_response);
                    task_result = Ok(llm_response.expect("Improper task description"));
                }

                // Process the task result immediately
                match task_result {
                    Ok(result_content) => {
                        
                        debug!(
                            "FullAgent: Task '{}' completed successfully. Result : {}",task_id, result_content);

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
                        error!("FullAgent: {}", error_msg);
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
                "FullAgent: Plan execution completed successfully for request ID: {}",plan.request_id);
        } else if matches!(plan.status, PlanStatus::InProgress) {
            let unfinished_tasks: Vec<_> = plan
                .tasks_definition
                .iter()
                .filter(|t| !completed_tasks.contains(&t.id))
                .map(|t| t.id.clone())
                .collect();
            let failure_reason = format!(
                "Plan execution finished, but not all tasks completed. Unfinished: {:?}",unfinished_tasks);
            warn!("FullAgent: {}", failure_reason);
            plan.status = PlanStatus::Failed(failure_reason);
            plan.updated_at = Some(Utc::now());
        }

        Ok(())
    }


   

    
    // todo:investigate about summarization
    // I have some erratic errors in case of general knowledge
    // not sure the output of internal search is transmitted in all cases

    async fn summarize_results(&self, plan: &mut Plan) -> Result<String> {

        info!("FullAgent: Summarizing results for plan ID: {}", plan.id);
        let mut context = format!("User's initial request: {}", plan.user_query);
        context.push_str(&format!("Plan ID: {}\nOverall Plan Summary by LLM: {}\nPlan Status: {:?}\nTasks executed:",
            plan.id, plan.plan_summary, plan.status
        ));

        // To include task results in summary, you would need to store them during execution.
        // Assuming for now we can just list the tasks and their final status.
        // A more complete solution would store task results in the Plan struct or a related structure.

        // Sort tasks by their original definition order for a consistent summary
        let  _sorted_tasks_defs = plan.tasks_definition.clone();
        // Assuming TaskDefinition has a way to maintain original order or we use the order from plan.tasks_definition directly
        // For now, let's just iterate through tasks_definition as is.

        for task_def in &plan.tasks_definition {
            // In a real implementation, you would fetch the execution result for this task_def.id
            // For this functional version, we'll just show the status and description.

            let task_execution_status = match plan.status {
                PlanStatus::Completed => TaskState::Completed, // Assuming all are completed if plan is completed
                PlanStatus::Failed(_) => TaskState::Failed, // Simplified: tasks might not have individual failure reasons here
                _ => TaskState::Working, // Or determine individual task status if stored
            };

            context.push_str(&format!(
                "- Task ID: {}, Description: {}, Status: {:?}, Skill: {:?}",
                task_def.id,
                task_def.description,
                task_execution_status,
                task_def.skill_to_use.as_deref().unwrap_or("N/A")
            ));

            // Include the task output if available
            if let Some(output) = plan.task_results.get(&task_def.id) {
                context.push_str(&format!(", Output: \"{}\"", output)); 
            }
        }

        if plan.status == PlanStatus::Completed {
            context.push_str("All tasks completed successfully. Please provide a concise summary of the overall outcome for the user based on the initial request and the plan summary.");
        } else if let PlanStatus::Failed(reason) = &plan.status {
            context.push_str(&format!("The plan failed. Reason: {}. Please provide a summary for the user of what was attempted and why it failed, based on the initial request and the plan details.", reason));
        } else {
            context.push_str("The plan is still in progress. Provide a brief update based on the plan summary and tasks.");
        }

        debug!("FullAgent: Context for summarization (length: {}): '{}'", context.len(), context);

        ////////////////////////////////////////////////////////////////////////////////////////////////
        // Generate answer based on Context
        // We sometime have to deal with rate limiting constrainst in llm service provider. 
        // Llm chat needed to be adjusted
        // there is still an issue of timeout in a2a implementation that will need to be addressed
        ////////////////////////////////////////////////////////////////////////////////////////////////
        
        let summary_response = self.llm_interaction.call_api_simple_v2("user".to_string(),context.to_string()).await
            .context("LLM API request failed during summarization")?;

        let summary = summary_response.ok_or_else(|| anyhow::anyhow!("LLM returned no content for summarization"))?;
        
        ////////////////////////////////////////////////////////////////////////////////////////////////

        plan.final_summary = Some(summary.clone());
        plan.updated_at = Some(Utc::now());
        debug!("FullAgent: Summary generated.");

        Ok(summary)
    }

        pub async fn submit_user_text(&mut self, user_query: String) ->  anyhow::Result<ExecutionResult>{


            let llm_message_user_request=LlmMessage{
                role: "user".to_string(), // Or appropriate role based on ExecutionResult
                content: Some(user_query),
                tool_call_id: None,
                tool_calls:None
            };

            let execution_result = self.handle_request(llm_message_user_request).await;
            execution_result
        }

    
    // Helper function to extract text from a Message
    // This function is not needed for now
    #[allow(dead_code)]
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
    

}