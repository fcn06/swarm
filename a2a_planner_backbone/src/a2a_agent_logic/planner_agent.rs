use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

// Assuming llm_api crate is available and has these
use llm_api::chat::{ChatCompletionRequest, call_chat_completions};
use llm_api::chat::call_chat_completions_v2;

use crate::PlannerAgentDefinition;
use crate::a2a_plan::plan_definition::{
    ExecutionPlan, ExecutionResult, Plan, PlanResponse, PlanStatus, 
};
use crate::a2a_plan::plan_execution::A2AClient;

// to update
use a2a_rs::domain::{Message, Part, TaskState};

/// Agent that will be in charge of the planning definition and execution
/// He will have access to various a2a resources for this purpose
pub struct PlannerAgent {
    config: PlannerAgentDefinition,
    client_agents: HashMap<String, A2AClient>,
}

impl PlannerAgent {
    pub async fn new(config: PlannerAgentDefinition) -> Result<Self> {
        let mut client_agents = HashMap::new();

        println!("PlannerAgent: Connecting to A2a server agents...");
        for agent_reference in &config.agent_configs {
            // Use agent_info (which implements AgentInfoProvider) to get details for connection
            let agent_reference = agent_reference.get_agent_reference().await?;

            println!(
                "PlannerAgent: Connecting to agent '{}' at {}",
                agent_reference.name, agent_reference.url
            );

            match A2AClient::connect(agent_reference.name.clone(), agent_reference.url.clone())
                .await
            {
                Ok(client) => {
                    println!(
                        "PlannerAgent: Successfully connected to agent '{}' at {}",
                        client.id, client.uri
                    );
                    // Use the connected client's ID as the key
                    client_agents.insert(client.id.clone(), client);
                }
                Err(e) => {
                    // Use details from agent_info for error reporting
                    eprintln!(
                        "PlannerAgent: Warning: Failed to connect to A2a agent '{}' at {}: {}",
                        agent_reference.name, agent_reference.url, e
                    );
                }
            }
        }

        if client_agents.is_empty() && !config.agent_configs.is_empty() {
            println!(
                "PlannerAgent: Warning: No A2a server agents connected, planner capabilities will be limited to direct LLM interaction if any."
            );
            // Depending on requirements, you might return an error here:
            // bail!("Critical: Failed to connect to any A2a server agents.");
        }

        Ok(Self {
            config,
            client_agents,
        })
    }

    async fn get_available_skills_description(&self) -> String {
        let mut description = "Available agent skills:".to_string();
        if self.client_agents.is_empty() {
            description.push_str("- No A2a agents connected.\n",);
        } else {
            for (name, agent) in &self.client_agents {
                description.push_str(&format!("- Agent '{}':", name));
                // Access skills directly from the A2AClient struct
                let skills = agent.get_skills();
                
                if skills.is_empty() {
                    description.push_str(" No specific skills listed.");
                } else {
                    for skill in skills {
                        description.push_str(&format!(" skill.id : {} -- skill.description : {} \n", skill.id,skill.description.clone()));
                    }
                }
                //description.push_str("\n");// Add newline for next agent
                // For testing purpose
                //println!("PlannerAgent: Agent '{}' has skills: {}", name, description);
            }
        }
        
        description
    }

    pub async fn handle_user_request(&mut self, user_request: Message) -> ExecutionResult {
        let request_id = Uuid::new_v4().to_string();

        // Extracting text from message
        let user_query = self.extract_text_from_message(&user_request).await;

        println!(
            "---PlannerAgent: Starting to handle user request --  Query: '{}'---",
            user_query
        );

        match self.create_plan(&user_request).await {
            Ok(mut plan) => {
                println!(
                    "PlannerAgent: Plan created successfully for request ID: {}. Plan ID: {}",
                    request_id, plan.id
                );

                // Attempt to execute the plan
                let _execution_outcome = self.execute_plan(&mut plan).await;

                // Attempt to summarize results regardless of execution outcome
                match self.summarize_results(&mut plan).await {
                    Ok(summary) => {
                        println!(
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
                        eprintln!(
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
                eprintln!("{}", error_msg);
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
        println!(
            "PlannerAgent: Creating plan for request ID: {}",
            Uuid::new_v4().to_string()
        ); // Removed request.id and used a new Uuid

        let skills_description = self.get_available_skills_description().await;

        let prompt = format!(
            "You are a planner agent that creates execution plans for user requests.

            You have access to the following agent skills:
            {}

            User request: {}

            Based on the user request and available skills, create a step-by-step plan to fulfill it.

            The plan should be a JSON object with 'plan_summary' (a brief description of the overall plan) and 'tasks' (an array of task objects).

            Each task object must have the following fields:

            - 'id': A unique string ID for the task (e.g., 'task_1', 'task_web_search').

            - 'description': A clear, concise description of what the task should achieve.

            - 'skill_to_use': (Optional) The specific skill required from an agent (e.g., 'skill_search_web', 'skill_calculate'). If no specific skill is needed or if the task is for the LLM itself to reflect/summarize, this should be null.

            - 'assigned_agent_id_preference': (Optional) If a specific skill is mentioned, suggest the ID of an agent that provides this skill (e.g., 'agent_search'). This is just a preference, the executor will find a suitable agent.

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
                  \"assigned_agent_id_preference\": \"agent_search\",
                  \"dependencies\": [],
                  \"expected_outcome\": \"Relevant search results.\"
                }},
                {{
                  \"id\": \"summarize_info\",
                  \"description\": \"Summarize the information found from the web search.\",
                  \"skill_to_use\": null,
                  \"assigned_agent_id_preference\": null,
                  \"dependencies\": [\"search_web\"],
                  \"expected_outcome\": \"A concise summary.\"
                }}]
            }}

            RETURN ONLY THE SIMPLE JSON REPRESENTING THE PLAN ON THE SAME FORMAT AS ABOVE.",

            skills_description, self.extract_text_from_message(request).await
        );
        // sometime the Json is sent between '''json''' , like in python and this needs to be avoided

        let _messages = vec![Message::user_text(
            prompt.clone().to_string(),
            Uuid::new_v4().to_string(),
        )];

        let messages_draft = vec![llm_api::chat::Message {
            role: "user".to_string(),
            content: prompt.to_string(),
            tool_call_id: None,
        }];

        let llm_request_payload = ChatCompletionRequest {
            model: self.config.model_id.clone(),
            messages: messages_draft,
            // Add other parameters like temperature if needed
            temperature: Some(0.7),
            tool_choice: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: None,
            tools: None,
        };

        let http_client = reqwest::Client::new();

        // The llm_api crate is expected to handle the API key, e.g., via environment variables
        // Assuming 'self.llm_client' is initialized elsewhere with a concrete implementation
        // For now, we'll call the function directly assuming it handles client creation/management
        //let llm_response = call_chat_completions(&http_client, &llm_request_payload)
        //    .await
        //    .context("LLM API request failed during plan creation")?;

        let llm_response = call_chat_completions_v2(&http_client, &llm_request_payload,self.config.llm_url.clone())
        .await
        .context("LLM API request failed during plan creation")?;

        let response_content = llm_response
            .choices
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("LLM response missing choices"))?
            .message
            .content
            .clone();

        // remove think tags from llm response
        let response_content = Some(
            self.remove_think_tags(response_content.clone().expect("REASON"))
                .await?,
        );

        println!(
            "PlannerAgent: LLM responded with plan content:{:?}",
            response_content
        );

        // Attempt to parse the LLM response content as JSON
        // Used to Fails here because the model details his thinking between <think> </think>
        let llm_plan_data: PlanResponse =
            match serde_json::from_str(&response_content.clone().expect("REASON")) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!(
                        "PlannerAgent: Failed to parse LLM plan response as JSON: {}",
                        e
                    );
                    eprintln!("PlannerAgent: LLM Raw Response: {:?}", response_content);
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
        ); // Used Uuid::new_v4().to_string() and extracted text

        //println!("PlannerAgent: Plan created with {} tasks.", plan.tasks_definition.len());
        Ok(plan)
    }

    // to be fine tuned and better tested
    async fn execute_plan(&mut self, plan: &mut Plan) -> Result<()> {
        println!(
            "PlannerAgent: Starting plan execution for request ID: {}",
            plan.request_id
        );
        plan.status = PlanStatus::InProgress;
        plan.updated_at = Some(Utc::now());

        let mut completed_tasks: HashSet<String> = HashSet::new();
        let mut task_queue: VecDeque<String> = VecDeque::new(); // Tasks ready to execute
        let  executing_plans: HashMap<String, ExecutionPlan> = HashMap::new();

        // Initial population of the queue with tasks that have no dependencies
        for task_def in &plan.tasks_definition {
            if task_def.dependencies.is_empty() {
                task_queue.push_back(task_def.id.clone());
            }
        }

        // Process tasks
        while !task_queue.is_empty() || !executing_plans.is_empty() {
            // Submit tasks from the queue that can be started
            while let Some(task_id) = task_queue.pop_front() {
                // Check if task is already completed or being executed
                if completed_tasks.contains(&task_id) || executing_plans.contains_key(&task_id) {
                    continue; // Skip if already processed
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
                    println!(
                        "PlannerAgent: Submitting task '{}': {}",
                        task_id, task_def.description
                    );

                    // Construct task description with results of dependencies
                    let mut full_task_description = task_def.description.clone();
                    if !task_def.dependencies.is_empty() {
                        full_task_description.push_str("Context from previous tasks:\n");
                        for dep_id in &task_def.dependencies {
                            if let Some(result) = plan.task_results.get(dep_id) {
                                full_task_description.push_str(&format!(
                                    "- Result of task '{}': {}\n",
                                    dep_id, result
                                ));
                            }
                        }
                    }

                    // Find a suitable agent or determine it's an LLM task
                    let assigned_agent_id: Option<String>;
                    let task_result: Result<String>;

                    if let Some(skill) = &task_def.skill_to_use {
                        let agent_client = self.find_agent_with_skill(skill, &task_id).await;
                        match agent_client {
                            Some(client) => {
                                assigned_agent_id = Some(client.id.clone());
                                task_result = client
                                    .execute_task(&full_task_description, skill)
                                    .await
                                    .map(|r| r);
                            }
                            None => {
                                assigned_agent_id = None;
                                task_result = Err(anyhow::anyhow!(
                                    "No agent found with skill '{}' for task '{}'",
                                    skill,
                                    task_id
                                ));
                            }
                        }
                    } else {
                        // IMPORTANT : Connect this task to a LLM
                        // Task requires no specific skill, potentially an LLM reflection task
                        assigned_agent_id = None; // No specific agent

                        // Use the full_task_description as the prompt for the LLM
                        let messages_draft = vec![llm_api::chat::Message {
                            role: "user".to_string(),
                            content: full_task_description.to_string(),
                            tool_call_id: None,
                        }];

                        let llm_request_payload = ChatCompletionRequest {
                            model: self.config.model_id.clone(),
                            messages: messages_draft,
                            temperature: Some(0.7),
                            tool_choice: None,
                            max_tokens: None,
                            top_p: None,
                            stop: None,
                            stream: None,
                            tools: None,
                        };

                        let http_client = reqwest::Client::new();
                        task_result = call_chat_completions_v2(&http_client, &llm_request_payload,self.config.llm_url.clone())
                            .await
                            .context("LLM API request failed during reflective task execution")?
                            .choices
                            .get(0)
                            .map(|choice| choice.message.content.clone().unwrap_or_default())
                            .ok_or_else(|| {
                                anyhow::anyhow!("LLM reflective task response missing content")
                            });
                    }

                    // Process the task result immediately for now (sequential execution)
                    match task_result {
                        Ok(result_content) => {
                            // remove think tags from result_content
                            let result_content = self.remove_think_tags(result_content).await?;
                            println!(
                                "PlannerAgent: Task '{}' completed successfully.Result : {}",
                                task_id, result_content
                            );

                            completed_tasks.insert(task_id.clone());
                            plan.task_results
                                .insert(task_id.clone(), result_content.clone()); // Store the result

                            // Update the task_output in the task_definition
                            if let Some(task_def_mut) =
                                plan.tasks_definition.get_mut(task_def_index)
                            {
                                task_def_mut.task_output = Some(result_content);
                            }

                            // Add dependent tasks to the queue
                            for dep_task_def in &plan.tasks_definition {
                                if dep_task_def.dependencies.contains(&task_id) {
                                    task_queue.push_back(dep_task_def.id.clone());
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Task '{}' failed: {}", task_id, e);
                            eprintln!("PlannerAgent: {}", error_msg);
                            plan.status =
                                PlanStatus::Failed(format!("Execution failed at task {}", task_id));
                            plan.updated_at = Some(Utc::now());
                            return Err(anyhow::anyhow!(error_msg)); // Stop plan execution on first failure
                        }
                    }
                } else {
                    // Dependencies not met, push back to queue for later
                    task_queue.push_back(task_id.clone());
                }
            }

            // In a more advanced version, you would poll executing_plans here.
            // Since we process sequentially above, executing_plans will always be empty.
        }

        // After the loop, check if the plan is completed or if there are pending tasks due to uncompleted dependencies or other issues.
        let all_tasks_completed = completed_tasks.len() == plan.tasks_definition.len(); // Check against original task definitions

        if all_tasks_completed {
            plan.status = PlanStatus::Completed;
            plan.updated_at = Some(Utc::now());
            println!(
                "PlannerAgent: Plan execution completed successfully for request ID: {}",
                plan.request_id
            );
        } else if matches!(plan.status, PlanStatus::InProgress) {
            // If not all completed and not already marked as failed
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
            eprintln!("PlannerAgent: {}", failure_reason);
            plan.status = PlanStatus::Failed(failure_reason);
            plan.updated_at = Some(Utc::now());
        }

        Ok(())
    }

    async fn find_agent_with_skill(&self, skill: &str, task_id: &str) -> Option<&A2AClient> {
        // 1. Try preferred agent if specified and has skill
        /*
        if let Some(pref_agent_id) = preferred_agent_id {
            if let Some(agent) = self.client_agents.get(pref_agent_id) {
                // Access skills directly from the A2AClient struct
                if agent.has_skill(skill) { // Use the has_skill method
                    println!("PlannerAgent: Found preferred agent '{}' with skill '{}'.", pref_agent_id, skill);
                    return Some(agent);
                }
            }
        }
        */

        // 2. If no preferred or preferred can't do it, find any agent with the skill
        for (agent_id, agent) in &self.client_agents {
            println!("Test PlannerAgent: Agents : '{}' with skill '{}'.",agent_id, skill);
            // Access skills directly from the A2AClient struct
            if agent.has_skill(skill) {
                // Use the has_skill method
                println!(
                    "PlannerAgent: Found agent '{}' with skill '{}'.",
                    agent_id, skill
                );
                return Some(agent);
            }
        }

        println!("PlannerAgent: No agent found with skill '{}'.", skill);
        None
    }

    async fn summarize_results(&self, plan: &mut Plan) -> Result<String> {
        let http_client = reqwest::Client::new();

        println!("PlannerAgent: Summarizing results for plan ID: {}", plan.id);
        let mut context = format!("User's initial request: {}\n", plan.user_query);
        context.push_str(&format!(
            "Plan ID: {}\nOverall Plan Summary by LLM: {}\nPlan Status: {:?}\nTasks executed:\n",
            plan.id, plan.plan_summary, plan.status
        ));

        // To include task results in summary, you would need to store them during execution.
        // Assuming for now we can just list the tasks and their final status.
        // A more complete solution would store task results in the Plan struct or a related structure.

        // Sort tasks by their original definition order for a consistent summary
        let  sorted_tasks_defs = plan.tasks_definition.clone();
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
                context.push_str(&format!(", Output: \"{}\"", output.replace('\n', " "))); // Replace newlines for cleaner output
            }
            context.push_str(
                "
",
            );
        }

        if plan.status == PlanStatus::Completed {
            context.push_str("All tasks completed successfully. Please provide a concise summary of the overall outcome for the user based on the initial request and the plan summary.");
        } else if let PlanStatus::Failed(reason) = &plan.status {
            context.push_str(&format!("The plan failed. Reason: {}. Please provide a summary for the user of what was attempted and why it failed, based on the initial request and the plan details.", reason));
        } else {
            context.push_str("The plan is still in progress. Provide a brief update based on the plan summary and tasks.");
        }

        let _messages = vec![Message::user_text(
            context.clone().to_string(),
            Uuid::new_v4().to_string(),
        )];

        let messages_draft = vec![llm_api::chat::Message {
            role: "user".to_string(),
            content: context.to_string(),
            tool_call_id: None,
        }];

        let llm_request_payload = ChatCompletionRequest {
            model: self.config.model_id.clone(),
            messages: messages_draft,
            temperature: Some(0.7),
            tool_choice: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: None,
            tools: None,
        };

        // Assuming call_chat_completions handles client internally or uses provided config
        let llm_response = call_chat_completions_v2(&http_client, &llm_request_payload,self.config.llm_url.clone())
            .await
            .context("LLM API request for summary failed")?;

        let summary = llm_response
            .choices
            .get(0)
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("LLM summary response missing content"))?;

        // remove <think> tags from summary
        let summary = self
            .remove_think_tags(summary.clone().expect("REASON"))
            .await?;

        plan.final_summary = Some(summary.clone());
        plan.updated_at = Some(Utc::now());
        println!("PlannerAgent: Summary generated.");

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

    // Helper function to extract text from a Message
    async fn remove_think_tags(&self, result: String) -> anyhow::Result<String> {
        let mut cleaned_result = String::new();
        let mut in_think_tag = false;

        // in case llm returns json markdown
        let result_clean= if result.contains("```json") {
            let result_1=result.replace("```json", "");
            result_1.replace("```", "")
        } else {result};
        let result=result_clean;

        // in case LLM return <think> and </think> tags
        for line in result.lines() {
            if line.contains("<think>") {
                in_think_tag = true;
            }
            if line.contains("</think>") {
                in_think_tag = false;
                continue;
            } // Continue to avoid adding the </think> line itself
            if !in_think_tag {
                cleaned_result.push_str(line);
                cleaned_result.push('\n');
            }
        }
        Ok(cleaned_result)
    }
}
